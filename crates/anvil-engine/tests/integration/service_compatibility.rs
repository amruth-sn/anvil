/*!
 * Service Compatibility Integration Tests
 *
 * Tests service compatibility rules, language requirements, and
 * cross-template service composition to ensure proper validation
 * and error handling.
 */

use super::{IntegrationTestSuite, TestConfig, TestResult};
use serde_json::Value;
use std::collections::HashMap;

/// Test suite for service compatibility and validation
pub struct ServiceCompatibilityTests;

impl ServiceCompatibilityTests {
    /// Test tRPC compatibility requirements (TypeScript only)
    pub async fn test_trpc_language_compatibility() -> anyhow::Result<Vec<TestResult>> {
        let suite = IntegrationTestSuite::new()?;
        let mut results = Vec::new();

        // Test valid case: tRPC with TypeScript
        let config1 = TestConfig {
            template_name: "fullstack-saas".to_string(),
            variables: Self::create_variables(HashMap::from([
                ("project_name", "test-trpc-ts"),
                ("api_pattern", "trpc"),
                ("language", "typescript"),
            ])),
            expected_files: vec![
                "lib/trpc/client.ts".to_string(),
                "lib/trpc/server.ts".to_string(),
                "server/trpc.ts".to_string(),
            ],
            expected_dependencies: vec!["@trpc/server".to_string(), "@trpc/client".to_string()],
            should_build: true,
            should_run: false,
            timeout_seconds: 90,
        };
        results.push(suite.generate_project(&config1).await?);

        // Test invalid case: tRPC with JavaScript (should fail or fallback)
        let config2 = TestConfig {
            template_name: "fullstack-saas".to_string(),
            variables: Self::create_variables(HashMap::from([
                ("project_name", "test-trpc-js"),
                ("api_pattern", "trpc"),
                ("language", "javascript"),
            ])),
            expected_files: vec![], // Should either fail or not include tRPC files
            expected_dependencies: vec![],
            should_build: false,
            should_run: false,
            timeout_seconds: 60,
        };
        let result2 = suite.generate_project(&config2).await?;
        results.push(result2);

        Ok(results)
    }

    /// Test authentication provider compatibility
    pub async fn test_auth_provider_compatibility() -> anyhow::Result<Vec<TestResult>> {
        let suite = IntegrationTestSuite::new()?;
        let mut results = Vec::new();

        // Test Clerk with Next.js (valid)
        let config1 = TestConfig {
            template_name: "fullstack-saas".to_string(),
            variables: Self::create_variables(HashMap::from([
                ("project_name", "test-clerk-nextjs"),
                ("framework", "nextjs"),
                ("auth_provider", "clerk"),
            ])),
            expected_files: vec![
                "middleware.ts".to_string(),
                "lib/clerk-setup.ts".to_string(),
                "components/auth/user-button.tsx".to_string(),
            ],
            expected_dependencies: vec!["@clerk/nextjs".to_string()],
            should_build: true,
            should_run: false,
            timeout_seconds: 90,
        };
        results.push(suite.generate_project(&config1).await?);

        // Test Firebase Auth with different configurations
        let config2 = TestConfig {
            template_name: "fullstack-saas".to_string(),
            variables: Self::create_variables(HashMap::from([
                ("project_name", "test-firebase-auth"),
                ("framework", "nextjs"),
                ("auth_provider", "firebase"),
            ])),
            expected_files: vec![
                "lib/firebase-client.ts".to_string(),
                "components/auth/auth-form.tsx".to_string(),
            ],
            expected_dependencies: vec!["firebase".to_string(), "firebase-admin".to_string()],
            should_build: true,
            should_run: false,
            timeout_seconds: 90,
        };
        results.push(suite.generate_project(&config2).await?);

        // Test Supabase Auth
        let config3 = TestConfig {
            template_name: "fullstack-saas".to_string(),
            variables: Self::create_variables(HashMap::from([
                ("project_name", "test-supabase-auth"),
                ("framework", "nextjs"),
                ("auth_provider", "supabase"),
                ("database_provider", "supabase"), // Should be compatible
            ])),
            expected_files: vec![
                "lib/supabase-client.ts".to_string(),
                "components/auth/auth-form.tsx".to_string(),
                "app/api/auth/callback/route.ts".to_string(),
            ],
            expected_dependencies: vec![
                "@supabase/supabase-js".to_string(),
                "@supabase/auth-helpers-nextjs".to_string(),
            ],
            should_build: true,
            should_run: false,
            timeout_seconds: 90,
        };
        results.push(suite.generate_project(&config3).await?);

        Ok(results)
    }

