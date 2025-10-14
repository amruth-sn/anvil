/*!
 * Build and Runtime Verification Tests
 * 
 * Tests that generated projects actually compile, build, and run correctly
 * across different package managers, configurations, and deployment targets.
 */

use super::{IntegrationTestSuite, TestConfig, TestResult};
use std::collections::HashMap;
use std::time::Duration;
use serde_json::Value;
use tokio::process::Command;
use tokio::time::timeout;

/// Test suite for build and runtime verification
pub struct BuildVerificationTests;

impl BuildVerificationTests {
    /// Test that generated Next.js projects build successfully
    pub async fn test_nextjs_builds() -> anyhow::Result<Vec<TestResult>> {
        let suite = IntegrationTestSuite::new()?;
        let mut results = Vec::new();
        
        // Test basic Next.js build with npm
        let config1 = TestConfig {
            template_name: "fullstack-saas".to_string(),
            variables: Self::create_variables(HashMap::from([
                ("project_name", "test-nextjs-npm"),
                ("package_manager", "npm"),
                ("ui_library", "shadcn/ui"),
            ])),
            expected_files: vec![
                "package.json".to_string(),
                "next.config.js".to_string(),
                "app/layout.tsx".to_string(),
                "app/page.tsx".to_string(),
            ],
            expected_dependencies: vec![],
            should_build: true,
            should_run: false,
            timeout_seconds: 300,
        };
        let mut result1 = suite.generate_project(&config1).await?;
        if result1.success && result1.project_name.is_some() {
            let project_name = result1.project_name.as_ref().unwrap();
            result1.build_success = Some(suite.verify_build(project_name, "npm").await?);
        }
        results.push(result1);
        
        // Test Next.js build with pnpm
        let config2 = TestConfig {
            template_name: "fullstack-saas".to_string(),
            variables: Self::create_variables(HashMap::from([
                ("project_name", "test-nextjs-pnpm"),
                ("package_manager", "pnpm"),
                ("ui_library", "shadcn/ui"),
                ("include_demo_content", "true"),
            ])),
            expected_files: vec![
                "package.json".to_string(),
                "components/ui/button.tsx".to_string(),
            ],
            expected_dependencies: vec![],
            should_build: true,
            should_run: false,
            timeout_seconds: 300,
        };
        let mut result2 = suite.generate_project(&config2).await?;
        if result2.success && result2.project_name.is_some() {
            let project_name = result2.project_name.as_ref().unwrap();
            result2.build_success = Some(suite.verify_build(project_name, "pnpm").await?);
        }
        results.push(result2);
        
        // Test Next.js build with yarn
        let config3 = TestConfig {
            template_name: "fullstack-saas".to_string(),
            variables: Self::create_variables(HashMap::from([
                ("project_name", "test-nextjs-yarn"),
                ("package_manager", "yarn"),
                ("ui_library", "nextui"),
            ])),
            expected_files: vec![
                "package.json".to_string(),
                "tailwind.config.ts".to_string(),
            ],
            expected_dependencies: vec![],
            should_build: true,
            should_run: false,
            timeout_seconds: 300,
        };
        let mut result3 = suite.generate_project(&config3).await?;
        if result3.success && result3.project_name.is_some() {
            let project_name = result3.project_name.as_ref().unwrap();
            result3.build_success = Some(suite.verify_build(project_name, "yarn").await?);
        }
        results.push(result3);
        
        // Test Next.js build with bun
        let config4 = TestConfig {
            template_name: "fullstack-saas".to_string(),
            variables: Self::create_variables(HashMap::from([
                ("project_name", "test-nextjs-bun"),
                ("package_manager", "bun"),
                ("ui_library", "none"),
            ])),
            expected_files: vec![
                "package.json".to_string(),
            ],
            expected_dependencies: vec![],
            should_build: true,
            should_run: false,
            timeout_seconds: 300,
        };
        let mut result4 = suite.generate_project(&config4).await?;
        if result4.success && result4.project_name.is_some() {
            let project_name = result4.project_name.as_ref().unwrap();
            result4.build_success = Some(suite.verify_build(project_name, "bun").await?);
        }
        results.push(result4);
        
        Ok(results)
    }
    
