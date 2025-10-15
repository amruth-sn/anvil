/*!
 * Simple Integration Test
 * 
 * A basic integration test to verify anvil functionality
 */

use std::process::Command;
use tempfile::TempDir;

#[tokio::test]
async fn test_basic_anvil_generation() {
    println!("🔥 Testing basic anvil project generation");
    
    // Create a temporary directory for the test
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let _temp_path = temp_dir.path();
    
    // Find the anvil binary with absolute path
    let current_dir = std::env::current_dir()
        .expect("Failed to get current dir");
    let workspace_dir = current_dir
        .parent()
        .expect("Failed to get parent")
        .parent()
        .expect("Failed to get grandparent");
    
    let binary_name = if cfg!(target_os = "windows") {
        "anvil.exe"
    } else {
        "anvil"
    };
    
    // Try release binary first, then debug binary
    let anvil_binary = if workspace_dir.join("target/release").join(binary_name).exists() {
        workspace_dir.join("target/release").join(binary_name)
    } else {
        workspace_dir.join("target/debug").join(binary_name)
    };
    
    // Test basic project generation from anvil workspace
    let workspace_dir = std::path::Path::new("../../");
    let output = Command::new(anvil_binary)
        .arg("create")
        .arg("test-basic")
        .arg("--template")
        .arg("rust-hello-world")
        .arg("--no-input")
        .current_dir(workspace_dir)
        .output()
        .expect("Failed to execute anvil command");
    
    println!("Command output: {}", String::from_utf8_lossy(&output.stdout));
    if !output.stderr.is_empty() {
        println!("Command stderr: {}", String::from_utf8_lossy(&output.stderr));
    }
    
    // Check if command succeeded
    if !output.status.success() {
        panic!("Anvil command failed with status: {}", output.status);
    }
    
    // Check if project directory was created in workspace
    let project_dir = workspace_dir.join("test-basic");
    assert!(project_dir.exists(), "Project directory should be created");
    
    // Check if basic files exist
    assert!(project_dir.join("Cargo.toml").exists(), "Cargo.toml should exist");
    assert!(project_dir.join("src").exists(), "src directory should exist");
    assert!(project_dir.join("src/main.rs").exists(), "main.rs should exist");
    
    // Clean up the test project
    if project_dir.exists() {
        std::fs::remove_dir_all(&project_dir).ok();
    }
    
    println!("✅ Basic anvil generation test passed!");
}

#[test]
fn test_anvil_binary_exists() {
    println!("🔍 Checking if anvil binary exists");
    
    let binary_name = if cfg!(target_os = "windows") {
        "anvil.exe"
    } else {
        "anvil"
    };
    
    // Try release binary first, then debug binary
    let anvil_binary = if std::path::Path::new("../../target/release").join(binary_name).exists() {
        format!("../../target/release/{}", binary_name)
    } else {
        format!("../../target/debug/{}", binary_name)
    };
    
    let binary_path = std::path::Path::new(&anvil_binary);
    assert!(binary_path.exists(), "Anvil binary should exist at {}", anvil_binary);
    
    println!("✅ Anvil binary found at: {}", anvil_binary);
}

#[test]
fn test_template_directory_exists() {
    println!("📁 Checking if templates directory exists");
    
    let templates_dir = std::path::Path::new("../../templates");
    assert!(templates_dir.exists(), "Templates directory should exist");
    
    // Check for some expected templates
    let fullstack_saas = templates_dir.join("fullstack-saas");
    let rust_hello = templates_dir.join("rust-hello-world");
    
    assert!(fullstack_saas.exists(), "fullstack-saas template should exist");
    assert!(rust_hello.exists(), "rust-hello-world template should exist");
    
    println!("✅ Template directory validation passed!");
}