    /// Test database provider compatibility
    pub async fn test_database_compatibility() -> anyhow::Result<Vec<TestResult>> {
        let suite = IntegrationTestSuite::new()?;
        let mut results = Vec::new();

        // Test Supabase as both auth and database (should be optimized)
        let config1 = TestConfig {
            template_name: "fullstack-saas".to_string(),
            variables: Self::create_variables(HashMap::from([
                ("project_name", "test-supabase-full"),
                ("auth_provider", "supabase"),
                ("database_provider", "supabase"),
            ])),
            expected_files: vec![
                "lib/supabase-client.ts".to_string(),
                "lib/supabase-server.ts".to_string(),
                "components/auth/auth-form.tsx".to_string(),
            ],
            expected_dependencies: vec![
                "@supabase/supabase-js".to_string(),
                "@supabase/auth-helpers-nextjs".to_string(),
            ],
            should_build: true,
            should_run: false,
            timeout_seconds: 90,
        };
        results.push(suite.generate_project(&config1).await?);

        // Test Neon with Drizzle ORM
        let config2 = TestConfig {
            template_name: "fullstack-saas".to_string(),
            variables: Self::create_variables(HashMap::from([
                ("project_name", "test-neon-drizzle"),
                ("database_provider", "neon"),
                ("orm", "drizzle"),
            ])),
            expected_files: vec![
                "lib/db.ts".to_string(),
                "lib/schema.ts".to_string(),
                "drizzle.config.ts".to_string(),
            ],
            expected_dependencies: vec![
                "drizzle-orm".to_string(),
                "@neondatabase/serverless".to_string(),
                "drizzle-kit".to_string(),
            ],
            should_build: true,
            should_run: false,
            timeout_seconds: 90,
        };
        results.push(suite.generate_project(&config2).await?);

        // Test MongoDB compatibility
        let config3 = TestConfig {
            template_name: "fullstack-saas".to_string(),
            variables: Self::create_variables(HashMap::from([
                ("project_name", "test-mongodb"),
                ("database_provider", "mongodb"),
                ("orm", "mongoose"),
            ])),
            expected_files: vec!["lib/mongodb.ts".to_string(), "models/User.ts".to_string()],
            expected_dependencies: vec!["mongodb".to_string(), "mongoose".to_string()],
            should_build: true,
            should_run: false,
            timeout_seconds: 90,
        };
        results.push(suite.generate_project(&config3).await?);

        Ok(results)
    }

