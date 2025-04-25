use futures::future::join_all;
use rand::{thread_rng, Rng};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

// Shared CPU-bound work function
pub fn process_value(value: u32) -> u32 {
    // Simulate CPU-bound work with some calculations
    let mut result = value;
    for _ in 0..1000 {
        result = result.wrapping_mul(31).wrapping_add(17) % 10000;
    }
    result
}

// Results structure to collect benchmark data
pub struct AsyncBenchmarkResult {
    pub library: String,
    pub best_time: Duration,
    pub avg_time: Duration,
    pub all_times: Vec<Duration>,
}

// Main function to benchmark async libraries
pub async fn benchmark_async_libraries(data_size: usize, iterations: usize) -> Vec<AsyncBenchmarkResult> {
    println!("Starting async library benchmarks...");
    
    let mut results = Vec::new();
    
    // Benchmark Tokio
    let mut tokio_times = Vec::with_capacity(iterations);
    let mut tokio_best = Duration::from_secs(u64::MAX);
    
    for _ in 0..iterations {
        // Generate random data
        let mut rng = thread_rng();
        let data: Vec<u32> = (0..data_size).map(|_| rng.gen_range(0..10000)).collect();
        let data_arc = Arc::new(data);
        
        let start = Instant::now();
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let results = Arc::new(Mutex::new(vec![0; data_arc.len()]));
            let mut handles = Vec::new();
            
            for (idx, &value) in data_arc.iter().enumerate() {
                let results_clone = results.clone();
                let handle = tokio::spawn(async move {
                    let processed = process_value(value);
                    let mut results = results_clone.lock().unwrap();
                    results[idx] = processed;
                });
                handles.push(handle);
            }
            
            for handle in handles {
                let _ = handle.await.unwrap();
            }
        });
        
        let duration = start.elapsed();
        if duration < tokio_best {
            tokio_best = duration;
        }
        tokio_times.push(duration);
    }
    
    let tokio_avg = tokio_times.iter().sum::<Duration>() / tokio_times.len() as u32;
    results.push(AsyncBenchmarkResult {
        library: "Tokio".to_string(),
        best_time: tokio_best,
        avg_time: tokio_avg,
        all_times: tokio_times,
    });
    
    // Benchmark async-std
    let mut async_std_times = Vec::with_capacity(iterations);
    let mut async_std_best = Duration::from_secs(u64::MAX);
    
    for _ in 0..iterations {
        let mut rng = thread_rng();
        let data: Vec<u32> = (0..data_size).map(|_| rng.gen_range(0..10000)).collect();
        let data_arc = Arc::new(data);
        
        let start = Instant::now();
        async_std::task::block_on(async {
            let results = Arc::new(Mutex::new(vec![0; data_arc.len()]));
            let mut handles = Vec::new();
            
            for (idx, &value) in data_arc.iter().enumerate() {
                let results_clone = results.clone();
                let handle = async_std::task::spawn(async move {
                    let processed = process_value(value);
                    let mut results = results_clone.lock().unwrap();
                    results[idx] = processed;
                });
                handles.push(handle);
            }
            
            for handle in handles {
                handle.await;
            }
        });
        
        let duration = start.elapsed();
        if duration < async_std_best {
            async_std_best = duration;
        }
        async_std_times.push(duration);
    }
    
    let async_std_avg = async_std_times.iter().sum::<Duration>() / async_std_times.len() as u32;
    results.push(AsyncBenchmarkResult {
        library: "async-std".to_string(),
        best_time: async_std_best,
        avg_time: async_std_avg,
        all_times: async_std_times,
    });
    
    // Benchmark smol
    let mut smol_times = Vec::with_capacity(iterations);
    let mut smol_best = Duration::from_secs(u64::MAX);
    
    for _ in 0..iterations {
        let mut rng = thread_rng();
        let data: Vec<u32> = (0..data_size).map(|_| rng.gen_range(0..10000)).collect();
        let data_arc = Arc::new(data);
        
        let start = Instant::now();
        smol::block_on(async {
            let results = Arc::new(Mutex::new(vec![0; data_arc.len()]));
            let mut handles = Vec::new();
            
            for (idx, &value) in data_arc.iter().enumerate() {
                let results_clone = results.clone();
                let handle = smol::spawn(async move {
                    let processed = process_value(value);
                    let mut results = results_clone.lock().unwrap();
                    results[idx] = processed;
                });
                handles.push(handle);
            }
            
            for handle in handles {
                handle.await;
            }
        });
        
        let duration = start.elapsed();
        if duration < smol_best {
            smol_best = duration;
        }
        smol_times.push(duration);
    }
    
    let smol_avg = smol_times.iter().sum::<Duration>() / smol_times.len() as u32;
    results.push(AsyncBenchmarkResult {
        library: "smol".to_string(),
        best_time: smol_best,
        avg_time: smol_avg,
        all_times: smol_times,
    });
    
    println!("Async library benchmarks completed.");
    results
}