use rand::{thread_rng, Rng};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use flume;
use nalgebra as na;
use num_cpus;

// Import the processing function from the async module
use crate::async_check::process_value;

// Results structure to collect benchmark data
pub struct HybridBenchmarkResult {
    pub library: String,
    pub best_time: Duration,
    pub avg_time: Duration,
    pub all_times: Vec<Duration>,
}

// Main function to benchmark hybrid libraries
pub fn benchmark_hybrid_libraries(data_size: usize, iterations: usize) -> Vec<HybridBenchmarkResult> {
    println!("Starting hybrid library benchmarks...");
    
    let mut results = Vec::new();
    
    // Benchmark actix-rt
    let mut actix_times = Vec::with_capacity(iterations);
    let mut actix_best = Duration::from_secs(u64::MAX);
    
    for _ in 0..iterations {
        // Generate random data
        let mut rng = thread_rng();
        let data: Vec<u32> = (0..data_size).map(|_| rng.gen_range(0..10000)).collect();
        let data_arc = Arc::new(data);
        
        let start = Instant::now();
        let system = actix_rt::System::new();
        system.block_on(async {
            let results = Arc::new(Mutex::new(vec![0; data_arc.len()]));
            let mut handles = Vec::new();
            
            for (idx, &value) in data_arc.iter().enumerate() {
                let results_clone = results.clone();
                let handle = actix_rt::spawn(async move {
                    let processed = process_value(value);
                    let mut results = results_clone.lock().unwrap();
                    results[idx] = processed;
                });
                handles.push(handle);
            }
            
            for handle in handles {
                let _ = handle.await;
            }
        });
        
        let duration = start.elapsed();
        if duration < actix_best {
            actix_best = duration;
        }
        actix_times.push(duration);
    }
    
    let actix_avg = actix_times.iter().sum::<Duration>() / actix_times.len() as u32;
    results.push(HybridBenchmarkResult {
        library: "Actix".to_string(),
        best_time: actix_best,
        avg_time: actix_avg,
        all_times: actix_times,
    });
    
    // Benchmark tokio + rayon hybrid approach
    let mut tokio_rayon_times = Vec::with_capacity(iterations);
    let mut tokio_rayon_best = Duration::from_secs(u64::MAX);
    
    for _ in 0..iterations {
        let mut rng = thread_rng();
        let data: Vec<u32> = (0..data_size).map(|_| rng.gen_range(0..10000)).collect();
        let data_arc = Arc::new(data);
        
        let start = Instant::now();
        let runtime = tokio::runtime::Runtime::new().unwrap();
        
        runtime.block_on(async {
            let results = Arc::new(Mutex::new(vec![0; data_arc.len()]));
            
            // Use tokio for task management but process in parallel using rayon
            let chunks: Vec<_> = data_arc
                .chunks(data_arc.len() / num_cpus::get().max(1))
                .collect();
                
            let mut handles = Vec::new();
            
            for (chunk_idx, chunk) in chunks.iter().enumerate() {
                let chunk_data = chunk.to_vec();
                let results_clone = results.clone();
                
                let handle = tokio::spawn(async move {
                    // Process this chunk with rayon
                    let offset = chunk_idx * chunk_data.len();
                    
                    rayon::scope(|s| {
                        for (i, &value) in chunk_data.iter().enumerate() {
                            let results = results_clone.clone();
                            let idx = offset + i;
                            s.spawn(move |_| {
                                let processed = process_value(value);
                                let mut results_guard = results.lock().unwrap();
                                results_guard[idx] = processed;
                            });
                        }
                    });
                });
                
                handles.push(handle);
            }
            
            for handle in handles {
                let _ = handle.await.unwrap();
            }
        });
        
        let duration = start.elapsed();
        if duration < tokio_rayon_best {
            tokio_rayon_best = duration;
        }
        tokio_rayon_times.push(duration);
    }
    
    let tokio_rayon_avg = tokio_rayon_times.iter().sum::<Duration>() / tokio_rayon_times.len() as u32;
    results.push(HybridBenchmarkResult {
        library: "Tokio+Rayon".to_string(),
        best_time: tokio_rayon_best,
        avg_time: tokio_rayon_avg,
        all_times: tokio_rayon_times,
    });
    
    // Benchmark async-std + crossbeam
    let mut async_std_crossbeam_times = Vec::with_capacity(iterations);
    let mut async_std_crossbeam_best = Duration::from_secs(u64::MAX);
    
    for _ in 0..iterations {
        let mut rng = thread_rng();
        let data: Vec<u32> = (0..data_size).map(|_| rng.gen_range(0..10000)).collect();
        let data_arc = Arc::new(data);
        
        let start = Instant::now();
        
        async_std::task::block_on(async {
            let results = Arc::new(Mutex::new(vec![0; data_arc.len()]));
            
            // Split data into chunks for processing
            let chunks: Vec<_> = data_arc
                .chunks(data_arc.len() / num_cpus::get().max(1))
                .collect();
                
            let mut handles = Vec::new();
            
            for (chunk_idx, chunk) in chunks.iter().enumerate() {
                let chunk_data = chunk.to_vec();
                let results_clone = results.clone();
                
                let handle = async_std::task::spawn(async move {
                    // Process this chunk with crossbeam
                    let offset = chunk_idx * chunk_data.len();
                    
                    crossbeam::scope(|s| {
                        for (i, &value) in chunk_data.iter().enumerate() {
                            let results = results_clone.clone();
                            let idx = offset + i;
                            s.spawn(move |_| {
                                let processed = process_value(value);
                                let mut results_guard = results.lock().unwrap();
                                results_guard[idx] = processed;
                            });
                        }
                    }).unwrap();
                });
                
                handles.push(handle);
            }
            
            for handle in handles {
                handle.await;
            }
        });
        
        let duration = start.elapsed();
        if duration < async_std_crossbeam_best {
            async_std_crossbeam_best = duration;
        }
        async_std_crossbeam_times.push(duration);
    }
    let async_std_crossbeam_avg = async_std_crossbeam_times.iter().sum::<Duration>() / async_std_crossbeam_times.len() as u32;
    results.push(HybridBenchmarkResult {
        library: "async-std+Crossbeam".to_string(),
        best_time: async_std_crossbeam_best,
        avg_time: async_std_crossbeam_avg,
        all_times: async_std_crossbeam_times,
    });
    
    // Benchmark using flume (MPMC channels)
    let mut flume_times = Vec::with_capacity(iterations);
    let mut flume_best = Duration::from_secs(u64::MAX);
    
    for _ in 0..iterations {
        let mut rng = thread_rng();
        let data: Vec<u32> = (0..data_size).map(|_| rng.gen_range(0..10000)).collect();
        let data_arc = Arc::new(data);
        
        let start = Instant::now();
        
        // Create the channels
        let (work_sender, work_receiver) = flume::unbounded();
        let (result_sender, result_receiver) = flume::unbounded();
        let results = Arc::new(Mutex::new(vec![0; data_arc.len()]));
        
        // Spawn worker threads
        let num_threads = num_cpus::get();
        let mut handles = Vec::new();
        
        for _ in 0..num_threads {
            let receiver = work_receiver.clone();
            let sender = result_sender.clone();
            let handle = std::thread::spawn(move || {
                while let Ok((idx, value)) = receiver.recv() {
                    let processed = process_value(value);
                    sender.send((idx, processed)).unwrap();
                }
            });
            handles.push(handle);
        }
        
        // Send work
        for (idx, &value) in data_arc.iter().enumerate() {
            work_sender.send((idx, value)).unwrap();
        }
        
        // Signal that there's no more work
        drop(work_sender);
        drop(result_sender);
        
        // Collect results
        let results_ref = results.clone();
        let collector_handle = std::thread::spawn(move || {
            let mut remaining = data_arc.len();
            while remaining > 0 {
                if let Ok((idx, result)) = result_receiver.recv() {
                    let mut results = results_ref.lock().unwrap();
                    results[idx] = result;
                    remaining -= 1;
                } else {
                    break;
                }
            }
        });
        
        // Wait for all workers to finish
        for handle in handles {
            handle.join().unwrap();
        }
        
        // Wait for collector
        collector_handle.join().unwrap();
        
        let duration = start.elapsed();
        if duration < flume_best {
            flume_best = duration;
        }
        flume_times.push(duration);
    }
    
    let flume_avg = flume_times.iter().sum::<Duration>() / flume_times.len() as u32;
    results.push(HybridBenchmarkResult {
        library: "Flume".to_string(),
        best_time: flume_best,
        avg_time: flume_avg,
        all_times: flume_times,
    });
    
    // Benchmark using nalgebra for matrix operations
    let mut nalgebra_times = Vec::with_capacity(iterations);
    let mut nalgebra_best = Duration::from_secs(u64::MAX);
    
    for _ in 0..iterations {
        let mut rng = thread_rng();
        
        // For nalgebra, let's create a square matrix of approximately the right size
        let matrix_size = (data_size as f64).sqrt() as usize;
        let matrix_size_squared = matrix_size * matrix_size;
        
        let values: Vec<f32> = (0..matrix_size_squared)
            .map(|_| rng.gen_range(0..10000) as f32)
            .collect();
        
        // Create a nalgebra matrix
        let matrix = na::DMatrix::<f32>::from_vec(matrix_size, matrix_size, values);
        
        let start = Instant::now();
        
        // Perform parallel computation with nalgebra
        let results = Arc::new(Mutex::new(vec![0; matrix_size_squared]));
        
        // Use tokio runtime for task management with nalgebra
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let chunk_size = matrix_size / num_cpus::get().max(1);
            let mut handles = Vec::new();
            
            for i in 0..num_cpus::get().max(1) {
                let start_row = i * chunk_size;
                let end_row = if i == num_cpus::get().max(1) - 1 {
                    matrix_size
                } else {
                    (i + 1) * chunk_size
                };
                
                let matrix_slice = matrix.clone();
                let results_clone = results.clone();
                
                let handle = tokio::spawn(async move {
                    for row in start_row..end_row {
                        for col in 0..matrix_size {
                            let value = matrix_slice[(row, col)] as u32;
                            let processed = process_value(value);
                            let idx = row * matrix_size + col;
                            let mut results_guard = results_clone.lock().unwrap();
                            results_guard[idx] = processed;
                        }
                    }
                });
                
                handles.push(handle);
            }
            
            for handle in handles {
                let _ = handle.await.unwrap();
            }
        });
        
        let duration = start.elapsed();
        if duration < nalgebra_best {
            nalgebra_best = duration;
        }
        nalgebra_times.push(duration);
    }
    
    let nalgebra_avg = nalgebra_times.iter().sum::<Duration>() / nalgebra_times.len() as u32;
    results.push(HybridBenchmarkResult {
        library: "Nalgebra+Tokio".to_string(),
        best_time: nalgebra_best,
        avg_time: nalgebra_avg,
        all_times: nalgebra_times,
    });
    
    // Benchmark with async-graphql-inspired worker pool
    // Note: We're not actually using async-graphql, just implementing a similar pattern
    let mut async_graphql_pattern_times = Vec::with_capacity(iterations);
    let mut async_graphql_pattern_best = Duration::from_secs(u64::MAX);
    
    for _ in 0..iterations {
        let mut rng = thread_rng();
        let data: Vec<u32> = (0..data_size).map(|_| rng.gen_range(0..10000)).collect();
        let data_arc = Arc::new(data);
        
        let start = Instant::now();
        
        // Similar to how async-graphql handles parallel execution
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let results = Arc::new(Mutex::new(vec![0; data_arc.len()]));
            let semaphore = Arc::new(tokio::sync::Semaphore::new(num_cpus::get()));
            
            // Process in batches of futures
            let mut all_futures = Vec::new();
            
            for (idx, &value) in data_arc.iter().enumerate() {
                let results_clone = results.clone();
                let semaphore_clone = semaphore.clone();
                
                all_futures.push(async move {
                    let _permit = semaphore_clone.acquire().await.unwrap();
                    
                    // Simulate some CPU-intensive work
                    let processed = process_value(value);
                    
                    let mut results_guard = results_clone.lock().unwrap();
                    results_guard[idx] = processed;
                });
            }
            
            // Execute all tasks in a way similar to async-graphql's parallel execution model
            futures::future::join_all(all_futures).await;
        });
        
        let duration = start.elapsed();
        if duration < async_graphql_pattern_best {
            async_graphql_pattern_best = duration;
        }
        async_graphql_pattern_times.push(duration);
    }
    
    let async_graphql_pattern_avg = async_graphql_pattern_times.iter().sum::<Duration>() / async_graphql_pattern_times.len() as u32;
    results.push(HybridBenchmarkResult {
        library: "AsyncGraphQL-pattern".to_string(),
        best_time: async_graphql_pattern_best,
        avg_time: async_graphql_pattern_avg,
        all_times: async_graphql_pattern_times,
    });
    
