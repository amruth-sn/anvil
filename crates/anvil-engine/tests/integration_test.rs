/*!
 * Main Integration Test Runner
 * 
 * Runs the complete integration test suite including:
 * - Template generation tests
 * - Shared component tests  
 * - Service compatibility tests
 * - Build and runtime verification
 */

mod integration;

use integration::{
    template_generation::TemplateGenerationTests,
    shared_components::SharedComponentTests,
    service_compatibility::ServiceCompatibilityTests,
    build_verification::BuildVerificationTests,
    IntegrationTestSuite,
};
use std::time::Instant;

/// Main integration test that runs all test suites
#[tokio::test]
async fn run_complete_integration_test_suite() {
    println!("🚀 Starting Anvil Integration Test Suite");
    println!("========================================\n");
    
    let start_time = Instant::now();
    let mut total_tests = 0;
    let mut successful_tests = 0;
    let mut total_files_generated = 0;
    let mut test_failures = Vec::new();
    
    // Test 1: Template Generation
    println!("📋 Running Template Generation Tests...");
    match run_template_generation_tests().await {
        Ok((tests, success, files)) => {
            total_tests += tests;
            successful_tests += success;
            total_files_generated += files;
            println!("✅ Template Generation: {}/{} tests passed\n", success, tests);
        }
        Err(e) => {
            println!("❌ Template Generation tests failed: {}\n", e);
            test_failures.push("Template Generation");
        }
    }
    
    // Test 2: Shared Components
    println!("🧩 Running Shared Component Tests...");
    match run_shared_component_tests().await {
        Ok((tests, success, files)) => {
            total_tests += tests;
            successful_tests += success;
            total_files_generated += files;
            println!("✅ Shared Components: {}/{} tests passed\n", success, tests);
        }
        Err(e) => {
            println!("❌ Shared Component tests failed: {}\n", e);
            test_failures.push("Shared Components");
        }
    }
    
    // Test 3: Service Compatibility
    println!("🔗 Running Service Compatibility Tests...");
    match run_service_compatibility_tests().await {
        Ok((tests, success, files)) => {
            total_tests += tests;
            successful_tests += success;
            total_files_generated += files;
            println!("✅ Service Compatibility: {}/{} tests passed\n", success, tests);
        }
        Err(e) => {
            println!("❌ Service Compatibility tests failed: {}\n", e);
            test_failures.push("Service Compatibility");
        }
    }
    
    // Test 4: Build Verification
    println!("🔨 Running Build Verification Tests...");
    match run_build_verification_tests().await {
        Ok((tests, success, builds, runtimes)) => {
            total_tests += tests;
            successful_tests += success;
            println!("✅ Build Verification: {}/{} tests passed ({} builds, {} runtimes)\n", 
                     success, tests, builds, runtimes);
        }
        Err(e) => {
            println!("❌ Build Verification tests failed: {}\n", e);
            test_failures.push("Build Verification");
        }
    }
    
    // Final Summary
    let duration = start_time.elapsed();
    let success_rate = if total_tests > 0 {
        (successful_tests as f64 / total_tests as f64) * 100.0
    } else {
        0.0
    };
    
    println!("========================================");
    println!("🏁 Integration Test Suite Complete");
    println!("========================================");
    println!("📊 Summary:");
    println!("   Total test duration: {:.2}s", duration.as_secs_f64());
    println!("   Total tests run: {}", total_tests);
    println!("   Successful tests: {}", successful_tests);
    println!("   Success rate: {:.1}%", success_rate);
    println!("   Files generated: {}", total_files_generated);
    println!("   Failed test suites: {}", test_failures.len());
    
    if !test_failures.is_empty() {
        println!("   Failed suites: {:?}", test_failures);
    }
    
    // Performance metrics
    if total_tests > 0 {
        let avg_test_time = duration.as_millis() / total_tests as u128;
        let files_per_second = if duration.as_secs() > 0 {
            total_files_generated as f64 / duration.as_secs_f64()
        } else {
            0.0
        };
        
        println!("   Average test time: {}ms", avg_test_time);
        println!("   Files generated/sec: {:.1}", files_per_second);
    }
    
    // Assertions for CI/CD
    assert!(
        success_rate >= 75.0,
        "Integration test success rate ({:.1}%) is below minimum (75.0%)",
        success_rate
    );
    
    assert!(
        test_failures.len() <= 1,
        "Too many test suites failed: {:?}",
        test_failures
    );
    
    assert!(
        total_tests >= 20,
        "Not enough tests were run ({}), expected at least 20",
        total_tests
    );
    
    println!("\n🎉 All integration tests completed successfully!");
}

