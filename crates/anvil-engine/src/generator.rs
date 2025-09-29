use std::path::{Path, PathBuf};
use std::os::unix::fs::PermissionsExt;
use tokio::fs;
use tokio::io::AsyncWriteExt;

use crate::engine::{ProcessedTemplate, ProcessedFile};
use crate::error::{EngineError, EngineResult};

pub struct FileGenerator {
    output_dir: PathBuf,
    dry_run: bool,
}

pub type ProgressCallback = Box<dyn Fn(usize, usize, &str) + Send + Sync>;

#[derive(Debug)]
pub struct GenerationResult {
    pub files_created: usize,
    pub directories_created: usize,
    pub bytes_written: u64,
    pub output_directory: PathBuf,
}

impl FileGenerator {
    pub fn new(output_dir: impl Into<PathBuf>) -> Self {
        Self {
            output_dir: output_dir.into(),
            dry_run: false,
        }
    }

    pub fn new_dry_run(output_dir: impl Into<PathBuf>) -> Self {
        Self {
            output_dir: output_dir.into(),
            dry_run: true,
        }
    }

    pub async fn generate_files(
        &self,
        template: ProcessedTemplate,
        progress_callback: Option<ProgressCallback>,
    ) -> EngineResult<GenerationResult> {
        let total_files = template.files.len();
        let mut files_created = 0;
        let mut directories_created = 0;
        let mut bytes_written = 0u64;

        if !self.dry_run {
            fs::create_dir_all(&self.output_dir)
                .await
                .map_err(|e| EngineError::file_error(&self.output_dir, e))?;
        }

        for (index, file) in template.files.into_iter().enumerate() {
            let (file_created, dirs_created, bytes) = Self::write_single_file(&self.output_dir, file, self.dry_run).await?;
            
            if file_created {
                files_created += 1;
            }
            directories_created += dirs_created;
            bytes_written += bytes;
            
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

    async fn write_single_file(
        output_dir: &Path,
        file: ProcessedFile,
        dry_run: bool,
    ) -> EngineResult<(bool, usize, u64)> {
        let full_path = output_dir.join(&file.output_path);
        let mut directories_created = 0;

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
            let mut file_handle = fs::File::create(&full_path)
                .await
                .map_err(|e| EngineError::file_error(&full_path, e))?;
            
            file_handle.write_all(file.content.as_bytes())
                .await
                .map_err(|e| EngineError::file_error(&full_path, e))?;
            
            file_handle.flush()
                .await
                .map_err(|e| EngineError::file_error(&full_path, e))?;

            if file.executable {
                Self::make_executable(&full_path).await?;
            }
        }

        Ok((true, directories_created, bytes_written))
    }

    #[cfg(unix)]
    async fn make_executable(path: &Path) -> EngineResult<()> {
        let metadata = fs::metadata(path)
            .await
            .map_err(|e| EngineError::file_error(path, e))?;
        
        let permissions = metadata.permissions();
        let _mode = permissions.mode();
        
        fs::set_permissions(path, permissions)
            .await
            .map_err(|e| EngineError::file_error(path, e))?;
        
        Ok(())
    }

    #[cfg(not(unix))]
    async fn make_executable(_path: &Path) -> EngineResult<()> {
        Ok(())
    }

    fn count_directories_in_path(created_path: &Path, base_path: &Path) -> usize {
        created_path
            .strip_prefix(base_path)
            .map(|relative| relative.components().count())
            .unwrap_or(0)
    }

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

    pub fn output_directory(&self) -> &Path {
        &self.output_dir
    }

    pub fn is_dry_run(&self) -> bool {
        self.dry_run
    }
}

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
        assert!(result.bytes_written > 0);
        assert_eq!(result.output_directory, output_dir);
        
        assert!(output_dir.join("main.rs").exists());
        assert!(output_dir.join("src/lib.rs").exists());
        assert!(output_dir.join("scripts/build.sh").exists());
        
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
        
        assert!(!output_dir.exists());
    }

    #[tokio::test]
    async fn test_directory_status_check() {
        let temp_dir = TempDir::new().unwrap();
        let output_dir = temp_dir.path().join("output");
        
        let generator = FileGenerator::new(&output_dir);
        
        let status = generator.check_output_directory().await.unwrap();
        assert_eq!(status, DirectoryStatus::DoesNotExist);
        
        fs::create_dir_all(&output_dir).await.unwrap();
        let status = generator.check_output_directory().await.unwrap();
        assert_eq!(status, DirectoryStatus::ExistsEmpty);
        
        fs::write(output_dir.join("test.txt"), "content").await.unwrap();
        let status = generator.check_output_directory().await.unwrap();
        assert_eq!(status, DirectoryStatus::ExistsWithContent);
    }

    #[tokio::test]
    async fn test_clean_output_directory() {
        let temp_dir = TempDir::new().unwrap();
        let output_dir = temp_dir.path().join("output");
        
        fs::create_dir_all(&output_dir).await.unwrap();
        fs::write(output_dir.join("test.txt"), "content").await.unwrap();
        
        let generator = FileGenerator::new(&output_dir);
        
        let status = generator.check_output_directory().await.unwrap();
        assert_eq!(status, DirectoryStatus::ExistsWithContent);
        
        generator.clean_output_directory().await.unwrap();
        
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

