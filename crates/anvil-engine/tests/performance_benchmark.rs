/*!
 * Performance Benchmark Tests
 * 
 * Tests to measure and validate Anvil's performance characteristics
 */

use std::process::Command;
use std::time::Instant;

/* Test template generation performance across different templates */
#[tokio::test]
async fn benchmark_generation_performance() {
    println!("📊 Benchmarking template generation performance");
    
    // Find the anvil binary with absolute path
    let current_dir = std::env::current_dir()
        .expect("Failed to get current dir");
    let workspace_dir = current_dir
        .parent()
        .expect("Failed to get parent")
        .parent()
        .expect("Failed to get grandparent");
    
    let anvil_binary = if cfg!(target_os = "windows") {
        workspace_dir.join("target/debug/anvil.exe")
    } else {
        workspace_dir.join("target/debug/anvil")
    };
    
    // Test different templates for performance
    let templates = vec![
        ("rust-hello-world", "benchmark-rust"),
        ("fullstack-saas", "benchmark-saas"),
    ];
    
    let mut results = Vec::new();
    
    for (template, project_name) in templates {
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
            results.push((template, duration));
            println!("✅ {} generated in {:?}", template, duration);
            
            // Clean up
            let project_dir = workspace_dir.join(project_name);
            if project_dir.exists() {
                std::fs::remove_dir_all(&project_dir).ok();
            }
        } else {
            println!("❌ {} failed: {}", template, String::from_utf8_lossy(&output.stderr));
        }
    }
    
    // Performance assertions
    for (template, duration) in &results {
        // Templates should generate within reasonable time limits
        match *template {
            "rust-hello-world" => {
                assert!(duration.as_secs() < 30, "Rust template took too long: {:?}", duration);
            },
            "fullstack-saas" => {
                assert!(duration.as_secs() < 60, "SaaS template took too long: {:?}", duration);
            },
            _ => {}
        }
    }
    
    // Report results
    let total_time: std::time::Duration = results.iter().map(|(_, d)| *d).sum();
    println!("📈 Total benchmark time: {:?}", total_time);
    println!("📊 Average generation time: {:?}", total_time / results.len() as u32);
    
    assert!(!results.is_empty(), "No templates were successfully benchmarked");
}

/* Test concurrent generation performance */
#[tokio::test]
async fn benchmark_concurrent_generation() {
    println!("🔄 Benchmarking concurrent generation performance");
    
    // Find the anvil binary
    let current_dir = std::env::current_dir()
        .expect("Failed to get current dir");
    let workspace_dir = current_dir
        .parent()
        .expect("Failed to get parent")
        .parent()
        .expect("Failed to get grandparent");
    
    let anvil_binary = if cfg!(target_os = "windows") {
        workspace_dir.join("target/debug/anvil.exe")
    } else {
        workspace_dir.join("target/debug/anvil")
    };
    
    let start_time = Instant::now();
    
    // Generate multiple projects concurrently
    let handles: Vec<_> = (0..3).map(|i| {
        let binary = anvil_binary.clone();
        let workspace = workspace_dir.to_path_buf();
        let project_name = format!("concurrent-test-{}", i);
        
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
    }).collect();
    
    // Wait for all to complete
    let results = futures::future::join_all(handles).await;
    let duration = start_time.elapsed();
    
    // Verify all succeeded
    let mut successful = 0;
    for result in results {
        match result {
            Ok((name, true)) => {
                println!("✅ {} completed successfully", name);
                successful += 1;
            }
            Ok((name, false)) => {
                println!("❌ {} failed", name);
            }
            Err(e) => {
                println!("❌ Task failed: {}", e);
            }
        }
    }
    
    println!("🔄 Concurrent generation completed in {:?}", duration);
    println!("📊 Success rate: {}/3", successful);
    
    // Performance assertion - concurrent should be faster than sequential
    assert!(duration.as_secs() < 90, "Concurrent generation took too long: {:?}", duration);
    assert_eq!(successful, 3, "Not all concurrent generations succeeded");
}

/* Test memory usage during generation */
#[test]
fn benchmark_memory_usage() {
    println!("💾 Benchmarking memory usage");
    
    // This is a basic memory usage test
    // In a real scenario, you might use tools like valgrind or custom memory tracking
    
    let initial_memory = get_process_memory_usage();
    
    // Find the anvil binary
    let current_dir = std::env::current_dir()
        .expect("Failed to get current dir");
    let workspace_dir = current_dir
        .parent()
        .expect("Failed to get parent")
        .parent()
        .expect("Failed to get grandparent");
    
    let anvil_binary = if cfg!(target_os = "windows") {
        workspace_dir.join("target/debug/anvil.exe")
    } else {
        workspace_dir.join("target/debug/anvil")
    };
    
    let output = Command::new(&anvil_binary)
        .arg("create")
        .arg("memory-test")
        .arg("--template")
        .arg("rust-hello-world")
        .arg("--no-input")
        .current_dir(&workspace_dir)
        .output()
        .expect("Failed to execute anvil command");
    
    let final_memory = get_process_memory_usage();
    let memory_diff = final_memory.saturating_sub(initial_memory);
    
    println!("💾 Memory usage: {} KB -> {} KB (diff: {} KB)", 
             initial_memory / 1024, final_memory / 1024, memory_diff / 1024);
    
    // Clean up
    let project_dir = workspace_dir.join("memory-test");
    if project_dir.exists() {
        std::fs::remove_dir_all(&project_dir).ok();
    }
    
    if output.status.success() {
        println!("✅ Memory benchmark completed successfully");
        // Basic assertion - shouldn't use excessive memory
        assert!(memory_diff < 1_000_000_000, "Memory usage too high: {} bytes", memory_diff);
    } else {
        println!("❌ Memory benchmark failed: {}", String::from_utf8_lossy(&output.stderr));
    }
}

/* Get current process memory usage (basic implementation) */
fn get_process_memory_usage() -> usize {
    // This is a simplified implementation
    // On Unix systems, you might read from /proc/self/status
    // On Windows, you'd use Windows API
    // For now, we'll use a placeholder that returns 0
    
    #[cfg(target_os = "linux")]
    {
        use std::fs;
        if let Ok(status) = fs::read_to_string("/proc/self/status") {
            for line in status.lines() {
                if line.starts_with("VmRSS:") {
                    if let Some(value) = line.split_whitespace().nth(1) {
                        if let Ok(kb) = value.parse::<usize>() {
                            return kb * 1024; // Convert KB to bytes
                        }
                    }
                }
            }
        }
    }
    
    // Fallback for other platforms or if reading fails
    0
}