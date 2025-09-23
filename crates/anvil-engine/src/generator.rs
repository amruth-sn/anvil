use std::path::{Path, PathBuf};
use std::os::unix::fs::PermissionsExt;
use tokio::fs;
use tokio::io::AsyncWriteExt;

use crate::engine::{ProcessedTemplate, ProcessedFile};
use crate::error::{EngineError, EngineResult};

/// File generator that writes processed files to disk
pub struct FileGenerator {
    output_dir: PathBuf,
    dry_run: bool,
}

/// Progress callback for file generation
pub type ProgressCallback = Box<dyn Fn(usize, usize, &str) + Send + Sync>;

/// Result of file generation operation
#[derive(Debug)]
pub struct GenerationResult {
    pub files_created: usize,
    pub directories_created: usize,
    pub bytes_written: u64,
    pub output_directory: PathBuf,
}

impl FileGenerator {
    /// Create a new file generator
    pub fn new(output_dir: impl Into<PathBuf>) -> Self {
        Self {
            output_dir: output_dir.into(),
            dry_run: false,
        }
    }

    /// Create a file generator in dry-run mode (doesn't actually write files)
    pub fn new_dry_run(output_dir: impl Into<PathBuf>) -> Self {
        Self {
            output_dir: output_dir.into(),
            dry_run: true,
        }
    }

    /// Generate all files from a processed template
    pub async fn generate_files(
        &self,
        template: ProcessedTemplate,
        progress_callback: Option<ProgressCallback>,
    ) -> EngineResult<GenerationResult> {
        let total_files = template.files.len();
        let mut files_created = 0;
        let mut directories_created = 0;
        let mut bytes_written = 0u64;

        // Ensure output directory exists
        if !self.dry_run {
            fs::create_dir_all(&self.output_dir)
                .await
                .map_err(|e| EngineError::file_error(&self.output_dir, e))?;
        }

        // Process files sequentially for now (can be optimized later)
        for (index, file) in template.files.into_iter().enumerate() {
            let (file_created, dirs_created, bytes) = Self::write_single_file(&self.output_dir, file, self.dry_run).await?;
            
            if file_created {
                files_created += 1;
            }
            directories_created += dirs_created;
            bytes_written += bytes;
            
            // Report progress if callback provided
            if let Some(callback) = &progress_callback {
                callback(index + 1, total_files, "Processing files");
            }
        }

        Ok(GenerationResult {
            files_created,
            directories_created,
            bytes_written,
            output_directory: self.output_dir.clone(),
        })
    }

    /// Write a single file to disk
    async fn write_single_file(
        output_dir: &Path,
        file: ProcessedFile,
        dry_run: bool,
    ) -> EngineResult<(bool, usize, u64)> {
        let full_path = output_dir.join(&file.output_path);
        let mut directories_created = 0;

        // Ensure parent directory exists
        if let Some(parent) = full_path.parent() {
            if !dry_run {
                if !parent.exists() {
                    fs::create_dir_all(parent)
                        .await
                        .map_err(|e| EngineError::file_error(parent, e))?;
                    directories_created = Self::count_directories_in_path(parent, output_dir);
                }
            }
        }

        let bytes_written = file.content.len() as u64;

        if !dry_run {
            // Write file content
            let mut file_handle = fs::File::create(&full_path)
                .await
                .map_err(|e| EngineError::file_error(&full_path, e))?;
            
            file_handle.write_all(file.content.as_bytes())
                .await
                .map_err(|e| EngineError::file_error(&full_path, e))?;
            
            file_handle.flush()
                .await
                .map_err(|e| EngineError::file_error(&full_path, e))?;

            // Set executable permissions if needed
            if file.executable {
                Self::make_executable(&full_path).await?;
            }
        }

        Ok((true, directories_created, bytes_written))
    }

    /// Make a file executable on Unix systems
    #[cfg(unix)]
    async fn make_executable(path: &Path) -> EngineResult<()> {
        let metadata = fs::metadata(path)
            .await
            .map_err(|e| EngineError::file_error(path, e))?;
        
        let mut permissions = metadata.permissions();
        let mode = permissions.mode();
        permissions.set_mode(mode | 0o111); // Add execute permission for owner, group, and others
        
        fs::set_permissions(path, permissions)
            .await
            .map_err(|e| EngineError::file_error(path, e))?;
        
        Ok(())
    }

    /// Make a file executable on non-Unix systems (no-op)
    #[cfg(not(unix))]
    async fn make_executable(_path: &Path) -> EngineResult<()> {
        // No-op on Windows
        Ok(())
    }

    /// Count the number of directories created in a path
    fn count_directories_in_path(created_path: &Path, base_path: &Path) -> usize {
        created_path
            .strip_prefix(base_path)
            .map(|relative| relative.components().count())
            .unwrap_or(0)
    }

    /// Check if output directory already exists and has content
    pub async fn check_output_directory(&self) -> EngineResult<DirectoryStatus> {
        if !self.output_dir.exists() {
            return Ok(DirectoryStatus::DoesNotExist);
        }

        let mut entries = fs::read_dir(&self.output_dir)
            .await
            .map_err(|e| EngineError::file_error(&self.output_dir, e))?;

        if entries.next_entry().await
            .map_err(|e| EngineError::file_error(&self.output_dir, e))?
            .is_some() {
            Ok(DirectoryStatus::ExistsWithContent)
        } else {
            Ok(DirectoryStatus::ExistsEmpty)
        }
    }