// Replace the problematic WGPU pattern benchmark with this corrected version:
let mut wgpu_pattern_times = Vec::with_capacity(iterations);
let mut wgpu_pattern_best = Duration::from_secs(u64::MAX);

for _ in 0..iterations {
    let mut rng = thread_rng();
    let data: Vec<u32> = (0..data_size).map(|_| rng.gen_range(0..10000)).collect();
    let data_arc = Arc::new(data);
    
    let start = Instant::now();
    
    // Simulate wgpu-like batch processing approach
    const WORKGROUP_SIZE: usize = 256; // Common workgroup size for GPU computation
    let results = Arc::new(Mutex::new(vec![0; data_arc.len()]));
    
    // Process data in batches similar to how GPU compute shaders would
    rayon::scope(|s| {
        for chunk_idx in 0..(data_arc.len() + WORKGROUP_SIZE - 1) / WORKGROUP_SIZE {
            let start_idx = chunk_idx * WORKGROUP_SIZE;
            let end_idx = (start_idx + WORKGROUP_SIZE).min(data_arc.len());
            let data_arc_clone = data_arc.clone(); // Clone for each workgroup
            let results_clone = results.clone(); // Clone for each workgroup
            
            s.spawn(move |_| {
                // Process all items in this "workgroup" in parallel
                for i in start_idx..end_idx {
                    let processed = process_value(data_arc_clone[i]);
                    let mut results_guard = results_clone.lock().unwrap();
                    results_guard[i] = processed;
                }
            });
        }
    });
    
    let duration = start.elapsed();
    if duration < wgpu_pattern_best {
        wgpu_pattern_best = duration;
    }
    wgpu_pattern_times.push(duration);
}

let wgpu_pattern_avg = wgpu_pattern_times.iter().sum::<Duration>() / wgpu_pattern_times.len() as u32;
results.push(HybridBenchmarkResult {
    library: "WGPU-pattern".to_string(),
    best_time: wgpu_pattern_best,
    avg_time: wgpu_pattern_avg,
    all_times: wgpu_pattern_times,
});

    
    println!("Hybrid library benchmarks completed.");
    results
}
    