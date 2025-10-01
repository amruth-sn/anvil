/*!
 * Integration Testing Suite for Anvil Template Engine
 *
 * This module provides comprehensive integration tests for:
 * - Template generation across different configurations
 * - Shared component compatibility and composition
 * - Cross-template service integration
 * - Build and runtime verification
 */

pub mod build_verification;
pub mod service_compatibility;
pub mod shared_components;
pub mod template_generation;

use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use tokio::process::Command;

/// Test configuration for different template scenarios
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TestConfig {
    pub template_name: String,
    pub variables: std::collections::HashMap<String, Value>,
    pub expected_files: Vec<String>,
    pub expected_dependencies: Vec<String>,
    pub should_build: bool,
    pub should_run: bool,
    pub timeout_seconds: u64,
}

/// Test result with detailed information
#[derive(Debug)]
pub struct TestResult {
    pub success: bool,
    pub duration_ms: u128,
    pub files_created: usize,
    pub build_success: Option<bool>,
    pub runtime_success: Option<bool>,
    pub error_message: Option<String>,
    pub warnings: Vec<String>,
    pub project_name: Option<String>,
}

/// Main test runner for integration tests
pub struct IntegrationTestSuite {
    pub workspace_dir: PathBuf,
    pub temp_dir: TempDir,
    pub anvil_binary: PathBuf,
}

impl IntegrationTestSuite {
    /// Create a new test suite instance
    pub fn new() -> anyhow::Result<Self> {
        let current_dir = std::env::current_dir()?;
        let temp_dir = TempDir::new()?;

        // Find workspace root - look for Cargo.toml with workspace members
        let workspace_dir = if current_dir.join("Cargo.toml").exists()
            && std::fs::read_to_string(current_dir.join("Cargo.toml"))?.contains("[workspace]")
        {
            current_dir
        } else if current_dir.parent().is_some()
            && current_dir.parent().unwrap().join("Cargo.toml").exists()
            && std::fs::read_to_string(current_dir.parent().unwrap().join("Cargo.toml"))?
                .contains("[workspace]")
        {
            current_dir.parent().unwrap().to_path_buf()
        } else if current_dir.parent().is_some()
            && current_dir.parent().unwrap().parent().is_some()
            && current_dir
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .join("Cargo.toml")
                .exists()
            && std::fs::read_to_string(
                current_dir
                    .parent()
                    .unwrap()
                    .parent()
                    .unwrap()
                    .join("Cargo.toml"),
            )?
            .contains("[workspace]")
        {
            current_dir
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .to_path_buf()
        } else {
            anyhow::bail!("Could not find workspace root");
        };

        // Try release binary first, then debug binary
        let anvil_binary = if workspace_dir.join("target/release/anvil").exists() {
            workspace_dir.join("target/release/anvil")
        } else if workspace_dir.join("target/debug/anvil").exists() {
            workspace_dir.join("target/debug/anvil")
        } else {
            anyhow::bail!(
                "Anvil binary not found. Run 'cargo build' or 'cargo build --release' first."
            );
        };

        Ok(Self {
            workspace_dir,
            temp_dir,
            anvil_binary,
        })
    }