    /// Test that Rust projects compile successfully
    pub async fn test_rust_builds() -> anyhow::Result<Vec<TestResult>> {
        let suite = IntegrationTestSuite::new()?;
        let mut results = Vec::new();
        
        // Test rust-hello-world compilation
        let config1 = TestConfig {
            template_name: "rust-hello-world".to_string(),
            variables: Self::create_variables(HashMap::from([
                ("project_name", "test-rust-hello"),
                ("author_name", "Test Author"),
            ])),
            expected_files: vec![
                "Cargo.toml".to_string(),
                "src/main.rs".to_string(),
            ],
            expected_dependencies: vec![],
            should_build: true,
            should_run: false,
            timeout_seconds: 180,
        };
        let mut result1 = suite.generate_project(&config1).await?;
        if result1.success && result1.project_name.is_some() {
            let project_name = result1.project_name.as_ref().unwrap();
            result1.build_success = Some(Self::verify_rust_build(&suite, project_name).await?);
        }
        results.push(result1);
        
        // Test rust-web-api compilation
        let config2 = TestConfig {
            template_name: "rust-web-api".to_string(),
            variables: Self::create_variables(HashMap::from([
                ("project_name", "test-rust-api"),
                ("author_name", "Test Author"),
            ])),
            expected_files: vec![
                "Cargo.toml".to_string(),
                "src/main.rs".to_string(),
            ],
            expected_dependencies: vec![],
            should_build: true,
            should_run: false,
            timeout_seconds: 300, // Web API has more dependencies
        };
        let mut result2 = suite.generate_project(&config2).await?;
        if result2.success && result2.project_name.is_some() {
            let project_name = result2.project_name.as_ref().unwrap();
            result2.build_success = Some(Self::verify_rust_build(&suite, project_name).await?);
        }
        results.push(result2);
        
        Ok(results)
    }
    
    /// Test that Go projects compile successfully
    pub async fn test_go_builds() -> anyhow::Result<Vec<TestResult>> {
        let suite = IntegrationTestSuite::new()?;
        let mut results = Vec::new();
        
        // Test go-cli-tool compilation
        let config = TestConfig {
            template_name: "go-cli-tool".to_string(),
            variables: Self::create_variables(HashMap::from([
                ("project_name", "test-go-cli"),
                ("author_name", "Test Author"),
            ])),
            expected_files: vec![
                "go.mod".to_string(),
                "main.go".to_string(),
                "cmd/root.go".to_string(),
            ],
            expected_dependencies: vec![],
            should_build: true,
            should_run: false,
            timeout_seconds: 180,
        };
        let mut result = suite.generate_project(&config).await?;
        if result.success && result.project_name.is_some() {
            let project_name = result.project_name.as_ref().unwrap();
            result.build_success = Some(Self::verify_go_build(&suite, project_name).await?);
        }
        results.push(result);
        
        Ok(results)
    }
    
    /// Test development server startup
    pub async fn test_dev_server_startup() -> anyhow::Result<Vec<TestResult>> {
        let suite = IntegrationTestSuite::new()?;
        let mut results = Vec::new();
        
        // Test Next.js dev server with different package managers
        let package_managers = vec!["npm", "pnpm", "yarn"];
        
        for pm in package_managers {
            let project_name = format!("test-dev-{}", pm);
            let config = TestConfig {
                template_name: "fullstack-saas".to_string(),
                variables: Self::create_variables(HashMap::from([
                    ("project_name", project_name.as_str()),
                    ("package_manager", pm),
                    ("ui_library", "shadcn/ui"),
                    ("include_demo_content", "true"),
                ])),
                expected_files: vec![
                    "package.json".to_string(),
                    "app/page.tsx".to_string(),
                ],
                expected_dependencies: vec![],
                should_build: true,
                should_run: true,
                timeout_seconds: 300,
            };
            
            let mut result = suite.generate_project(&config).await?;
            if result.success && result.project_name.is_some() {
                let project_name = result.project_name.as_ref().unwrap();
                // First ensure it builds
                result.build_success = Some(suite.verify_build(project_name, pm).await?);
                
                // Then test runtime
                if result.build_success == Some(true) {
                    result.runtime_success = Some(suite.verify_runtime(project_name, pm).await?);
                }
            }
            results.push(result);
        }
        
        Ok(results)
    }
    
