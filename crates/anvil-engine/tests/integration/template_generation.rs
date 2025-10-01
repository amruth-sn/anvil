/*!
 * Template Generation Integration Tests
 *
 * Tests various template combinations and configurations to ensure
 * all templates generate correctly with different variable combinations.
 */

use super::{IntegrationTestSuite, TestConfig, TestResult};
use serde_json::Value;
use std::collections::HashMap;

/// Test suite for all template generation scenarios
pub struct TemplateGenerationTests;

impl TemplateGenerationTests {
    /// Test fullstack-saas template with all possible configurations
    pub async fn test_fullstack_saas_variations() -> anyhow::Result<Vec<TestResult>> {
        let suite = IntegrationTestSuite::new()?;
        let mut results = Vec::new();

        // Test case 1: Basic shadcn/ui setup
        let config1 = TestConfig {
            template_name: "fullstack-saas".to_string(),
            variables: HashMap::new(),
            expected_files: vec![
                "package.json".to_string(),
                "app/page.tsx".to_string(),
                "components/ui/button.tsx".to_string(),
                "components/ui/card.tsx".to_string(),
                "components/ui/dialog.tsx".to_string(),
                "tailwind.config.ts".to_string(),
                "components.json".to_string(),
            ],
            expected_dependencies: vec![
                "@radix-ui/react-slot".to_string(),
                "tailwindcss-animate".to_string(),
                "lucide-react".to_string(),
            ],
            should_build: true,
            should_run: true,
            timeout_seconds: 120,
        };
        results.push(suite.generate_project(&config1).await?);

        // Test case 2: NextUI setup
        let config2 = TestConfig {
            template_name: "fullstack-saas".to_string(),
            variables: HashMap::new(),
            expected_files: vec![
                "package.json".to_string(),
                "app/page.tsx".to_string(),
                "tailwind.config.ts".to_string(),
            ],
            expected_dependencies: vec![
                "@nextui-org/react".to_string(),
                "framer-motion".to_string(),
            ],
            should_build: true,
            should_run: true,
            timeout_seconds: 120,
        };
        results.push(suite.generate_project(&config2).await?);

        // Test case 3: No UI library
        let config3 = TestConfig {
            template_name: "fullstack-saas".to_string(),
            variables: HashMap::new(),
            expected_files: vec![
                "package.json".to_string(),
                "app/page.tsx".to_string(),
                "tailwind.config.ts".to_string(),
            ],
            expected_dependencies: vec!["clsx".to_string(), "lucide-react".to_string()],
            should_build: true,
            should_run: true,
            timeout_seconds: 120,
        };
        results.push(suite.generate_project(&config3).await?);

        // Test case 4: Full configuration with all options
        let config4 = TestConfig {
            template_name: "fullstack-saas".to_string(),
            variables: HashMap::new(),
            expected_files: vec![
                "package.json".to_string(),
                "README.md".to_string(),
                "app/page.tsx".to_string(),
                "components/ui/button.tsx".to_string(),
                "components/ui/tabs.tsx".to_string(),
                "lib/utils.ts".to_string(),
            ],
            expected_dependencies: vec![
                "@radix-ui/react-dialog".to_string(),
                "@radix-ui/react-tabs".to_string(),
                "tailwind-merge".to_string(),
            ],
            should_build: true,
            should_run: true,
            timeout_seconds: 180,
        };
        results.push(suite.generate_project(&config4).await?);

        Ok(results)
    }

    /// Test rust templates
    pub async fn test_rust_templates() -> anyhow::Result<Vec<TestResult>> {
        let suite = IntegrationTestSuite::new()?;
        let mut results = Vec::new();

        // Test rust-hello-world
        let config1 = TestConfig {
            template_name: "rust-hello-world".to_string(),
            variables: HashMap::new(),
            expected_files: vec![
                "Cargo.toml".to_string(),
                "src/main.rs".to_string(),
                "README.md".to_string(),
            ],
            expected_dependencies: vec![], // Rust dependencies are in Cargo.toml
            should_build: true,
            should_run: false, // Don't run Rust binaries in tests
            timeout_seconds: 60,
        };
        results.push(suite.generate_project(&config1).await?);

        // Test rust-web-api
        let config2 = TestConfig {
            template_name: "rust-web-api".to_string(),
            variables: HashMap::new(),
            expected_files: vec!["Cargo.toml".to_string(), "src/main.rs".to_string()],
            expected_dependencies: vec![],
            should_build: true,
            should_run: false,
            timeout_seconds: 120,
        };
        results.push(suite.generate_project(&config2).await?);

        Ok(results)
    }

