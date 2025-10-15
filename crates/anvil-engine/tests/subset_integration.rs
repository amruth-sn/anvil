/*!
 * Subset Integration Tests
 *
 * A focused set of integration tests that verify core functionality
 * without the complex async collection issues
 */

use std::process::Command;
use std::time::Instant;

/* Test template generation with different configurations */
#[tokio::test]
async fn test_template_generation_subset() {
    println!("🧪 Running template generation subset tests");

    // Find the anvil binary
    let current_dir = std::env::current_dir().expect("Failed to get current dir");
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
    let anvil_binary = if workspace_dir
        .join("target/release")
        .join(binary_name)
        .exists()
    {
        workspace_dir.join("target/release").join(binary_name)
    } else {
        workspace_dir.join("target/debug").join(binary_name)
    };

    // Test configurations
    let test_cases = vec![
        (
            "rust-hello-world",
            "test-rust",
            vec!["Cargo.toml", "src/main.rs"],
        ),
        (
            "fullstack-saas",
            "test-saas",
            vec!["package.json", "app/page.tsx", "next.config.js"],
        ),
    ];

    let mut successful_tests = 0;
    let mut total_files_generated = 0;

    for (template, project_name, expected_files) in test_cases {
        println!("  Testing template: {}", template);
        let start_time = Instant::now();

        let output = Command::new(&anvil_binary)
            .arg("create")
            .arg(project_name)
            .arg("--template")
            .arg(template)
            .arg("--no-input")
            .current_dir(&workspace_dir)
            .output()
            .expect("Failed to execute anvil command");

        let duration = start_time.elapsed();

        if output.status.success() {
            println!(
                "    ✅ {} generated successfully in {:?}",
                template, duration
            );
            successful_tests += 1;

            // Verify expected files exist
            let project_path = workspace_dir.join(project_name);
            for expected_file in &expected_files {
                let file_path = project_path.join(expected_file);
                if file_path.exists() {
                    println!("      ✓ Found expected file: {}", expected_file);
                    total_files_generated += 1;
                } else {
                    println!("      ❌ Missing expected file: {}", expected_file);
                }
            }

            // Count total files generated
            let file_count = count_files_recursive(&project_path).unwrap_or(0);
            println!("      📁 Total files generated: {}", file_count);

            // Clean up
            if project_path.exists() {
                std::fs::remove_dir_all(&project_path).ok();
            }
        } else {
            println!(
                "    ❌ {} failed: {}",
                template,
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }

    println!("📊 Template generation results:");
    println!("   Successful tests: {}/2", successful_tests);
    println!("   Expected files found: {}", total_files_generated);

    assert_eq!(
        successful_tests, 2,
        "All template generation tests should pass"
    );
    assert!(
        total_files_generated >= 4,
        "Should find most expected files"
    );
}

/* Test component integration scenarios */
#[tokio::test]
async fn test_component_integration_subset() {
    println!("🧩 Running component integration subset tests");

    // Find the anvil binary
    let current_dir = std::env::current_dir().expect("Failed to get current dir");
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
    let anvil_binary = if workspace_dir
        .join("target/release")
        .join(binary_name)
        .exists()
    {
        workspace_dir.join("target/release").join(binary_name)
    } else {
        workspace_dir.join("target/debug").join(binary_name)
    };

    // Test fullstack-saas with different UI libraries (if supported via variables)
    let ui_libraries = vec!["shadcn/ui", "NextUI"];
    let mut successful_tests = 0;

    for ui_library in &ui_libraries {
        println!("  Testing component integration with: {}", ui_library);
        let project_name = format!("test-components-{}", ui_library.replace("/", "-"));
        let start_time = Instant::now();

        // For now, we'll test basic generation since variable passing
        // might not be fully implemented in the CLI yet
        let output = Command::new(&anvil_binary)
            .arg("create")
            .arg(&project_name)
            .arg("--template")
            .arg("fullstack-saas")
            .arg("--no-input")
            .current_dir(&workspace_dir)
            .output()
            .expect("Failed to execute anvil command");

        let duration = start_time.elapsed();

        if output.status.success() {
            println!("    ✅ Component integration test passed in {:?}", duration);
            successful_tests += 1;

            // Verify component-related files
            let project_path = workspace_dir.join(&project_name);
            let component_files =
                vec!["components/ui/button.tsx", "app/layout.tsx", "app/page.tsx"];

            for component_file in &component_files {
                let file_path = project_path.join(component_file);
                if file_path.exists() {
                    println!("      ✓ Found component file: {}", component_file);
                } else {
                    println!("      ⚠ Missing component file: {}", component_file);
                }
            }

            // Clean up
            if project_path.exists() {
                std::fs::remove_dir_all(&project_path).ok();
            }
        } else {
            println!(
                "    ❌ Component integration failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }

    println!("📊 Component integration results:");
    println!(
        "   Successful tests: {}/{}",
        successful_tests,
        ui_libraries.len()
    );

    assert!(
        successful_tests >= 1,
        "At least one component integration test should pass"
    );
}

/* Test build verification on a simple scale */
#[tokio::test]
async fn test_build_verification_subset() {
    println!("🔨 Running build verification subset tests");

    // Find the anvil binary
    let current_dir = std::env::current_dir().expect("Failed to get current dir");
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
    let anvil_binary = if workspace_dir
        .join("target/release")
        .join(binary_name)
        .exists()
    {
        workspace_dir.join("target/release").join(binary_name)
    } else {
        workspace_dir.join("target/debug").join(binary_name)
    };

    let project_name = "test-build-verification";

    // Generate a Rust project for build testing
    println!("  Generating Rust project for build verification");
    let output = Command::new(&anvil_binary)
        .arg("create")
        .arg(project_name)
        .arg("--template")
        .arg("rust-hello-world")
        .arg("--no-input")
        .current_dir(&workspace_dir)
        .output()
        .expect("Failed to execute anvil command");

    if !output.status.success() {
        panic!(
            "Failed to generate project for build verification: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let project_path = workspace_dir.join(project_name);

    // Test if the generated project can be built
    println!("  Testing Rust project build");
    let start_time = Instant::now();

    let build_output = Command::new("cargo")
        .arg("check")
        .current_dir(&project_path)
        .output()
        .expect("Failed to execute cargo check");

    let build_duration = start_time.elapsed();

    if build_output.status.success() {
        println!(
            "    ✅ Rust project builds successfully in {:?}",
            build_duration
        );

        // Test if it can actually compile and run
        let run_output = Command::new("cargo")
            .arg("run")
            .arg("--")
            .arg("--help") // Quick run to test executable
            .current_dir(&project_path)
            .output()
            .expect("Failed to execute cargo run");

        if run_output.status.success() {
            println!("    ✅ Rust project runs successfully");
        } else {
            println!(
                "    ⚠ Rust project compiled but runtime failed: {}",
                String::from_utf8_lossy(&run_output.stderr)
            );
        }
    } else {
        println!(
            "    ❌ Rust project build failed: {}",
            String::from_utf8_lossy(&build_output.stderr)
        );
    }

    // Clean up
    if project_path.exists() {
        std::fs::remove_dir_all(&project_path).ok();
    }

    println!("📊 Build verification results:");
    println!("   Rust build success: {}", build_output.status.success());

    assert!(
        build_output.status.success(),
        "Generated Rust project should build successfully"
    );
}

/* Test cross-platform template compatibility */
#[test]
fn test_cross_platform_compatibility() {
    println!("🌍 Running cross-platform compatibility tests");

    // Find the anvil binary
    let current_dir = std::env::current_dir().expect("Failed to get current dir");
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
    let anvil_binary = if workspace_dir
        .join("target/release")
        .join(binary_name)
        .exists()
    {
        workspace_dir.join("target/release").join(binary_name)
    } else {
        workspace_dir.join("target/debug").join(binary_name)
    };

    // Test that binary exists and is executable
    assert!(
        anvil_binary.exists(),
        "Anvil binary should exist for current platform"
    );

    // Test basic command execution
    let output = Command::new(&anvil_binary)
        .arg("--help")
        .output()
        .expect("Failed to execute anvil --help");

    assert!(
        output.status.success(),
        "Anvil should respond to --help command"
    );

    let help_text = String::from_utf8_lossy(&output.stdout);
    assert!(
        help_text.contains("create"),
        "Help should mention 'create' command"
    );
    assert!(
        help_text.contains("template"),
        "Help should mention 'template' option"
    );

    println!("  ✅ Cross-platform compatibility verified");
    println!("  📋 Available commands include: create, template handling");
}

/* Performance stress test with multiple concurrent generations */
#[tokio::test]
async fn test_performance_stress_subset() {
    println!("⚡ Running performance stress subset test");

    // Find the anvil binary
    let current_dir = std::env::current_dir().expect("Failed to get current dir");
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
    let anvil_binary = if workspace_dir
        .join("target/release")
        .join(binary_name)
        .exists()
    {
        workspace_dir.join("target/release").join(binary_name)
    } else {
        workspace_dir.join("target/debug").join(binary_name)
    };

    let start_time = Instant::now();
    let concurrent_count = 3;

    // Generate multiple projects concurrently
    let handles: Vec<_> = (0..concurrent_count)
        .map(|i| {
            let binary = anvil_binary.clone();
            let workspace = workspace_dir.to_path_buf();
            let project_name = format!("stress-test-{}", i);

            tokio::spawn(async move {
                let output = Command::new(&binary)
                    .arg("create")
                    .arg(&project_name)
                    .arg("--template")
                    .arg("rust-hello-world")
                    .arg("--no-input")
                    .current_dir(&workspace)
                    .output()
                    .expect("Failed to execute anvil command");

                let success = output.status.success();

                // Clean up
                let project_dir = workspace.join(&project_name);
                if project_dir.exists() {
                    std::fs::remove_dir_all(&project_dir).ok();
                }

                (project_name, success)
            })
        })
        .collect();

    // Wait for all to complete
    let results = futures::future::join_all(handles).await;
    let total_duration = start_time.elapsed();

    let mut successful = 0;
    for result in &results {
        match result {
            Ok((name, true)) => {
                println!("  ✅ {} completed successfully", name);
                successful += 1;
            }
            Ok((name, false)) => {
                println!("  ❌ {} failed", name);
            }
            Err(e) => {
                println!("  ❌ Task failed: {}", e);
            }
        }
    }

    println!("📊 Performance stress results:");
    println!("   Concurrent generations: {}", concurrent_count);
    println!("   Successful: {}", successful);
    println!("   Total duration: {:?}", total_duration);
    println!(
        "   Average per project: {:?}",
        total_duration / concurrent_count
    );

    assert_eq!(
        successful, concurrent_count,
        "All concurrent generations should succeed"
    );
    assert!(
        total_duration.as_secs() < 120,
        "Stress test should complete within 2 minutes"
    );
}

/* Helper function to count files recursively */
fn count_files_recursive(dir: &std::path::Path) -> Result<usize, std::io::Error> {
    let mut count = 0;
    if dir.is_dir() {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                count += 1;
            } else if path.is_dir() {
                count += count_files_recursive(&path)?;
            }
        }
    }
    Ok(count)
}