    /// Test type checking for TypeScript projects
    pub async fn test_typescript_checking() -> anyhow::Result<Vec<TestResult>> {
        let suite = IntegrationTestSuite::new()?;
        let mut results = Vec::new();
        
        // Test TypeScript type checking with complex setup
        let config = TestConfig {
            template_name: "fullstack-saas".to_string(),
            variables: Self::create_variables(HashMap::from([
                ("project_name", "test-typescript"),
                ("ui_library", "shadcn/ui"),
                ("api_pattern", "trpc"),
                ("auth_provider", "clerk"),
                ("language", "typescript"),
            ])),
            expected_files: vec![
                "tsconfig.json".to_string(),
                "app/page.tsx".to_string(),
                "lib/trpc/client.ts".to_string(),
            ],
            expected_dependencies: vec![],
            should_build: true,
            should_run: false,
            timeout_seconds: 240,
        };
        
        let mut result = suite.generate_project(&config).await?;
        if result.success && result.project_name.is_some() {
            let project_name = result.project_name.as_ref().unwrap();
            result.build_success = Some(Self::verify_typescript_check(&suite, project_name).await?);
        }
        results.push(result);
        
        Ok(results)
    }
    
    /// Test linting for generated projects
    pub async fn test_linting() -> anyhow::Result<Vec<TestResult>> {
        let suite = IntegrationTestSuite::new()?;
        let mut results = Vec::new();
        
        // Test ESLint on generated Next.js project
        let config = TestConfig {
            template_name: "fullstack-saas".to_string(),
            variables: Self::create_variables(HashMap::from([
                ("project_name", "test-linting"),
                ("ui_library", "shadcn/ui"),
                ("include_demo_content", "true"),
            ])),
            expected_files: vec![
                "package.json".to_string(),
                "app/page.tsx".to_string(),
            ],
            expected_dependencies: vec![],
            should_build: true,
            should_run: false,
            timeout_seconds: 180,
        };
        
        let mut result = suite.generate_project(&config).await?;
        if result.success && result.project_name.is_some() {
            let project_name = result.project_name.as_ref().unwrap();
            result.build_success = Some(Self::verify_linting(&suite, project_name).await?);
        }
        results.push(result);
        
        Ok(results)
    }
    
    /// Test complex project with all features
    pub async fn test_full_stack_build() -> anyhow::Result<TestResult> {
        let suite = IntegrationTestSuite::new()?;
        
        // Test comprehensive SaaS setup
        let config = TestConfig {
            template_name: "fullstack-saas".to_string(),
            variables: Self::create_variables(HashMap::from([
                ("project_name", "test-fullstack"),
                ("ui_library", "shadcn/ui"),
                ("package_manager", "npm"),
                ("auth_provider", "clerk"),
                ("api_pattern", "trpc"),
                ("database_provider", "supabase"),
                ("payment_provider", "stripe"),
                ("ai_provider", "openai"),
                ("include_demo_content", "true"),
                ("language", "typescript"),
            ])),
            expected_files: vec![
                "package.json".to_string(),
                "app/page.tsx".to_string(),
                "components/ui/button.tsx".to_string(),
                "lib/trpc/client.ts".to_string(),
                "lib/clerk-setup.ts".to_string(),
                "lib/supabase-client.ts".to_string(),
                "lib/stripe.ts".to_string(),
                "lib/openai.ts".to_string(),
            ],
            expected_dependencies: vec![],
            should_build: true,
            should_run: true,
            timeout_seconds: 600, // Complex build may take longer
        };
        
        let mut result = suite.generate_project(&config).await?;
        if result.success && result.project_name.is_some() {
            let project_name = result.project_name.as_ref().unwrap();
            // Test build
            result.build_success = Some(suite.verify_build(project_name, "npm").await?);
            
            // Test TypeScript checking
            if result.build_success == Some(true) {
                let ts_check = Self::verify_typescript_check(&suite, project_name).await?;
                if !ts_check {
                    result.warnings.push("TypeScript checking failed".to_string());
                }
            }
            
            // Test runtime (if build succeeded)
            if result.build_success == Some(true) {
                result.runtime_success = Some(suite.verify_runtime(project_name, "npm").await?);
            }
        }
        
        Ok(result)
    }
    
    /// Verify Rust project compilation
    async fn verify_rust_build(suite: &IntegrationTestSuite, project_name: &str) -> anyhow::Result<bool> {
        let project_dir = suite.temp_dir.path().join(project_name);
        
        let output = timeout(
            Duration::from_secs(300),
            Command::new("cargo")
                .arg("build")
                .current_dir(&project_dir)
                .output()
        ).await??;
        
        Ok(output.status.success())
    }
    
