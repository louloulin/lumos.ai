use std::time::{Duration, Instant};
use std::collections::HashMap;

/// 性能基准测试框架
pub struct PerformanceBenchmark {
    name: String,
    measurements: Vec<Duration>,
    memory_measurements: Vec<usize>,
    start_time: Option<Instant>,
}

impl PerformanceBenchmark {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            measurements: Vec::new(),
            memory_measurements: Vec::new(),
            start_time: None,
        }
    }
    
    /// 开始测量
    pub fn start(&mut self) {
        self.start_time = Some(Instant::now());
    }
    
    /// 结束测量并记录
    pub fn end(&mut self) {
        if let Some(start) = self.start_time.take() {
            let duration = start.elapsed();
            self.measurements.push(duration);
        }
    }
    
    /// 记录内存使用
    pub fn record_memory(&mut self, usage: usize) {
        self.memory_measurements.push(usage);
    }
    
    /// 获取平均执行时间
    pub fn average_duration(&self) -> Duration {
        if self.measurements.is_empty() {
            return Duration::from_millis(0);
        }
        
        let total: Duration = self.measurements.iter().sum();
        total / self.measurements.len() as u32
    }
    
    /// 获取最小执行时间
    pub fn min_duration(&self) -> Duration {
        self.measurements.iter().min().copied().unwrap_or_default()
    }
    
    /// 获取最大执行时间
    pub fn max_duration(&self) -> Duration {
        self.measurements.iter().max().copied().unwrap_or_default()
    }
    
    /// 获取P95执行时间
    pub fn p95_duration(&self) -> Duration {
        if self.measurements.is_empty() {
            return Duration::from_millis(0);
        }
        
        let mut sorted = self.measurements.clone();
        sorted.sort();
        let index = (sorted.len() as f64 * 0.95) as usize;
        sorted.get(index).copied().unwrap_or_default()
    }
    
    /// 获取平均内存使用
    pub fn average_memory(&self) -> usize {
        if self.memory_measurements.is_empty() {
            return 0;
        }
        
        self.memory_measurements.iter().sum::<usize>() / self.memory_measurements.len()
    }
    
    /// 生成报告
    pub fn report(&self) -> PerformanceReport {
        PerformanceReport {
            name: self.name.clone(),
            sample_count: self.measurements.len(),
            avg_duration: self.average_duration(),
            min_duration: self.min_duration(),
            max_duration: self.max_duration(),
            p95_duration: self.p95_duration(),
            avg_memory: self.average_memory(),
        }
    }
}

/// 性能报告
#[derive(Debug, Clone)]
pub struct PerformanceReport {
    pub name: String,
    pub sample_count: usize,
    pub avg_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub p95_duration: Duration,
    pub avg_memory: usize,
}

impl PerformanceReport {
    /// 打印报告
    pub fn print(&self) {
        println!("=== Performance Report: {} ===", self.name);
        println!("Sample Count: {}", self.sample_count);
        println!("Average Duration: {:?}", self.avg_duration);
        println!("Min Duration: {:?}", self.min_duration);
        println!("Max Duration: {:?}", self.max_duration);
        println!("P95 Duration: {:?}", self.p95_duration);
        println!("Average Memory: {} bytes", self.avg_memory);
        println!("=====================================");
    }
    
    /// 验证性能是否符合预期
    pub fn validate(&self, max_avg_duration: Duration, max_p95_duration: Duration) -> bool {
        self.avg_duration <= max_avg_duration && self.p95_duration <= max_p95_duration
    }
}

/// 性能基准管理器
pub struct PerformanceManager {
    benchmarks: HashMap<String, PerformanceBenchmark>,
}

impl PerformanceManager {
    pub fn new() -> Self {
        Self {
            benchmarks: HashMap::new(),
        }
    }
    
    /// 创建新的基准测试
    pub fn create_benchmark(&mut self, name: &str) -> &mut PerformanceBenchmark {
        self.benchmarks.insert(name.to_string(), PerformanceBenchmark::new(name));
        self.benchmarks.get_mut(name).unwrap()
    }
    
    /// 获取基准测试
    pub fn get_benchmark(&mut self, name: &str) -> Option<&mut PerformanceBenchmark> {
        self.benchmarks.get_mut(name)
    }
    
    /// 生成所有报告
    pub fn generate_reports(&self) -> Vec<PerformanceReport> {
        self.benchmarks.values().map(|b| b.report()).collect()
    }
    
    /// 打印所有报告
    pub fn print_all_reports(&self) {
        for report in self.generate_reports() {
            report.print();
        }
    }
}

/// 性能测试宏
#[macro_export]
macro_rules! benchmark {
    ($manager:expr, $name:expr, $code:block) => {
        {
            let benchmark = $manager.create_benchmark($name);
            benchmark.start();
            let result = $code;
            benchmark.end();
            result
        }
    };
}

/// 并发性能测试
pub async fn concurrent_benchmark<F, Fut, T>(
    name: &str,
    task_count: usize,
    task_factory: F,
) -> PerformanceReport
where
    F: Fn() -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    let mut benchmark = PerformanceBenchmark::new(name);
    
    benchmark.start();
    
    let tasks: Vec<_> = (0..task_count)
        .map(|_| tokio::spawn(task_factory()))
        .collect();
    
    // 等待所有任务完成
    for task in tasks {
        let _ = task.await;
    }
    
    benchmark.end();
    benchmark.report()
}

/// 负载测试
pub async fn load_test<F, Fut, T>(
    name: &str,
    duration: Duration,
    concurrent_tasks: usize,
    task_factory: F,
) -> PerformanceReport
where
    F: Fn() -> Fut + Send + Sync + Clone + 'static,
    Fut: std::future::Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    let mut benchmark = PerformanceBenchmark::new(name);
    let start_time = Instant::now();
    
    while start_time.elapsed() < duration {
        benchmark.start();
        
        let tasks: Vec<_> = (0..concurrent_tasks)
            .map(|_| {
                let factory = task_factory.clone();
                tokio::spawn(factory())
            })
            .collect();
        
        // 等待所有任务完成
        for task in tasks {
            let _ = task.await;
        }
        
        benchmark.end();
    }
    
    benchmark.report()
}