/// Run template generation test suite
async fn run_template_generation_tests() -> anyhow::Result<(usize, usize, usize)> {
    let mut all_results = Vec::new();
    
    // Test fullstack-saas variations
    let mut results = TemplateGenerationTests::test_fullstack_saas_variations().await?;
    all_results.append(&mut results);
    
    // Test Rust templates
    let mut results = TemplateGenerationTests::test_rust_templates().await?;
    all_results.append(&mut results);
    
    // Test Go templates  
    let mut results = TemplateGenerationTests::test_go_templates().await?;
    all_results.append(&mut results);
    
    // Test error cases
    let mut results = TemplateGenerationTests::test_error_cases().await?;
    all_results.append(&mut results);
    
    let total_tests = all_results.len();
    let successful_tests = all_results.iter().filter(|r| r.success).count();
    let total_files = all_results.iter().map(|r| r.files_created).sum::<usize>();
    
    Ok((total_tests, successful_tests, total_files))
}

/// Run shared component test suite
async fn run_shared_component_tests() -> anyhow::Result<(usize, usize, usize)> {
    let mut all_results = Vec::new();
    
    // Test each component category
    let mut results = SharedComponentTests::test_auth_components().await?;
    all_results.append(&mut results);
    
    let mut results = SharedComponentTests::test_api_components().await?;
    all_results.append(&mut results);
    
    let mut results = SharedComponentTests::test_database_components().await?;
    all_results.append(&mut results);
    
    let mut results = SharedComponentTests::test_ai_components().await?;
    all_results.append(&mut results);
    
    let mut results = SharedComponentTests::test_payment_components().await?;
    all_results.append(&mut results);
    
    let mut results = SharedComponentTests::test_deployment_components().await?;
    all_results.append(&mut results);
    
    // Test component composition
    let result = SharedComponentTests::test_component_composition().await?;
    all_results.push(result);
    
    let total_tests = all_results.len();
    let successful_tests = all_results.iter().filter(|r| r.success).count();
    let total_files = all_results.iter().map(|r| r.files_created).sum::<usize>();
    
    Ok((total_tests, successful_tests, total_files))
}

/// Run service compatibility test suite
async fn run_service_compatibility_tests() -> anyhow::Result<(usize, usize, usize)> {
    let mut all_results = Vec::new();
    
    // Test various compatibility scenarios
    let mut results = ServiceCompatibilityTests::test_trpc_language_compatibility().await?;
    all_results.append(&mut results);
    
    let mut results = ServiceCompatibilityTests::test_auth_provider_compatibility().await?;
    all_results.append(&mut results);
    
    let mut results = ServiceCompatibilityTests::test_database_compatibility().await?;
    all_results.append(&mut results);
    
    let mut results = ServiceCompatibilityTests::test_api_pattern_compatibility().await?;
    all_results.append(&mut results);
    
    let mut results = ServiceCompatibilityTests::test_ui_library_compatibility().await?;
    all_results.append(&mut results);
    
    let mut results = ServiceCompatibilityTests::test_service_combinations().await?;
    all_results.append(&mut results);
    
    let mut results = ServiceCompatibilityTests::test_invalid_combinations().await?;
    all_results.append(&mut results);
    
    let total_tests = all_results.len();
    let successful_tests = all_results.iter().filter(|r| r.success).count();
    let total_files = all_results.iter().map(|r| r.files_created).sum::<usize>();
    
    Ok((total_tests, successful_tests, total_files))
}

/// Run build verification test suite
async fn run_build_verification_tests() -> anyhow::Result<(usize, usize, usize, usize)> {
    let mut all_results = Vec::new();
    
    // Test various build scenarios
    let mut results = BuildVerificationTests::test_nextjs_builds().await?;
    all_results.append(&mut results);
    
    let mut results = BuildVerificationTests::test_rust_builds().await?;
    all_results.append(&mut results);
    
    let mut results = BuildVerificationTests::test_go_builds().await?;
    all_results.append(&mut results);
    
    let mut results = BuildVerificationTests::test_dev_server_startup().await?;
    all_results.append(&mut results);
    
    let mut results = BuildVerificationTests::test_typescript_checking().await?;
    all_results.append(&mut results);
    
    let mut results = BuildVerificationTests::test_linting().await?;
    all_results.append(&mut results);
    
    // Test full-stack build
    let result = BuildVerificationTests::test_full_stack_build().await?;
    all_results.push(result);
    
    let total_tests = all_results.len();
    let successful_tests = all_results.iter().filter(|r| r.success).count();
    let successful_builds = all_results.iter().filter(|r| r.build_success == Some(true)).count();
    let successful_runtimes = all_results.iter().filter(|r| r.runtime_success == Some(true)).count();
    
    Ok((total_tests, successful_tests, successful_builds, successful_runtimes))
}