    /// Clean the output directory (remove all contents)
    pub async fn clean_output_directory(&self) -> EngineResult<()> {
        if self.dry_run {
            return Ok(());
        }

        if self.output_dir.exists() {
            fs::remove_dir_all(&self.output_dir)
                .await
                .map_err(|e| EngineError::file_error(&self.output_dir, e))?;
        }

        fs::create_dir_all(&self.output_dir)
            .await
            .map_err(|e| EngineError::file_error(&self.output_dir, e))?;

        Ok(())
    }

    /// Get the output directory path
    pub fn output_directory(&self) -> &Path {
        &self.output_dir
    }

    /// Check if this is a dry run
    pub fn is_dry_run(&self) -> bool {
        self.dry_run
    }
}

/// Status of the output directory
#[derive(Debug, PartialEq, Eq)]
pub enum DirectoryStatus {
    DoesNotExist,
    ExistsEmpty,
    ExistsWithContent,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{ProcessedTemplate, ProcessedFile};
    use tempfile::TempDir;
    use std::path::PathBuf;

    fn create_test_processed_template() -> ProcessedTemplate {
        ProcessedTemplate {
            files: vec![
                ProcessedFile {
                    output_path: PathBuf::from("main.rs"),
                    content: "fn main() { println!(\"Hello, world!\"); }".to_string(),
                    executable: false,
                },
                ProcessedFile {
                    output_path: PathBuf::from("src/lib.rs"),
                    content: "// Library code".to_string(),
                    executable: false,
                },
                ProcessedFile {
                    output_path: PathBuf::from("scripts/build.sh"),
                    content: "#!/bin/bash\necho 'Building...'".to_string(),
                    executable: true,
                },
            ],
        }
    }

    #[tokio::test]
    async fn test_file_generation() {
        let temp_dir = TempDir::new().unwrap();
        let output_dir = temp_dir.path().join("output");
        
        let generator = FileGenerator::new(&output_dir);
        let template = create_test_processed_template();
        
        let result = generator.generate_files(template, None).await.unwrap();
        
        assert_eq!(result.files_created, 3);
        assert!(result.directories_created > 0); // src/ and scripts/ directories
        assert!(result.bytes_written > 0);
        assert_eq!(result.output_directory, output_dir);
        
        // Verify files were created
        assert!(output_dir.join("main.rs").exists());
        assert!(output_dir.join("src/lib.rs").exists());
        assert!(output_dir.join("scripts/build.sh").exists());
        
        // Verify content
        let main_content = fs::read_to_string(output_dir.join("main.rs")).await.unwrap();
        assert!(main_content.contains("Hello, world!"));
    }

    #[tokio::test]
    async fn test_dry_run() {
        let temp_dir = TempDir::new().unwrap();
        let output_dir = temp_dir.path().join("output");
        
        let generator = FileGenerator::new_dry_run(&output_dir);
        let template = create_test_processed_template();
        
        let result = generator.generate_files(template, None).await.unwrap();
        
        assert_eq!(result.files_created, 3);
        assert!(result.bytes_written > 0);
        
        // Verify no files were actually created
        assert!(!output_dir.exists());
    }

    #[tokio::test]
    async fn test_directory_status_check() {
        let temp_dir = TempDir::new().unwrap();
        let output_dir = temp_dir.path().join("output");
        
        let generator = FileGenerator::new(&output_dir);
        
        // Non-existent directory
        let status = generator.check_output_directory().await.unwrap();
        assert_eq!(status, DirectoryStatus::DoesNotExist);
        
        // Empty directory
        fs::create_dir_all(&output_dir).await.unwrap();
        let status = generator.check_output_directory().await.unwrap();
        assert_eq!(status, DirectoryStatus::ExistsEmpty);
        
        // Directory with content
        fs::write(output_dir.join("test.txt"), "content").await.unwrap();
        let status = generator.check_output_directory().await.unwrap();
        assert_eq!(status, DirectoryStatus::ExistsWithContent);
    }

    #[tokio::test]
    async fn test_clean_output_directory() {
        let temp_dir = TempDir::new().unwrap();
        let output_dir = temp_dir.path().join("output");
        
        // Create directory with content
        fs::create_dir_all(&output_dir).await.unwrap();
        fs::write(output_dir.join("test.txt"), "content").await.unwrap();
        
        let generator = FileGenerator::new(&output_dir);
        
        // Verify content exists
        let status = generator.check_output_directory().await.unwrap();
        assert_eq!(status, DirectoryStatus::ExistsWithContent);
        
        // Clean directory
        generator.clean_output_directory().await.unwrap();
        
        // Verify directory is now empty
        let status = generator.check_output_directory().await.unwrap();
        assert_eq!(status, DirectoryStatus::ExistsEmpty);
    }

    #[tokio::test]
    async fn test_progress_callback() {
        let temp_dir = TempDir::new().unwrap();
        let output_dir = temp_dir.path().join("output");
        
        let generator = FileGenerator::new(&output_dir);
        let template = create_test_processed_template();
        
        let progress_counter = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let counter_clone = progress_counter.clone();
        
        let progress_callback: ProgressCallback = Box::new(move |current, total, _msg| {
            counter_clone.store(current, std::sync::atomic::Ordering::Relaxed);
            assert!(current <= total);
        });
        
        let result = generator.generate_files(template, Some(progress_callback)).await.unwrap();
        
        assert_eq!(result.files_created, 3);
        assert_eq!(progress_counter.load(std::sync::atomic::Ordering::Relaxed), 3);
    }
}