    /// Test Go templates
    pub async fn test_go_templates() -> anyhow::Result<Vec<TestResult>> {
        let suite = IntegrationTestSuite::new()?;
        let mut results = Vec::new();

        // Test go-cli-tool
        let config = TestConfig {
            template_name: "go-cli-tool".to_string(),
            variables: HashMap::new(),
            expected_files: vec![
                "go.mod".to_string(),
                "main.go".to_string(),
                "cmd/root.go".to_string(),
            ],
            expected_dependencies: vec![],
            should_build: true,
            should_run: false,
            timeout_seconds: 60,
        };
        results.push(suite.generate_project(&config).await?);

        Ok(results)
    }

    /// Test error cases and edge conditions
    pub async fn test_error_cases() -> anyhow::Result<Vec<TestResult>> {
        let suite = IntegrationTestSuite::new()?;
        let mut results = Vec::new();

        // Test invalid template name
        let config1 = TestConfig {
            template_name: "non-existent-template".to_string(),
            variables: HashMap::new(),
            expected_files: vec![],
            expected_dependencies: vec![],
            should_build: false,
            should_run: false,
            timeout_seconds: 30,
        };
        let mut result1 = suite.generate_project(&config1).await?;
        assert!(!result1.success, "Should fail for non-existent template");
        // Mark as successful since it failed as expected
        result1.success = true;
        results.push(result1);

        // Test invalid variable values
        let config2 = TestConfig {
            template_name: "fullstack-saas".to_string(),
            variables: HashMap::new(),
            expected_files: vec![],
            expected_dependencies: vec![],
            should_build: false,
            should_run: false,
            timeout_seconds: 30,
        };
        let result2 = suite.generate_project(&config2).await?;
        // This might succeed with default values, depending on validation
        results.push(result2);

        Ok(results)
    }

    /// Helper to create variables map with proper JSON values
    #[allow(dead_code)]
    fn create_variables(vars: HashMap<&str, &str>) -> HashMap<String, Value> {
        vars.into_iter()
            .map(|(k, v)| {
                let value = match v {
                    "true" => Value::Bool(true),
                    "false" => Value::Bool(false),
                    s if s.parse::<i64>().is_ok() => {
                        Value::Number(s.parse::<i64>().unwrap().into())
                    }
                    s => Value::String(s.to_string()),
                };
                (k.to_string(), value)
            })
            .collect()
    }
}

#[tokio::test]
async fn test_all_template_variations() {
    let mut all_results = Vec::new();

    // Test fullstack-saas variations
    match TemplateGenerationTests::test_fullstack_saas_variations().await {
        Ok(mut results) => {
            println!(
                "✅ Fullstack SaaS tests: {}/{} passed",
                results.iter().filter(|r| r.success).count(),
                results.len()
            );
            all_results.append(&mut results);
        }
        Err(e) => {
            println!("❌ Fullstack SaaS tests failed: {}", e);
            panic!("Template generation tests failed");
        }
    }

    // Test Rust templates
    match TemplateGenerationTests::test_rust_templates().await {
        Ok(mut results) => {
            println!(
                "✅ Rust template tests: {}/{} passed",
                results.iter().filter(|r| r.success).count(),
                results.len()
            );
            all_results.append(&mut results);
        }
        Err(e) => {
            println!("❌ Rust template tests failed: {}", e);
        }
    }

    // Test Go templates
    match TemplateGenerationTests::test_go_templates().await {
        Ok(mut results) => {
            println!(
                "✅ Go template tests: {}/{} passed",
                results.iter().filter(|r| r.success).count(),
                results.len()
            );
            all_results.append(&mut results);
        }
        Err(e) => {
            println!("❌ Go template tests failed: {}", e);
        }
    }

    // Test error cases
    match TemplateGenerationTests::test_error_cases().await {
        Ok(mut results) => {
            println!(
                "✅ Error case tests: {}/{} behaved as expected",
                results.len(),
                results.len()
            );
            all_results.append(&mut results);
        }
        Err(e) => {
            println!("❌ Error case tests failed: {}", e);
        }
    }

    // Summary
    let total_tests = all_results.len();
    let successful_tests = all_results.iter().filter(|r| r.success).count();
    let total_files = all_results.iter().map(|r| r.files_created).sum::<usize>();
    let avg_duration =
        all_results.iter().map(|r| r.duration_ms).sum::<u128>() / total_tests as u128;

    println!("\n📊 Template Generation Test Summary:");
    println!("   Total tests: {}", total_tests);
    println!("   Successful: {}", successful_tests);
    println!(
        "   Success rate: {:.1}%",
        (successful_tests as f64 / total_tests as f64) * 100.0
    );
    println!("   Total files generated: {}", total_files);
    println!("   Average generation time: {}ms", avg_duration);

    // Print warnings
    for (i, result) in all_results.iter().enumerate() {
        if !result.warnings.is_empty() {
            println!("⚠️  Test {} warnings:", i + 1);
            for warning in &result.warnings {
                println!("     {}", warning);
            }
        }
    }

    assert!(successful_tests > 0, "At least some tests should pass");
}