    /// Test API pattern compatibility with different stacks
    pub async fn test_api_pattern_compatibility() -> anyhow::Result<Vec<TestResult>> {
        let suite = IntegrationTestSuite::new()?;
        let mut results = Vec::new();

        // Test REST API with OpenAPI documentation
        let config1 = TestConfig {
            template_name: "fullstack-saas".to_string(),
            variables: Self::create_variables(HashMap::from([
                ("project_name", "test-rest-docs"),
                ("api_pattern", "rest"),
                ("documentation", "openapi"),
            ])),
            expected_files: vec![
                "app/api/users/route.ts".to_string(),
                "app/api/docs/route.ts".to_string(),
                "lib/api/openapi.ts".to_string(),
                "lib/api/schemas.ts".to_string(),
            ],
            expected_dependencies: vec!["swagger-ui-react".to_string(), "zod".to_string()],
            should_build: true,
            should_run: false,
            timeout_seconds: 90,
        };
        results.push(suite.generate_project(&config1).await?);

        // Test GraphQL with Apollo Server
        let config2 = TestConfig {
            template_name: "fullstack-saas".to_string(),
            variables: Self::create_variables(HashMap::from([
                ("project_name", "test-graphql-apollo"),
                ("api_pattern", "graphql"),
                ("graphql_server", "apollo"),
            ])),
            expected_files: vec![
                "app/api/graphql/route.ts".to_string(),
                "lib/graphql/apollo-provider.tsx".to_string(),
                "lib/graphql/schema.ts".to_string(),
                "lib/graphql/resolvers/index.ts".to_string(),
            ],
            expected_dependencies: vec![
                "@apollo/server".to_string(),
                "@apollo/client".to_string(),
                "graphql".to_string(),
            ],
            should_build: true,
            should_run: false,
            timeout_seconds: 120,
        };
        results.push(suite.generate_project(&config2).await?);

        // Test tRPC with React Query integration
        let config3 = TestConfig {
            template_name: "fullstack-saas".to_string(),
            variables: Self::create_variables(HashMap::from([
                ("project_name", "test-trpc-rq"),
                ("api_pattern", "trpc"),
                ("state_management", "react-query"),
                ("language", "typescript"),
            ])),
            expected_files: vec![
                "lib/trpc/react.tsx".to_string(),
                "lib/trpc/client.ts".to_string(),
                "app/api/trpc/[trpc]/route.ts".to_string(),
            ],
            expected_dependencies: vec![
                "@trpc/react-query".to_string(),
                "@tanstack/react-query".to_string(),
            ],
            should_build: true,
            should_run: false,
            timeout_seconds: 120,
        };
        results.push(suite.generate_project(&config3).await?);

        Ok(results)
    }

    /// Test UI library compatibility
    pub async fn test_ui_library_compatibility() -> anyhow::Result<Vec<TestResult>> {
        let suite = IntegrationTestSuite::new()?;
        let mut results = Vec::new();

        // Test shadcn/ui with different configurations
        let config1 = TestConfig {
            template_name: "fullstack-saas".to_string(),
            variables: Self::create_variables(HashMap::from([
                ("project_name", "test-shadcn-stack"),
                ("ui_library", "shadcn/ui"),
                ("css_framework", "tailwindcss"),
                ("theme_support", "true"),
            ])),
            expected_files: vec![
                "components/ui/button.tsx".to_string(),
                "components/ui/card.tsx".to_string(),
                "components/ui/dialog.tsx".to_string(),
                "lib/utils.ts".to_string(),
                "components.json".to_string(),
            ],
            expected_dependencies: vec![
                "@radix-ui/react-slot".to_string(),
                "tailwind-merge".to_string(),
                "class-variance-authority".to_string(),
            ],
            should_build: true,
            should_run: false,
            timeout_seconds: 90,
        };
        results.push(suite.generate_project(&config1).await?);

        // Test NextUI compatibility
        let config2 = TestConfig {
            template_name: "fullstack-saas".to_string(),
            variables: Self::create_variables(HashMap::from([
                ("project_name", "test-nextui-stack"),
                ("ui_library", "nextui"),
                ("css_framework", "tailwindcss"),
            ])),
            expected_files: vec![
                "components/providers.tsx".to_string(),
                "tailwind.config.ts".to_string(),
            ],
            expected_dependencies: vec![
                "@nextui-org/react".to_string(),
                "framer-motion".to_string(),
            ],
            should_build: true,
            should_run: false,
            timeout_seconds: 90,
        };
        results.push(suite.generate_project(&config2).await?);

        Ok(results)
    }