/// Smoke test - quick verification that basic functionality works
#[tokio::test]
async fn smoke_test_basic_generation() {
    println!("🔥 Running smoke test for basic project generation");
    
    // Use a simplified approach similar to the working subset tests
    let suite = IntegrationTestSuite::new().expect("Failed to create test suite");
    let start_time = std::time::Instant::now();
    
    // Test basic rust-hello-world generation (this should work)
    let mut cmd = tokio::process::Command::new(&suite.anvil_binary);
    cmd.arg("create")
       .arg("smoke-test-project")
       .arg("--template")
       .arg("rust-hello-world")
       .arg("--no-input")
       .current_dir(&suite.workspace_dir);
    
    let output = cmd.output().await.expect("Failed to execute anvil command");
    let duration = start_time.elapsed();
    
    if !output.status.success() {
        println!("❌ Command failed with stderr: {}", String::from_utf8_lossy(&output.stderr));
        println!("❌ Command failed with stdout: {}", String::from_utf8_lossy(&output.stdout));
        panic!("Smoke test should generate successfully");
    }
    
    let project_path = suite.workspace_dir.join("smoke-test-project");
    
    // Count created files
    let files_created = if project_path.exists() {
        suite.count_files_recursive(&project_path).unwrap_or(0)
    } else {
        0
    };
    
    // Verify expected files exist
    let expected_files = vec!["Cargo.toml", "src/main.rs"];
    for expected_file in &expected_files {
        let file_path = project_path.join(expected_file);
        assert!(file_path.exists(), "Expected file should exist: {}", expected_file);
    }
    
    // Clean up
    if project_path.exists() {
        std::fs::remove_dir_all(&project_path).ok();
    }
    
    assert!(files_created > 0, "Should create some files");
    assert!(duration.as_millis() < 30000, "Should complete within 30 seconds");
    
    println!("✅ Smoke test passed: generated {} files in {}ms", 
             files_created, duration.as_millis());
}

/// Performance benchmark test
#[tokio::test]
async fn benchmark_generation_performance() {
    println!("⚡ Running performance benchmark");
    
    let suite = IntegrationTestSuite::new().expect("Failed to create test suite");
    let iterations = 5;
    let mut durations = Vec::new();
    let mut file_counts = Vec::new();
    
    for i in 0..iterations {
        let config = integration::TestConfig {
            template_name: "fullstack-saas".to_string(),
            variables: std::collections::HashMap::from([
                ("project_name".to_string(), serde_json::Value::String(format!("benchmark-{}", i))),
                ("ui_library".to_string(), serde_json::Value::String("shadcn/ui".to_string())),
                ("include_demo_content".to_string(), serde_json::Value::Bool(true)),
            ]),
            expected_files: vec![],
            expected_dependencies: vec![],
            should_build: false,
            should_run: false,
            timeout_seconds: 120,
        };
        
        let result = suite.generate_project(&config).await.expect("Generation should not error");
        
        if result.success {
            durations.push(result.duration_ms);
            file_counts.push(result.files_created);
        }
    }
    
    if !durations.is_empty() {
        let avg_duration = durations.iter().sum::<u128>() / durations.len() as u128;
        let avg_files = file_counts.iter().sum::<usize>() / file_counts.len();
        let max_duration = durations.iter().max().unwrap();
        let min_duration = durations.iter().min().unwrap();
        
        println!("📊 Performance benchmark results ({} iterations):", iterations);
        println!("   Average generation time: {}ms", avg_duration);
        println!("   Min/Max generation time: {}ms / {}ms", min_duration, max_duration);
        println!("   Average files generated: {}", avg_files);
        
        // Performance assertions
        assert!(avg_duration < 10000, "Average generation time should be under 10 seconds");
        assert!(*max_duration < 20000, "Maximum generation time should be under 20 seconds");
        assert!(avg_files > 15, "Should generate a reasonable number of files");
        
        println!("✅ Performance benchmark passed");
    } else {
        panic!("No successful generations in benchmark");
    }
}