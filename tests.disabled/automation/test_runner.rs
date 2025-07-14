// Automated test runner for LumosAI framework
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use std::collections::HashMap;

/// Test suite configuration
#[derive(Debug, Clone)]
pub struct TestSuiteConfig {
    pub name: String,
    pub timeout: Duration,
    pub retry_count: u32,
    pub parallel: bool,
    pub coverage_required: bool,
    pub performance_baseline: Option<Duration>,
}

/// Test result
#[derive(Debug, Clone)]
pub struct TestResult {
    pub suite_name: String,
    pub passed: bool,
    pub duration: Duration,
    pub output: String,
    pub error: Option<String>,
    pub coverage: Option<f64>,
}

/// Test runner
pub struct TestRunner {
    pub config: HashMap<String, TestSuiteConfig>,
    pub results: Vec<TestResult>,
}

impl TestRunner {
    pub fn new() -> Self {
        let mut config = HashMap::new();
        
        // Unit tests configuration
        config.insert("unit".to_string(), TestSuiteConfig {
            name: "Unit Tests".to_string(),
            timeout: Duration::from_secs(300), // 5 minutes
            retry_count: 2,
            parallel: true,
            coverage_required: true,
            performance_baseline: Some(Duration::from_secs(60)),
        });
        
        // Integration tests configuration
        config.insert("integration".to_string(), TestSuiteConfig {
            name: "Integration Tests".to_string(),
            timeout: Duration::from_secs(600), // 10 minutes
            retry_count: 1,
            parallel: false,
            coverage_required: true,
            performance_baseline: Some(Duration::from_secs(300)),
        });
        
        // Performance tests configuration
        config.insert("performance".to_string(), TestSuiteConfig {
            name: "Performance Tests".to_string(),
            timeout: Duration::from_secs(1800), // 30 minutes
            retry_count: 0,
            parallel: false,
            coverage_required: false,
            performance_baseline: Some(Duration::from_secs(900)),
        });
        
        // Example validation configuration
        config.insert("examples".to_string(), TestSuiteConfig {
            name: "Example Validation".to_string(),
            timeout: Duration::from_secs(900), // 15 minutes
            retry_count: 1,
            parallel: true,
            coverage_required: false,
            performance_baseline: Some(Duration::from_secs(300)),
        });
        
        Self {
            config,
            results: Vec::new(),
        }
    }
    
    /// Run all test suites
    pub async fn run_all(&mut self) -> bool {
        println!("🚀 Starting automated test execution...");
        println!("==========================================");
        
        let start_time = Instant::now();
        let mut all_passed = true;
        
        // Run test suites in order
        let suite_order = vec!["unit", "integration", "examples", "performance"];
        
        for suite_name in suite_order {
            if let Some(config) = self.config.get(suite_name).cloned() {
                println!("\n📋 Running {}", config.name);
                println!("----------------------------------------");
                
                let result = self.run_suite(suite_name, &config).await;
                
                if result.passed {
                    println!("✅ {} passed in {:?}", config.name, result.duration);
                } else {
                    println!("❌ {} failed in {:?}", config.name, result.duration);
                    if let Some(error) = &result.error {
                        println!("   Error: {}", error);
                    }
                    all_passed = false;
                }
                
                self.results.push(result);
            }
        }
        
        let total_duration = start_time.elapsed();
        
        println!("\n🏁 Test execution completed");
        println!("==========================================");
        self.print_summary(total_duration);
        
        all_passed
    }
    
    /// Run a specific test suite
    pub async fn run_suite(&self, suite_name: &str, config: &TestSuiteConfig) -> TestResult {
        let start_time = Instant::now();
        
        // Determine test command based on suite
        let (command, args) = match suite_name {
            "unit" => ("cargo", vec!["test", "--lib", "--tests", "unit"]),
            "integration" => ("cargo", vec!["test", "--tests", "integration"]),
            "performance" => ("cargo", vec!["test", "--tests", "performance", "--release"]),
            "examples" => ("cargo", vec!["test", "--examples"]),
            _ => ("cargo", vec!["test"]),
        };
        
        // Execute test command with timeout
        let result = self.execute_with_timeout(command, &args, config.timeout).await;
        
        let duration = start_time.elapsed();
        
        match result {
            Ok(output) => {
                // Check if tests passed
                let passed = output.contains("test result: ok") || 
                           !output.contains("FAILED") && 
                           !output.contains("error:");
                
                // Extract coverage if available
                let coverage = self.extract_coverage(&output);
                
                TestResult {
                    suite_name: suite_name.to_string(),
                    passed,
                    duration,
                    output,
                    error: None,
                    coverage,
                }
            }
            Err(error) => {
                TestResult {
                    suite_name: suite_name.to_string(),
                    passed: false,
                    duration,
                    output: String::new(),
                    error: Some(error),
                    coverage: None,
                }
            }
        }
    }
    
    /// Execute command with timeout
    async fn execute_with_timeout(
        &self,
        command: &str,
        args: &[&str],
        timeout: Duration,
    ) -> Result<String, String> {
        let mut cmd = Command::new(command);
        cmd.args(args)
           .stdout(Stdio::piped())
           .stderr(Stdio::piped());
        
        let start = Instant::now();
        
        match cmd.output() {
            Ok(output) => {
                let elapsed = start.elapsed();
                
                if elapsed > timeout {
                    return Err(format!("Command timed out after {:?}", elapsed));
                }
                
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                
                if output.status.success() {
                    Ok(format!("{}\n{}", stdout, stderr))
                } else {
                    Err(format!("Command failed: {}\n{}", stdout, stderr))
                }
            }
            Err(e) => Err(format!("Failed to execute command: {}", e)),
        }
    }
    