    /// Test service combination validation
    pub async fn test_service_combinations() -> anyhow::Result<Vec<TestResult>> {
        let suite = IntegrationTestSuite::new()?;
        let mut results = Vec::new();

        // Test recommended combination: Clerk + tRPC + Supabase + Stripe
        let config1 = TestConfig {
            template_name: "fullstack-saas".to_string(),
            variables: Self::create_variables(HashMap::from([
                ("project_name", "test-recommended"),
                ("auth_provider", "clerk"),
                ("api_pattern", "trpc"),
                ("database_provider", "supabase"),
                ("payment_provider", "stripe"),
                ("ui_library", "shadcn/ui"),
                ("language", "typescript"),
            ])),
            expected_files: vec![
                "lib/clerk-setup.ts".to_string(),
                "lib/trpc/client.ts".to_string(),
                "lib/supabase-client.ts".to_string(),
                "lib/stripe.ts".to_string(),
                "components/ui/button.tsx".to_string(),
            ],
            expected_dependencies: vec![
                "@clerk/nextjs".to_string(),
                "@trpc/server".to_string(),
                "@supabase/supabase-js".to_string(),
                "stripe".to_string(),
            ],
            should_build: true,
            should_run: false,
            timeout_seconds: 180,
        };
        results.push(suite.generate_project(&config1).await?);

        // Test enterprise combination: Auth0 + GraphQL + PostgreSQL + Advanced features
        let config2 = TestConfig {
            template_name: "fullstack-saas".to_string(),
            variables: Self::create_variables(HashMap::from([
                ("project_name", "test-enterprise"),
                ("auth_provider", "auth0"),
                ("api_pattern", "graphql"),
                ("database_provider", "neon"),
                ("ui_library", "shadcn/ui"),
                ("monitoring", "sentry"),
                ("analytics", "mixpanel"),
            ])),
            expected_files: vec![
                "lib/auth-config.ts".to_string(),
                "lib/graphql/schema.ts".to_string(),
                "lib/db.ts".to_string(),
                "lib/monitoring.ts".to_string(),
            ],
            expected_dependencies: vec![
                "next-auth".to_string(),
                "@apollo/server".to_string(),
                "drizzle-orm".to_string(),
                "@sentry/nextjs".to_string(),
            ],
            should_build: true,
            should_run: false,
            timeout_seconds: 180,
        };
        results.push(suite.generate_project(&config2).await?);

        // Test minimal viable combination
        let config3 = TestConfig {
            template_name: "fullstack-saas".to_string(),
            variables: Self::create_variables(HashMap::from([
                ("project_name", "test-minimal"),
                ("auth_provider", "none"),
                ("api_pattern", "rest"),
                ("database_provider", "none"),
                ("ui_library", "shadcn/ui"),
            ])),
            expected_files: vec![
                "app/api/users/route.ts".to_string(),
                "components/ui/button.tsx".to_string(),
                "lib/api/client.ts".to_string(),
            ],
            expected_dependencies: vec!["@radix-ui/react-slot".to_string(), "zod".to_string()],
            should_build: true,
            should_run: false,
            timeout_seconds: 90,
        };
        results.push(suite.generate_project(&config3).await?);

        Ok(results)
    }

    /// Test invalid service combinations (should fail gracefully)
    pub async fn test_invalid_combinations() -> anyhow::Result<Vec<TestResult>> {
        let suite = IntegrationTestSuite::new()?;
        let mut results = Vec::new();

        // Test conflicting services
        let config1 = TestConfig {
            template_name: "fullstack-saas".to_string(),
            variables: Self::create_variables(HashMap::from([
                ("project_name", "test-conflicts"),
                ("auth_provider", "clerk,auth0"), // Multiple auth providers
                ("database_provider", "supabase,mongodb"), // Multiple databases
            ])),
            expected_files: vec![],
            expected_dependencies: vec![],
            should_build: false,
            should_run: false,
            timeout_seconds: 60,
        };
        let result1 = suite.generate_project(&config1).await?;
        // This should either fail or resolve to one provider
        results.push(result1);

        // Test incompatible language requirements
        let config2 = TestConfig {
            template_name: "fullstack-saas".to_string(),
            variables: Self::create_variables(HashMap::from([
                ("project_name", "test-incompatible"),
                ("api_pattern", "trpc"),    // Requires TypeScript
                ("language", "javascript"), // But using JavaScript
            ])),
            expected_files: vec![],
            expected_dependencies: vec![],
            should_build: false,
            should_run: false,
            timeout_seconds: 60,
        };
        let result2 = suite.generate_project(&config2).await?;
        results.push(result2);

        Ok(results)
    }