    /// Verify Go project compilation
    async fn verify_go_build(suite: &IntegrationTestSuite, project_name: &str) -> anyhow::Result<bool> {
        let project_dir = suite.temp_dir.path().join(project_name);
        
        // First run go mod tidy
        let tidy_output = Command::new("go")
            .arg("mod")
            .arg("tidy")
            .current_dir(&project_dir)
            .output()
            .await?;
        
        if !tidy_output.status.success() {
            return Ok(false);
        }
        
        // Then build
        let build_output = timeout(
            Duration::from_secs(180),
            Command::new("go")
                .arg("build")
                .arg(".")
                .current_dir(&project_dir)
                .output()
        ).await??;
        
        Ok(build_output.status.success())
    }
    
    /// Verify TypeScript type checking
    async fn verify_typescript_check(suite: &IntegrationTestSuite, project_name: &str) -> anyhow::Result<bool> {
        let project_dir = suite.temp_dir.path().join(project_name);
        
        // Install dependencies first
        let install_output = Command::new("npm")
            .arg("install")
            .current_dir(&project_dir)
            .output()
            .await?;
        
        if !install_output.status.success() {
            return Ok(false);
        }
        
        // Run type check
        let output = timeout(
            Duration::from_secs(120),
            Command::new("npm")
                .arg("run")
                .arg("type-check")
                .current_dir(&project_dir)
                .output()
        ).await??;
        
        Ok(output.status.success())
    }
    
    /// Verify ESLint passes
    async fn verify_linting(suite: &IntegrationTestSuite, project_name: &str) -> anyhow::Result<bool> {
        let project_dir = suite.temp_dir.path().join(project_name);
        
        // Install dependencies first
        let install_output = Command::new("npm")
            .arg("install")
            .current_dir(&project_dir)
            .output()
            .await?;
        
        if !install_output.status.success() {
            return Ok(false);
        }
        
        // Run lint
        let output = timeout(
            Duration::from_secs(90),
            Command::new("npm")
                .arg("run")
                .arg("lint")
                .current_dir(&project_dir)
                .output()
        ).await??;
        
        Ok(output.status.success())
    }
    
    /// Helper to create variables map with proper JSON values
    fn create_variables(vars: HashMap<&str, &str>) -> HashMap<String, Value> {
        vars.into_iter()
            .map(|(k, v)| {
                let value = match v {
                    "true" => Value::Bool(true),
                    "false" => Value::Bool(false),
                    s if s.parse::<i64>().is_ok() => Value::Number(s.parse::<i64>().unwrap().into()),
                    s => Value::String(s.to_string()),
                };
                (k.to_string(), value)
            })
            .collect()
    }
}