    /// Generate a project using anvil CLI
    pub async fn generate_project(&self, config: &TestConfig) -> anyhow::Result<TestResult> {
        let start_time = std::time::Instant::now();

        // Create unique project name based on template name and short suffix
        let unique_suffix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            % 10000; // Use last 4 digits to keep it short
        let project_name = format!("test-{}", unique_suffix);
        let project_dir = self.temp_dir.path().join(&project_name);

        // Prepare command arguments - run from workspace with output to temp dir
        let mut cmd = Command::new(&self.anvil_binary);
        cmd.arg("create")
            .arg(&project_name)
            .arg("--template")
            .arg(&config.template_name)
            .arg("--output")
            .arg(self.temp_dir.path())
            .arg("--no-input")
            .arg("--force")
            .current_dir(&self.workspace_dir);

        // Debug: print the command being executed (remove when stable)
        // println!("🔍 Running command: {:?} in workspace {:?}", cmd, self.workspace_dir);

        // Map variables to CLI service arguments
        for (key, value) in &config.variables {
            if let Value::String(string_value) = value {
                match key.as_str() {
                    "auth_provider" => {
                        cmd.arg("--auth").arg(string_value);
                    }
                    "api_pattern" => {
                        cmd.arg("--api").arg(string_value);
                    }
                    "database_provider" => {
                        cmd.arg("--database").arg(string_value);
                    }
                    "ai_provider" => {
                        cmd.arg("--ai").arg(string_value);
                    }
                    "payment_provider" => {
                        cmd.arg("--payments").arg(string_value);
                    }
                    "deployment_target" => {
                        cmd.arg("--deployment").arg(string_value);
                    }
                    "monitoring" => {
                        cmd.arg("--monitoring").arg(string_value);
                    }
                    "language" => {
                        cmd.arg("--language").arg(string_value);
                    }
                    _ => {
                        // For other variables, we'll ignore them for now
                        // as they are not service-related
                    }
                }
            }
        }

        // Execute generation
        let output = cmd.output().await?;
        let duration = start_time.elapsed();

        if !output.status.success() {
            return Ok(TestResult {
                success: false,
                duration_ms: duration.as_millis(),
                files_created: 0,
                build_success: None,
                runtime_success: None,
                error_message: Some(String::from_utf8_lossy(&output.stderr).to_string()),
                warnings: vec![],
                project_name: None,
            });
        }

        // Count created files
        let files_created = self.count_files_recursive(&project_dir)?;

        // Verify expected files exist
        let mut warnings = Vec::new();
        for expected_file in &config.expected_files {
            let file_path = project_dir.join(expected_file);
            if !file_path.exists() {
                warnings.push(format!("Expected file not found: {}", expected_file));
            }
        }

        Ok(TestResult {
            success: true,
            duration_ms: duration.as_millis(),
            files_created,
            build_success: None,
            runtime_success: None,
            error_message: None,
            warnings,
            project_name: Some(project_name),
        })
    }

    /// Count files recursively in a directory
    pub fn count_files_recursive(&self, dir: &Path) -> anyhow::Result<usize> {
        let mut count = 0;
        if dir.exists() && dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() {
                    count += 1;
                } else if path.is_dir() {
                    count += self.count_files_recursive(&path)?;
                }
            }
        }
        Ok(count)
    }

    /// Run build verification on generated project
    pub async fn verify_build(
        &self,
        project_name: &str,
        package_manager: &str,
    ) -> anyhow::Result<bool> {
        let project_dir = self.temp_dir.path().join(project_name);

        // Install dependencies
        let install_cmd = match package_manager {
            "npm" => vec!["npm", "install"],
            "pnpm" => vec!["pnpm", "install"],
            "yarn" => vec!["yarn", "install"],
            "bun" => vec!["bun", "install"],
            _ => vec!["npm", "install"],
        };

        let output = Command::new(install_cmd[0])
            .args(&install_cmd[1..])
            .current_dir(&project_dir)
            .output()
            .await?;

        if !output.status.success() {
            return Ok(false);
        }

        // Run build
        let build_cmd = match package_manager {
            "npm" => vec!["npm", "run", "build"],
            "pnpm" => vec!["pnpm", "run", "build"],
            "yarn" => vec!["yarn", "build"],
            "bun" => vec!["bun", "run", "build"],
            _ => vec!["npm", "run", "build"],
        };

        let output = Command::new(build_cmd[0])
            .args(&build_cmd[1..])
            .current_dir(&project_dir)
            .output()
            .await?;

        Ok(output.status.success())
    }

    /// Run runtime verification (start dev server briefly)
    pub async fn verify_runtime(
        &self,
        project_name: &str,
        package_manager: &str,
    ) -> anyhow::Result<bool> {
        let project_dir = self.temp_dir.path().join(project_name);

        let dev_cmd = match package_manager {
            "npm" => vec!["npm", "run", "dev"],
            "pnpm" => vec!["pnpm", "run", "dev"],
            "yarn" => vec!["yarn", "dev"],
            "bun" => vec!["bun", "run", "dev"],
            _ => vec!["npm", "run", "dev"],
        };

        // Start dev server in background
        let mut child = Command::new(dev_cmd[0])
            .args(&dev_cmd[1..])
            .current_dir(&project_dir)
            .spawn()?;

        // Wait a few seconds for server to start
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

        // Try to connect to localhost:3000
        let client = reqwest::Client::new();
        let response = client
            .get("http://localhost:3000")
            .timeout(tokio::time::Duration::from_secs(3))
            .send()
            .await;

        // Kill the dev server
        let _ = child.kill().await;

        match response {
            Ok(resp) => Ok(resp.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}
