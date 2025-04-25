use rand::{thread_rng, Rng};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

// Import the processing function from the async module
use crate::async_check::process_value;

// Results structure to collect benchmark data
pub struct ParallelBenchmarkResult {
    pub library: String,
    pub best_time: Duration,
    pub avg_time: Duration,
    pub all_times: Vec<Duration>,
}

// Main function to benchmark parallel libraries
pub fn benchmark_parallel_libraries(data_size: usize, iterations: usize) -> Vec<ParallelBenchmarkResult> {
    println!("Starting parallel library benchmarks...");
    
    let mut results = Vec::new();
    
    // Benchmark Rayon
    let mut rayon_times = Vec::with_capacity(iterations);
    let mut rayon_best = Duration::from_secs(u64::MAX);

    for _ in 0..iterations {
        // Generate random data
        let mut rng = thread_rng();
        let data: Vec<u32> = (0..data_size).map(|_| rng.gen_range(0..10000)).collect();
        let data_arc = Arc::new(data);
        
        let start = Instant::now();
        let results = Arc::new(Mutex::new(vec![0; data_arc.len()]));
        
        rayon::scope(|s| {
            for (idx, &value) in data_arc.iter().enumerate() {
                let results = results.clone();
                s.spawn(move |_| {
                    let processed = process_value(value);
                    let mut results_guard = results.lock().unwrap();
                    results_guard[idx] = processed;
                });
            }
        });
        
        let duration = start.elapsed();
        if duration < rayon_best {
            rayon_best = duration;
        }
        rayon_times.push(duration);
    }
    
    let rayon_avg = rayon_times.iter().sum::<Duration>() / rayon_times.len() as u32;
    results.push(ParallelBenchmarkResult {
        library: "Rayon".to_string(),
        best_time: rayon_best,
        avg_time: rayon_avg,
        all_times: rayon_times,
    });
    
    // Benchmark std::thread
    let mut std_thread_times = Vec::with_capacity(iterations);
    let mut std_thread_best = Duration::from_secs(u64::MAX);
    
    for _ in 0..iterations {
        let mut rng = thread_rng();
        let data: Vec<u32> = (0..data_size).map(|_| rng.gen_range(0..10000)).collect();
        let data_arc = Arc::new(data);
        
        let start = Instant::now();
        let results = Arc::new(Mutex::new(vec![0; data_arc.len()]));
        let mut handles = Vec::new();
        
        for (idx, &value) in data_arc.iter().enumerate() {
            let results_clone = results.clone();
            let handle = std::thread::spawn(move || {
                let processed = process_value(value);
                let mut results = results_clone.lock().unwrap();
                results[idx] = processed;
            });
            handles.push(handle);
        }
        
        for handle in handles {
            let _ = handle.join().unwrap();
        }
        
        let duration = start.elapsed();
        if duration < std_thread_best {
            std_thread_best = duration;
        }
        std_thread_times.push(duration);
    }
    
    let std_thread_avg = std_thread_times.iter().sum::<Duration>() / std_thread_times.len() as u32;
    results.push(ParallelBenchmarkResult {
        library: "std::thread".to_string(),
        best_time: std_thread_best,
        avg_time: std_thread_avg,
        all_times: std_thread_times,
    });
    
    // Benchmark crossbeam
    let mut crossbeam_times = Vec::with_capacity(iterations);
    let mut crossbeam_best = Duration::from_secs(u64::MAX);
    
    for _ in 0..iterations {
        let mut rng = thread_rng();
        let data: Vec<u32> = (0..data_size).map(|_| rng.gen_range(0..10000)).collect();
        let data_arc = Arc::new(data);
        
        let start = Instant::now();
        let results = Arc::new(Mutex::new(vec![0; data_arc.len()]));
        
        crossbeam::scope(|scope| {
            for (idx, &value) in data_arc.iter().enumerate() {
                let results = results.clone();
                scope.spawn(move |_| {
                    let processed = process_value(value);
                    let mut results_guard = results.lock().unwrap();
                    results_guard[idx] = processed;
                });
            }
        }).unwrap();
        
        let duration = start.elapsed();
        if duration < crossbeam_best {
            crossbeam_best = duration;
        }
        crossbeam_times.push(duration);
    }
    
    let crossbeam_avg = crossbeam_times.iter().sum::<Duration>() / crossbeam_times.len() as u32;
    results.push(ParallelBenchmarkResult {
        library: "Crossbeam".to_string(),
        best_time: crossbeam_best,
        avg_time: crossbeam_avg,
        all_times: crossbeam_times,
    });
    
    println!("Parallel library benchmarks completed.");
    results
}