    /// Extract coverage percentage from output
    fn extract_coverage(&self, output: &str) -> Option<f64> {
        // Look for coverage patterns in output
        for line in output.lines() {
            if line.contains("coverage:") || line.contains("Coverage:") {
                // Extract percentage using regex or simple parsing
                if let Some(start) = line.find(char::is_numeric) {
                    if let Some(end) = line[start..].find('%') {
                        if let Ok(coverage) = line[start..start + end].parse::<f64>() {
                            return Some(coverage);
                        }
                    }
                }
            }
        }
        None
    }
    
    /// Print test summary
    fn print_summary(&self, total_duration: Duration) {
        let total_tests = self.results.len();
        let passed_tests = self.results.iter().filter(|r| r.passed).count();
        let failed_tests = total_tests - passed_tests;
        
        println!("📊 Test Summary:");
        println!("   Total suites: {}", total_tests);
        println!("   Passed: {} ✅", passed_tests);
        println!("   Failed: {} ❌", failed_tests);
        println!("   Total time: {:?}", total_duration);
        
        if failed_tests == 0 {
            println!("   Status: 🎉 ALL TESTS PASSED!");
        } else {
            println!("   Status: ⚠️  {} test suite(s) failed", failed_tests);
        }
        
        println!("\n📋 Detailed Results:");
        for result in &self.results {
            let status = if result.passed { "✅" } else { "❌" };
            println!("   {} {} - {:?}", status, result.suite_name, result.duration);
            
            if let Some(coverage) = result.coverage {
                println!("      Coverage: {:.1}%", coverage);
            }
            
            if let Some(error) = &result.error {
                println!("      Error: {}", error.lines().next().unwrap_or("Unknown error"));
            }
        }
        
        // Performance analysis
        println!("\n⚡ Performance Analysis:");
        for result in &self.results {
            if let Some(config) = self.config.get(&result.suite_name) {
                if let Some(baseline) = config.performance_baseline {
                    let performance_ratio = result.duration.as_secs_f64() / baseline.as_secs_f64();
                    let status = if performance_ratio <= 1.0 { "🟢" } else if performance_ratio <= 1.5 { "🟡" } else { "🔴" };
                    
                    println!("   {} {} - {:.1}x baseline ({:?} vs {:?})", 
                             status, result.suite_name, performance_ratio, result.duration, baseline);
                }
            }
        }
    }
    
    /// Generate test report
    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        
        report.push_str("# LumosAI Test Report\n\n");
        report.push_str(&format!("Generated: {}\n\n", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));
        
        // Summary
        let total_tests = self.results.len();
        let passed_tests = self.results.iter().filter(|r| r.passed).count();
        
        report.push_str("## Summary\n\n");
        report.push_str(&format!("- **Total Test Suites**: {}\n", total_tests));
        report.push_str(&format!("- **Passed**: {} ✅\n", passed_tests));
        report.push_str(&format!("- **Failed**: {} ❌\n", total_tests - passed_tests));
        
        let total_duration: Duration = self.results.iter().map(|r| r.duration).sum();
        report.push_str(&format!("- **Total Duration**: {:?}\n\n", total_duration));
        
        // Detailed results
        report.push_str("## Detailed Results\n\n");
        
        for result in &self.results {
            let status = if result.passed { "✅ PASSED" } else { "❌ FAILED" };
            report.push_str(&format!("### {} - {}\n\n", result.suite_name, status));
            report.push_str(&format!("- **Duration**: {:?}\n", result.duration));
            
            if let Some(coverage) = result.coverage {
                report.push_str(&format!("- **Coverage**: {:.1}%\n", coverage));
            }
            
            if let Some(error) = &result.error {
                report.push_str(&format!("- **Error**: {}\n", error));
            }
            
            report.push_str("\n");
        }
        
        report
    }
    
    /// Save report to file
    pub fn save_report(&self, filename: &str) -> Result<(), std::io::Error> {
        use std::fs;
        let report = self.generate_report();
        fs::write(filename, report)?;
        println!("📄 Test report saved to: {}", filename);
        Ok(())
    }
}

/// CLI interface for test runner
pub struct TestRunnerCli;

impl TestRunnerCli {
    pub async fn run() {
        let args: Vec<String> = std::env::args().collect();
        
        let mut runner = TestRunner::new();
        
        if args.len() > 1 {
            match args[1].as_str() {
                "unit" => {
                    if let Some(config) = runner.config.get("unit").cloned() {
                        let result = runner.run_suite("unit", &config).await;
                        println!("Unit tests: {}", if result.passed { "PASSED" } else { "FAILED" });
                    }
                }
                "integration" => {
                    if let Some(config) = runner.config.get("integration").cloned() {
                        let result = runner.run_suite("integration", &config).await;
                        println!("Integration tests: {}", if result.passed { "PASSED" } else { "FAILED" });
                    }
                }
                "performance" => {
                    if let Some(config) = runner.config.get("performance").cloned() {
                        let result = runner.run_suite("performance", &config).await;
                        println!("Performance tests: {}", if result.passed { "PASSED" } else { "FAILED" });
                    }
                }
                "all" | _ => {
                    let success = runner.run_all().await;
                    std::process::exit(if success { 0 } else { 1 });
                }
            }
        } else {
            let success = runner.run_all().await;
            
            // Save report
            if let Err(e) = runner.save_report("test_report.md") {
                eprintln!("Failed to save report: {}", e);
            }
            
            std::process::exit(if success { 0 } else { 1 });
        }
    }
}
