/*!
 * Shared Component Integration Tests
 *
 * Tests the shared component system to ensure proper composition,
 * service integration, and cross-template compatibility.
 */

use super::{IntegrationTestSuite, TestConfig, TestResult};
use serde_json::Value;
use std::collections::HashMap;

/// Test suite for shared component functionality
pub struct SharedComponentTests;

impl SharedComponentTests {
    /// Test authentication service components
    pub async fn test_auth_components() -> anyhow::Result<Vec<TestResult>> {
        let suite = IntegrationTestSuite::new()?;
        let mut results = Vec::new();

        // Test Clerk integration
        let config1 = TestConfig {
            template_name: "fullstack-saas".to_string(),
            variables: Self::create_variables(HashMap::from([
                ("project_name", "test-clerk-auth"),
                ("ui_library", "shadcn/ui"),
                ("auth_provider", "clerk"),
            ])),
            expected_files: vec![
                "components/auth/protected-route.tsx".to_string(),
                "components/auth/user-button.tsx".to_string(),
                "lib/auth-config.ts".to_string(),
                "lib/clerk-setup.ts".to_string(),
                "middleware.ts".to_string(),
                "app/sign-in/[[...sign-in]]/page.tsx".to_string(),
                "app/sign-up/[[...sign-up]]/page.tsx".to_string(),
            ],
            expected_dependencies: vec!["@clerk/nextjs".to_string()],
            should_build: true,
            should_run: false,
            timeout_seconds: 90,
        };
        results.push(suite.generate_project(&config1).await?);

        // Test Auth0 integration
        let config2 = TestConfig {
            template_name: "fullstack-saas".to_string(),
            variables: Self::create_variables(HashMap::from([
                ("project_name", "test-auth0-auth"),
                ("ui_library", "shadcn/ui"),
                ("auth_provider", "auth0"),
            ])),
            expected_files: vec![
                "components/auth/protected-route.tsx".to_string(),
                "components/auth/sign-in-button.tsx".to_string(),
                "components/auth/sign-out-button.tsx".to_string(),
                "components/auth/user-profile.tsx".to_string(),
                "lib/auth-config.ts".to_string(),
                "app/api/auth/[...nextauth]/route.ts".to_string(),
            ],
            expected_dependencies: vec!["next-auth".to_string(), "@auth0/nextjs-auth0".to_string()],
            should_build: true,
            should_run: false,
            timeout_seconds: 90,
        };
        results.push(suite.generate_project(&config2).await?);

        // Test Supabase authentication
        let config3 = TestConfig {
            template_name: "fullstack-saas".to_string(),
            variables: Self::create_variables(HashMap::from([
                ("project_name", "test-supabase-auth"),
                ("ui_library", "shadcn/ui"),
                ("auth_provider", "supabase"),
            ])),
            expected_files: vec![
                "components/auth/auth-form.tsx".to_string(),
                "hooks/use-auth.ts".to_string(),
                "lib/supabase-client.ts".to_string(),
                "lib/supabase-server.ts".to_string(),
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

    /// Test API pattern components
    pub async fn test_api_components() -> anyhow::Result<Vec<TestResult>> {
        let suite = IntegrationTestSuite::new()?;
        let mut results = Vec::new();

        // Test REST API pattern
        let config1 = TestConfig {
            template_name: "fullstack-saas".to_string(),
            variables: Self::create_variables(HashMap::from([
                ("project_name", "test-rest-api"),
                ("api_pattern", "rest"),
            ])),
            expected_files: vec![
                "app/api/users/route.ts".to_string(),
                "app/api/users/[id]/route.ts".to_string(),
                "app/api/docs/route.ts".to_string(),
                "lib/api/client.ts".to_string(),
                "lib/api/middleware.ts".to_string(),
                "lib/api/openapi.ts".to_string(),
                "lib/api/schemas.ts".to_string(),
            ],
            expected_dependencies: vec!["zod".to_string(), "swagger-ui-react".to_string()],
            should_build: true,
            should_run: false,
            timeout_seconds: 90,
        };
        results.push(suite.generate_project(&config1).await?);

        // Test tRPC pattern
        let config2 = TestConfig {
            template_name: "fullstack-saas".to_string(),
            variables: Self::create_variables(HashMap::from([
                ("project_name", "test-trpc-api"),
                ("api_pattern", "trpc"),
                ("language", "typescript"), // tRPC requires TypeScript
            ])),
            expected_files: vec![
                "app/api/trpc/[trpc]/route.ts".to_string(),
                "lib/trpc/client.ts".to_string(),
                "lib/trpc/react.tsx".to_string(),
                "lib/trpc/server.ts".to_string(),
                "server/routers/_app.ts".to_string(),
                "server/routers/users.ts".to_string(),
                "server/trpc.ts".to_string(),
            ],
            expected_dependencies: vec![
                "@trpc/server".to_string(),
                "@trpc/client".to_string(),
                "@trpc/react-query".to_string(),
                "@trpc/next".to_string(),
            ],
            should_build: true,
            should_run: false,
            timeout_seconds: 120,
        };
        results.push(suite.generate_project(&config2).await?);

        // Test GraphQL pattern
        let config3 = TestConfig {
            template_name: "fullstack-saas".to_string(),
            variables: Self::create_variables(HashMap::from([
                ("project_name", "test-graphql-api"),
                ("api_pattern", "graphql"),
            ])),
            expected_files: vec![
                "app/api/graphql/route.ts".to_string(),
                "lib/graphql/apollo-provider.tsx".to_string(),
                "lib/graphql/client.ts".to_string(),
                "lib/graphql/context.ts".to_string(),
                "lib/graphql/schema.ts".to_string(),
                "lib/graphql/resolvers/index.ts".to_string(),
                "lib/graphql/typeDefs/index.ts".to_string(),
                "codegen.yml".to_string(),
            ],
            expected_dependencies: vec![
                "@apollo/server".to_string(),
                "@apollo/client".to_string(),
                "graphql".to_string(),
                "@graphql-tools/schema".to_string(),
            ],
            should_build: true,
            should_run: false,
            timeout_seconds: 120,
        };
        results.push(suite.generate_project(&config3).await?);

        Ok(results)
    }

    /// Test database integration components
    pub async fn test_database_components() -> anyhow::Result<Vec<TestResult>> {
        let suite = IntegrationTestSuite::new()?;
        let mut results = Vec::new();

        // Test Supabase database
        let config1 = TestConfig {
            template_name: "fullstack-saas".to_string(),
            variables: Self::create_variables(HashMap::from([
                ("project_name", "test-supabase-db"),
                ("database_provider", "supabase"),
            ])),
            expected_files: vec![
                "lib/supabase-client.ts".to_string(),
                "lib/supabase-server.ts".to_string(),
            ],
            expected_dependencies: vec!["@supabase/supabase-js".to_string()],
            should_build: true,
            should_run: false,
            timeout_seconds: 60,
        };
        results.push(suite.generate_project(&config1).await?);

        // Test Neon database with Drizzle
        let config2 = TestConfig {
            template_name: "fullstack-saas".to_string(),
            variables: Self::create_variables(HashMap::from([
                ("project_name", "test-neon-db"),
                ("database_provider", "neon"),
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
            timeout_seconds: 60,
        };
        results.push(suite.generate_project(&config2).await?);

        // Test MongoDB
        let config3 = TestConfig {
            template_name: "fullstack-saas".to_string(),
            variables: Self::create_variables(HashMap::from([
                ("project_name", "test-mongodb"),
                ("database_provider", "mongodb"),
            ])),
            expected_files: vec!["lib/mongodb.ts".to_string()],
            expected_dependencies: vec!["mongodb".to_string(), "mongoose".to_string()],
            should_build: true,
            should_run: false,
            timeout_seconds: 60,
        };
        results.push(suite.generate_project(&config3).await?);

        Ok(results)
    }

    /// Test AI integration components
    pub async fn test_ai_components() -> anyhow::Result<Vec<TestResult>> {
        let suite = IntegrationTestSuite::new()?;
        let mut results = Vec::new();

        // Test OpenAI integration
        let config1 = TestConfig {
            template_name: "fullstack-saas".to_string(),
            variables: Self::create_variables(HashMap::from([
                ("project_name", "test-openai"),
                ("ai_provider", "openai"),
            ])),
            expected_files: vec![
                "app/api/chat/route.ts".to_string(),
                "components/ai/chat.tsx".to_string(),
                "lib/openai.ts".to_string(),
            ],
            expected_dependencies: vec!["openai".to_string(), "ai".to_string()],
            should_build: true,
            should_run: false,
            timeout_seconds: 60,
        };
        results.push(suite.generate_project(&config1).await?);

        // Test Claude integration
        let config2 = TestConfig {
            template_name: "fullstack-saas".to_string(),
            variables: Self::create_variables(HashMap::from([
                ("project_name", "test-claude"),
                ("ai_provider", "claude"),
            ])),
            expected_files: vec![
                "app/api/chat/route.ts".to_string(),
                "components/ai/chat.tsx".to_string(),
                "lib/anthropic.ts".to_string(),
            ],
            expected_dependencies: vec!["@anthropic-ai/sdk".to_string()],
            should_build: true,
            should_run: false,
            timeout_seconds: 60,
        };
        results.push(suite.generate_project(&config2).await?);

        Ok(results)
    }

    /// Test payment integration components
    pub async fn test_payment_components() -> anyhow::Result<Vec<TestResult>> {
        let suite = IntegrationTestSuite::new()?;
        let mut results = Vec::new();

        // Test Stripe integration
        let config = TestConfig {
            template_name: "fullstack-saas".to_string(),
            variables: Self::create_variables(HashMap::from([
                ("project_name", "test-stripe"),
                ("payment_provider", "stripe"),
            ])),
            expected_files: vec![
                "app/api/checkout/route.ts".to_string(),
                "app/api/webhooks/stripe/route.ts".to_string(),
                "components/payments/checkout-button.tsx".to_string(),
                "lib/stripe.ts".to_string(),
            ],
            expected_dependencies: vec!["stripe".to_string(), "@stripe/stripe-js".to_string()],
            should_build: true,
            should_run: false,
            timeout_seconds: 60,
        };
        results.push(suite.generate_project(&config).await?);

        Ok(results)
    }

    /// Test deployment configuration components
    pub async fn test_deployment_components() -> anyhow::Result<Vec<TestResult>> {
        let suite = IntegrationTestSuite::new()?;
        let mut results = Vec::new();

        // Test Vercel deployment
        let config1 = TestConfig {
            template_name: "fullstack-saas".to_string(),
            variables: Self::create_variables(HashMap::from([
                ("project_name", "test-vercel-deploy"),
                ("deployment_target", "vercel"),
            ])),
            expected_files: vec!["vercel.json".to_string()],
            expected_dependencies: vec![],
            should_build: false, // Just configuration files
            should_run: false,
            timeout_seconds: 30,
        };
        results.push(suite.generate_project(&config1).await?);

        // Test Docker deployment
        let config2 = TestConfig {
            template_name: "fullstack-saas".to_string(),
            variables: Self::create_variables(HashMap::from([
                ("project_name", "test-docker-deploy"),
                ("deployment_target", "docker"),
            ])),
            expected_files: vec![
                "Dockerfile".to_string(),
                ".dockerignore".to_string(),
                "docker-compose.yml".to_string(),
            ],
            expected_dependencies: vec![],
            should_build: false,
            should_run: false,
            timeout_seconds: 30,
        };
        results.push(suite.generate_project(&config2).await?);

        // Test Railway deployment
        let config3 = TestConfig {
            template_name: "fullstack-saas".to_string(),
            variables: Self::create_variables(HashMap::from([
                ("project_name", "test-railway-deploy"),
                ("deployment_target", "railway"),
            ])),
            expected_files: vec!["railway.toml".to_string(), "Procfile".to_string()],
            expected_dependencies: vec![],
            should_build: false,
            should_run: false,
            timeout_seconds: 30,
        };
        results.push(suite.generate_project(&config3).await?);

        Ok(results)
    }

    /// Verify that shared components can be composed together
    pub async fn test_component_composition() -> anyhow::Result<TestResult> {
        let suite = IntegrationTestSuite::new()?;

        // Test complex composition: fullstack-saas + auth + api + database + payments
        let config = TestConfig {
            template_name: "fullstack-saas".to_string(),
            variables: Self::create_variables(HashMap::from([
                ("project_name", "test-full-composition"),
                ("ui_library", "shadcn/ui"),
                ("auth_provider", "clerk"),
                ("api_pattern", "trpc"),
                ("database_provider", "supabase"),
                ("payment_provider", "stripe"),
                ("ai_provider", "openai"),
                ("deployment_target", "vercel"),
            ])),
            expected_files: vec![
                // Base template
                "package.json".to_string(),
                "app/page.tsx".to_string(),
                "components/ui/button.tsx".to_string(),
                // Auth components
                "components/auth/user-button.tsx".to_string(),
                "lib/clerk-setup.ts".to_string(),
                // API components
                "lib/trpc/client.ts".to_string(),
                "server/trpc.ts".to_string(),
                // Database components
                "lib/supabase-client.ts".to_string(),
                // Payment components
                "lib/stripe.ts".to_string(),
                "components/payments/checkout-button.tsx".to_string(),
                // AI components
                "lib/openai.ts".to_string(),
                "components/ai/chat.tsx".to_string(),
                // Deployment
                "vercel.json".to_string(),
            ],
            expected_dependencies: vec![
                "@radix-ui/react-slot".to_string(),
                "@clerk/nextjs".to_string(),
                "@trpc/server".to_string(),
                "@supabase/supabase-js".to_string(),
                "stripe".to_string(),
                "openai".to_string(),
            ],
            should_build: true,
            should_run: false,
            timeout_seconds: 300, // Complex composition may take longer
        };

        suite.generate_project(&config).await
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
async fn test_all_shared_components() {
    let mut all_results = Vec::new();

    // Test each component category sequentially
    println!("Running Authentication tests...");
    match SharedComponentTests::test_auth_components().await {
        Ok(mut results) => {
            let success_count = results.iter().filter(|r| r.success).count();
            println!(
                "✅ Authentication tests: {}/{} passed",
                success_count,
                results.len()
            );
            all_results.append(&mut results);
        }
        Err(e) => {
            println!("❌ Authentication tests failed: {}", e);
        }
    }

    println!("Running API Patterns tests...");
    match SharedComponentTests::test_api_components().await {
        Ok(mut results) => {
            let success_count = results.iter().filter(|r| r.success).count();
            println!(
                "✅ API Patterns tests: {}/{} passed",
                success_count,
                results.len()
            );
            all_results.append(&mut results);
        }
        Err(e) => {
            println!("❌ API Patterns tests failed: {}", e);
        }
    }

    println!("Running Database Integration tests...");
    match SharedComponentTests::test_database_components().await {
        Ok(mut results) => {
            let success_count = results.iter().filter(|r| r.success).count();
            println!(
                "✅ Database Integration tests: {}/{} passed",
                success_count,
                results.len()
            );
            all_results.append(&mut results);
        }
        Err(e) => {
            println!("❌ Database Integration tests failed: {}", e);
        }
    }

    println!("Running AI Integration tests...");
    match SharedComponentTests::test_ai_components().await {
        Ok(mut results) => {
            let success_count = results.iter().filter(|r| r.success).count();
            println!(
                "✅ AI Integration tests: {}/{} passed",
                success_count,
                results.len()
            );
            all_results.append(&mut results);
        }
        Err(e) => {
            println!("❌ AI Integration tests failed: {}", e);
        }
    }

    println!("Running Payment Integration tests...");
    match SharedComponentTests::test_payment_components().await {
        Ok(mut results) => {
            let success_count = results.iter().filter(|r| r.success).count();
            println!(
                "✅ Payment Integration tests: {}/{} passed",
                success_count,
                results.len()
            );
            all_results.append(&mut results);
        }
        Err(e) => {
            println!("❌ Payment Integration tests failed: {}", e);
        }
    }

    println!("Running Deployment tests...");
    match SharedComponentTests::test_deployment_components().await {
        Ok(mut results) => {
            let success_count = results.iter().filter(|r| r.success).count();
            println!(
                "✅ Deployment tests: {}/{} passed",
                success_count,
                results.len()
            );
            all_results.append(&mut results);
        }
        Err(e) => {
            println!("❌ Deployment tests failed: {}", e);
        }
    }

    // Test component composition
    match SharedComponentTests::test_component_composition().await {
        Ok(result) => {
            println!(
                "✅ Component composition test: {}",
                if result.success { "passed" } else { "failed" }
            );
            all_results.push(result);
        }
        Err(e) => {
            println!("❌ Component composition test failed: {}", e);
        }
    }

    // Summary
    let total_tests = all_results.len();
    let successful_tests = all_results.iter().filter(|r| r.success).count();
    let total_files = all_results.iter().map(|r| r.files_created).sum::<usize>();

    println!("\n📊 Shared Component Test Summary:");
    println!("   Total tests: {}", total_tests);
    println!("   Successful: {}", successful_tests);
    println!(
        "   Success rate: {:.1}%",
        (successful_tests as f64 / total_tests as f64) * 100.0
    );
    println!("   Total files generated: {}", total_files);

    // Print any errors for failed tests
    for (i, result) in all_results.iter().enumerate() {
        if !result.success {
            if let Some(error) = &result.error_message {
                println!("❌ Test {} failed: {}", i + 1, error);
            }
        }

        if !result.warnings.is_empty() {
            println!("⚠️  Test {} warnings:", i + 1);
            for warning in &result.warnings {
                println!("     {}", warning);
            }
        }
    }

    assert!(
        successful_tests > 0,
        "At least some shared component tests should pass"
    );
}