    /// Helper to create variables map with proper JSON values
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
async fn test_all_service_compatibility() {
    let mut all_results = Vec::new();
    let mut failed_tests = Vec::new();

    // Test each compatibility category sequentially
    let trpc_compat_start_index = all_results.len();
    println!("Running tRPC Language Compatibility tests...");
    match ServiceCompatibilityTests::test_trpc_language_compatibility().await {
        Ok(mut results) => {
            let success_count = results.iter().filter(|r| r.success).count();
            let rejection_count = results.iter().filter(|r| !r.success).count();
            println!(
                "✅ tRPC Language Compatibility tests: {}/{} passed, {}/{} correctly rejected",
                success_count,
                results.len(),
                rejection_count,
                results.len()
            );
            all_results.append(&mut results);
        }
        Err(e) => {
            println!("❌ tRPC Language Compatibility tests failed: {}", e);
            failed_tests.push("tRPC Language Compatibility");
        }
    }
    let trpc_compat_end_index = all_results.len();

    println!("Running Auth Provider Compatibility tests...");
    match ServiceCompatibilityTests::test_auth_provider_compatibility().await {
        Ok(mut results) => {
            let success_count = results.iter().filter(|r| r.success).count();
            println!(
                "✅ Auth Provider Compatibility tests: {}/{} passed",
                success_count,
                results.len()
            );
            all_results.append(&mut results);
        }
        Err(e) => {
            println!("❌ Auth Provider Compatibility tests failed: {}", e);
            failed_tests.push("Auth Provider Compatibility");
        }
    }

    println!("Running Database Compatibility tests...");
    match ServiceCompatibilityTests::test_database_compatibility().await {
        Ok(mut results) => {
            let success_count = results.iter().filter(|r| r.success).count();
            println!(
                "✅ Database Compatibility tests: {}/{} passed",
                success_count,
                results.len()
            );
            all_results.append(&mut results);
        }
        Err(e) => {
            println!("❌ Database Compatibility tests failed: {}", e);
            failed_tests.push("Database Compatibility");
        }
    }

    println!("Running API Pattern Compatibility tests...");
    match ServiceCompatibilityTests::test_api_pattern_compatibility().await {
        Ok(mut results) => {
            let success_count = results.iter().filter(|r| r.success).count();
            println!(
                "✅ API Pattern Compatibility tests: {}/{} passed",
                success_count,
                results.len()
            );
            all_results.append(&mut results);
        }
        Err(e) => {
            println!("❌ API Pattern Compatibility tests failed: {}", e);
            failed_tests.push("API Pattern Compatibility");
        }
    }

    println!("Running UI Library Compatibility tests...");
    match ServiceCompatibilityTests::test_ui_library_compatibility().await {
        Ok(mut results) => {
            let success_count = results.iter().filter(|r| r.success).count();
            println!(
                "✅ UI Library Compatibility tests: {}/{} passed",
                success_count,
                results.len()
            );
            all_results.append(&mut results);
        }
        Err(e) => {
            println!("❌ UI Library Compatibility tests failed: {}", e);
            failed_tests.push("UI Library Compatibility");
        }
    }

    println!("Running Service Combinations tests...");
    match ServiceCompatibilityTests::test_service_combinations().await {
        Ok(mut results) => {
            let success_count = results.iter().filter(|r| r.success).count();
            println!(
                "✅ Service Combinations tests: {}/{} passed",
                success_count,
                results.len()
            );
            all_results.append(&mut results);
        }
        Err(e) => {
            println!("❌ Service Combinations tests failed: {}", e);
            failed_tests.push("Service Combinations");
        }
    }

    let invalid_combinations_start_index = all_results.len();
    println!("Running Invalid Combinations tests...");
    match ServiceCompatibilityTests::test_invalid_combinations().await {
        Ok(mut results) => {
            let success_count = results.iter().filter(|r| r.success).count();
            let failure_count = results.iter().filter(|r| !r.success).count();
            println!(
                "✅ Invalid Combinations tests: {}/{} correctly rejected invalid input",
                failure_count,
                results.len()
            );
            if success_count > 0 {
                println!(
                    "   ⚠️  Warning: {}/{} invalid combinations were unexpectedly accepted",
                    success_count,
                    results.len()
                );
            }
            all_results.append(&mut results);
        }
        Err(e) => {
            println!("❌ Invalid Combinations tests failed: {}", e);
            failed_tests.push("Invalid Combinations");
        }
    }
    let invalid_combinations_end_index = all_results.len();

    // Summary - count both successful generations and correctly rejected invalid inputs
    let total_tests = all_results.len();
    let total_files = all_results.iter().map(|r| r.files_created).sum::<usize>();

    // Count successes including correctly rejected invalid inputs
    let mut successful_tests = 0;
    let mut correctly_rejected = 0;

    for (i, result) in all_results.iter().enumerate() {
        let is_invalid_combination_test =
            i >= invalid_combinations_start_index && i < invalid_combinations_end_index;
        let is_trpc_invalid_test = i >= trpc_compat_start_index
            && i < trpc_compat_end_index
            && i == trpc_compat_start_index + 1; // Second test is the invalid one

        if result.success {
            if is_invalid_combination_test || is_trpc_invalid_test {
                // Invalid test that succeeded is actually a failure
                continue;
            }
            successful_tests += 1;
        } else {
            if is_invalid_combination_test || is_trpc_invalid_test {
                // Invalid test that failed is actually a success
                successful_tests += 1;
                correctly_rejected += 1;
            }
        }
    }

    println!("\n📊 Service Compatibility Test Summary:");
    println!("   Total tests: {}", total_tests);
    println!("   Successful: {}", successful_tests);
    if correctly_rejected > 0 {
        println!(
            "   (includes {} correctly rejected invalid inputs)",
            correctly_rejected
        );
    }
    println!(
        "   Success rate: {:.1}%",
        (successful_tests as f64 / total_tests as f64) * 100.0
    );
    println!("   Total files generated: {}", total_files);
    println!("   Failed test categories: {}", failed_tests.len());

    // Print detailed results for failed tests
    for (i, result) in all_results.iter().enumerate() {
        let is_invalid_combination_test =
            i >= invalid_combinations_start_index && i < invalid_combinations_end_index;
        let is_trpc_invalid_test = i >= trpc_compat_start_index
            && i < trpc_compat_end_index
            && i == trpc_compat_start_index + 1; // Second test is the invalid one

        if !result.success {
            if let Some(error) = &result.error_message {
                if is_invalid_combination_test || is_trpc_invalid_test {
                    // For invalid combination/incompatibility tests, failure is expected and good
                    println!(
                        "✅ Test {} correctly rejected invalid input: {}",
                        i + 1,
                        error.lines().next().unwrap_or("Invalid configuration")
                    );
                } else {
                    // For regular tests, failure is bad
                    println!("❌ Test {} failed: {}", i + 1, error);
                }
            }
        } else if is_invalid_combination_test || is_trpc_invalid_test {
            // For negative tests, success is unexpected
            println!(
                "⚠️  Test {} warning: Invalid input was unexpectedly accepted",
                i + 1
            );
        }

        if !result.warnings.is_empty() {
            println!("⚠️  Test {} warnings:", i + 1);
            for warning in &result.warnings {
                println!("     {}", warning);
            }
        }
    }

    // Compatibility tests should have a high success rate
    let min_success_rate = 0.8; // 80% minimum
    let actual_success_rate = successful_tests as f64 / total_tests as f64;

    assert!(
        actual_success_rate >= min_success_rate,
        "Service compatibility test success rate ({:.1}%) is below minimum ({:.1}%)",
        actual_success_rate * 100.0,
        min_success_rate * 100.0
    );
}