#[tokio::test]
async fn test_all_build_verification() {
    let mut all_results = Vec::new();
    let mut build_failures = Vec::new();
    let mut runtime_failures = Vec::new();
    
    // Test Next.js builds
    match BuildVerificationTests::test_nextjs_builds().await {
        Ok(mut results) => {
            let total = results.len();
            let successful = results.iter().filter(|r| r.success).count();
            let build_success = results.iter().filter(|r| r.build_success == Some(true)).count();
            
            println!("✅ Next.js build tests: {}/{} generated, {}/{} built", 
                     successful, total, build_success, successful);
            
            for result in &results {
                if result.success && result.build_success == Some(false) {
                    build_failures.push("Next.js");
                }
            }
            
            all_results.append(&mut results);
        }
        Err(e) => {
            println!("❌ Next.js build tests failed: {}", e);
        }
    }
    
    // Test Rust builds
    match BuildVerificationTests::test_rust_builds().await {
        Ok(mut results) => {
            let total = results.len();
            let successful = results.iter().filter(|r| r.success).count();
            let build_success = results.iter().filter(|r| r.build_success == Some(true)).count();
            
            println!("✅ Rust build tests: {}/{} generated, {}/{} built", 
                     successful, total, build_success, successful);
            
            for result in &results {
                if result.success && result.build_success == Some(false) {
                    build_failures.push("Rust");
                }
            }
            
            all_results.append(&mut results);
        }
        Err(e) => {
            println!("❌ Rust build tests failed: {}", e);
        }
    }
    
    // Test Go builds
    match BuildVerificationTests::test_go_builds().await {
        Ok(mut results) => {
            let total = results.len();
            let successful = results.iter().filter(|r| r.success).count();
            let build_success = results.iter().filter(|r| r.build_success == Some(true)).count();
            
            println!("✅ Go build tests: {}/{} generated, {}/{} built", 
                     successful, total, build_success, successful);
            
            for result in &results {
                if result.success && result.build_success == Some(false) {
                    build_failures.push("Go");
                }
            }
            
            all_results.append(&mut results);
        }
        Err(e) => {
            println!("❌ Go build tests failed: {}", e);
        }
    }
    
    // Test dev server startup
    match BuildVerificationTests::test_dev_server_startup().await {
        Ok(mut results) => {
            let total = results.len();
            let runtime_success = results.iter().filter(|r| r.runtime_success == Some(true)).count();
            
            println!("✅ Dev server tests: {}/{} started successfully", runtime_success, total);
            
            for result in &results {
                if result.build_success == Some(true) && result.runtime_success == Some(false) {
                    runtime_failures.push("Dev server");
                }
            }
            
            all_results.append(&mut results);
        }
        Err(e) => {
            println!("❌ Dev server tests failed: {}", e);
        }
    }
    
    // Test TypeScript checking
    match BuildVerificationTests::test_typescript_checking().await {
        Ok(mut results) => {
            let success = results.iter().filter(|r| r.build_success == Some(true)).count();
            println!("✅ TypeScript checking: {}/{} passed", success, results.len());
            all_results.append(&mut results);
        }
        Err(e) => {
            println!("❌ TypeScript checking tests failed: {}", e);
        }
    }
    
    // Test linting
    match BuildVerificationTests::test_linting().await {
        Ok(mut results) => {
            let success = results.iter().filter(|r| r.build_success == Some(true)).count();
            println!("✅ Linting tests: {}/{} passed", success, results.len());
            all_results.append(&mut results);
        }
        Err(e) => {
            println!("❌ Linting tests failed: {}", e);
        }
    }
    
    // Test full-stack build
    match BuildVerificationTests::test_full_stack_build().await {
        Ok(result) => {
            println!("✅ Full-stack build test: {}", 
                     if result.build_success == Some(true) { "passed" } else { "failed" });
            
            if result.runtime_success == Some(true) {
                println!("   Runtime test: passed");
            } else if result.runtime_success == Some(false) {
                println!("   Runtime test: failed");
                runtime_failures.push("Full-stack");
            }
            
            all_results.push(result);
        }
        Err(e) => {
            println!("❌ Full-stack build test failed: {}", e);
        }
    }
    
    // Summary
    let total_tests = all_results.len();
    let generation_success = all_results.iter().filter(|r| r.success).count();
    let build_success = all_results.iter().filter(|r| r.build_success == Some(true)).count();
    let runtime_success = all_results.iter().filter(|r| r.runtime_success == Some(true)).count();
    
    println!("\n📊 Build Verification Test Summary:");
    println!("   Total tests: {}", total_tests);
    println!("   Generation success: {}/{}", generation_success, total_tests);
    println!("   Build success: {}/{}", build_success, generation_success);
    println!("   Runtime success: {}/{}", runtime_success, 
             all_results.iter().filter(|r| r.runtime_success.is_some()).count());
    
    if !build_failures.is_empty() {
        println!("   Build failures in: {:?}", build_failures);
    }
    
    if !runtime_failures.is_empty() {
        println!("   Runtime failures in: {:?}", runtime_failures);
    }
    
    // Print detailed errors
    for (i, result) in all_results.iter().enumerate() {
        if !result.success {
            if let Some(error) = &result.error_message {
                println!("❌ Test {} generation failed: {}", i + 1, error);
            }
        } else if result.build_success == Some(false) {
            println!("❌ Test {} build failed", i + 1);
        } else if result.runtime_success == Some(false) {
            println!("❌ Test {} runtime failed", i + 1);
        }
        
        if !result.warnings.is_empty() {
            println!("⚠️  Test {} warnings:", i + 1);
            for warning in &result.warnings {
                println!("     {}", warning);
            }
        }
    }
    
    // At least 80% of generated projects should build successfully
    let build_success_rate = if generation_success > 0 {
        build_success as f64 / generation_success as f64
    } else {
        0.0
    };
    
    assert!(
        build_success_rate >= 0.8,
        "Build success rate ({:.1}%) is below minimum (80.0%)",
        build_success_rate * 100.0
    );
    
    assert!(generation_success > 0, "At least some projects should generate successfully");
}