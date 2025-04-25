use futures::future::join_all;
use rand::{thread_rng, Rng};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

// Simulated intensive CPU work
fn process_value(value: u32) -> u32 {
    // Simulate CPU-bound work with some calculations
    let mut result = value;
    for _ in 0..1000 {
        result = result.wrapping_mul(31).wrapping_add(17) % 10000;
    }
    result
}

fn main() {
    const ITERATIONS: usize = 5;
    println!("Starting benchmark with 10,000 values...");
    println!("Running {} iterations for each runtime and showing best result", ITERATIONS);
    
    // Track best times for each framework
    let mut actix_best = Duration::from_secs(u64::MAX);
    let mut tokio_best = Duration::from_secs(u64::MAX);
    let mut async_std_best = Duration::from_secs(u64::MAX);
    let mut smol_best = Duration::from_secs(u64::MAX);
    let mut rayon_best = Duration::from_secs(u64::MAX);
    let mut std_thread_best = Duration::from_secs(u64::MAX);
    let mut crossbeam_best = Duration::from_secs(u64::MAX);
    
    // Track all times for calculating averages
    let mut actix_durations = Vec::with_capacity(ITERATIONS);
    let mut tokio_durations = Vec::with_capacity(ITERATIONS);
    let mut async_std_durations = Vec::with_capacity(ITERATIONS);
    let mut smol_durations = Vec::with_capacity(ITERATIONS);
    let mut rayon_durations = Vec::with_capacity(ITERATIONS);
    let mut std_thread_durations = Vec::with_capacity(ITERATIONS);
    let mut crossbeam_durations = Vec::with_capacity(ITERATIONS);
    
    for i in 0..ITERATIONS {
        println!("\n--- Iteration {} of {} ---", i + 1, ITERATIONS);
        
        // Generate random data
        let mut rng = thread_rng();
        let data: Vec<u32> = (0..10000).map(|_| rng.gen_range(0..10000)).collect();
        let data_arc = Arc::new(data);

        // 1. Benchmark with Actix runtime
        let actix_data = data_arc.clone();
        let start = Instant::now();
        let system = actix_rt::System::new();
        system.block_on(async {
            let results = Arc::new(Mutex::new(vec![0; actix_data.len()]));
            let mut handles = Vec::new();
            
            for (idx, &value) in actix_data.iter().enumerate() {
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
        let actix_duration = start.elapsed();
        actix_durations.push(actix_duration);
        if actix_duration < actix_best {
            actix_best = actix_duration;
        }
        println!("Actix runtime: {:?}", actix_duration);

        // 2. Benchmark with Tokio runtime
        let tokio_data = data_arc.clone();
        let start = Instant::now();
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let results = Arc::new(Mutex::new(vec![0; tokio_data.len()]));
            let mut handles = Vec::new();
            
            for (idx, &value) in tokio_data.iter().enumerate() {
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
        let tokio_duration = start.elapsed();
        tokio_durations.push(tokio_duration);
        if tokio_duration < tokio_best {
            tokio_best = tokio_duration;
        }
        println!("Tokio runtime: {:?}", tokio_duration);

        // 3. Benchmark with async-std
        let async_std_data = data_arc.clone();
        let start = Instant::now();
        async_std::task::block_on(async {
            let results = Arc::new(Mutex::new(vec![0; async_std_data.len()]));
            let mut handles = Vec::new();
            
            for (idx, &value) in async_std_data.iter().enumerate() {
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
        let async_std_duration = start.elapsed();
        async_std_durations.push(async_std_duration);
        if async_std_duration < async_std_best {
            async_std_best = async_std_duration;
        }
        println!("async-std runtime: {:?}", async_std_duration);

        // 4. Benchmark with smol
        let smol_data = data_arc.clone();
        let start = Instant::now();
        smol::block_on(async {
            let results = Arc::new(Mutex::new(vec![0; smol_data.len()]));
            let mut handles = Vec::new();
            
            for (idx, &value) in smol_data.iter().enumerate() {
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
        let smol_duration = start.elapsed();
        smol_durations.push(smol_duration);
        if smol_duration < smol_best {
            smol_best = smol_duration;
        }
        println!("smol runtime: {:?}", smol_duration);

        // 5. Benchmark with Rayon
        let rayon_data = data_arc.clone();
        let start = Instant::now();
        let results = Arc::new(Mutex::new(vec![0; rayon_data.len()]));
        
        rayon::scope(|s| {
            for (idx, &value) in rayon_data.iter().enumerate() {
                let results = results.clone();
                s.spawn(move |_| {
                    let processed = process_value(value);
                    let mut results_guard = results.lock().unwrap();
                    results_guard[idx] = processed;
                });
            }
        });
        
        let rayon_duration = start.elapsed();
        rayon_durations.push(rayon_duration);
        if rayon_duration < rayon_best {
            rayon_best = rayon_duration;
        }
        println!("Rayon: {:?}", rayon_duration);
                // 6. Benchmark with std::thread
                let threads_data = data_arc.clone();
                let start = Instant::now();
                let results = Arc::new(Mutex::new(vec![0; threads_data.len()]));
                let mut handles = Vec::new();
                
                for (idx, &value) in threads_data.iter().enumerate() {
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
                
                let std_thread_duration = start.elapsed();
                std_thread_durations.push(std_thread_duration);
                if std_thread_duration < std_thread_best {
                    std_thread_best = std_thread_duration;
                }
                println!("std::thread: {:?}", std_thread_duration);
                
                // 7. Benchmark with Crossbeam
                let crossbeam_data = data_arc.clone();
                let start = Instant::now();
                let results = Arc::new(Mutex::new(vec![0; crossbeam_data.len()]));
                
                crossbeam::scope(|scope| {
                    for (idx, &value) in crossbeam_data.iter().enumerate() {
                        let results = results.clone();
                        scope.spawn(move |_| {
                            let processed = process_value(value);
                            let mut results_guard = results.lock().unwrap();
                            results_guard[idx] = processed;
                        });
                    }
                }).unwrap();
                
                let crossbeam_duration = start.elapsed();
                crossbeam_durations.push(crossbeam_duration);
                if crossbeam_duration < crossbeam_best {
                    crossbeam_best = crossbeam_duration;
                }
                println!("Crossbeam: {:?}", crossbeam_duration);
            }
        
            // Calculate average durations
            let actix_avg = actix_durations.iter().sum::<Duration>() / actix_durations.len() as u32;
            let tokio_avg = tokio_durations.iter().sum::<Duration>() / tokio_durations.len() as u32;
            let async_std_avg = async_std_durations.iter().sum::<Duration>() / async_std_durations.len() as u32;
            let smol_avg = smol_durations.iter().sum::<Duration>() / smol_durations.len() as u32;
            let rayon_avg = rayon_durations.iter().sum::<Duration>() / rayon_durations.len() as u32;
            let std_thread_avg = std_thread_durations.iter().sum::<Duration>() / std_thread_durations.len() as u32;
            let crossbeam_avg = crossbeam_durations.iter().sum::<Duration>() / crossbeam_durations.len() as u32;
        
            // Determine the overall fastest framework
            let mut frameworks = vec![
                ("Actix", actix_best),
                ("Tokio", tokio_best),
                ("async-std", async_std_best),
                ("smol", smol_best),
                ("Rayon", rayon_best),
                ("std::thread", std_thread_best),
                ("Crossbeam", crossbeam_best)
            ];
            
            frameworks.sort_by_key(|&(_, duration)| duration);
            let fastest = frameworks[0];
        
            // Print summary
            println!("\n=== BENCHMARK RESULTS ===");
            println!("CPU-bound task processing 10,000 values with {} iterations", ITERATIONS);
            println!("\nBest times for each framework:");
            println!("--------------------------------");
            println!("Actix:       {:?}", actix_best);
            println!("Tokio:       {:?}", tokio_best);
            println!("async-std:   {:?}", async_std_best);
            println!("smol:        {:?}", smol_best);
            println!("Rayon:       {:?}", rayon_best);
            println!("std::thread: {:?}", std_thread_best);
            println!("Crossbeam:   {:?}", crossbeam_best);
            
            println!("\nAverage times for each framework:");
            println!("--------------------------------");
            println!("Actix:       {:?}", actix_avg);
            println!("Tokio:       {:?}", tokio_avg);
            println!("async-std:   {:?}", async_std_avg);
            println!("smol:        {:?}", smol_avg);
            println!("Rayon:       {:?}", rayon_avg);
            println!("std::thread: {:?}", std_thread_avg);
            println!("Crossbeam:   {:?}", crossbeam_avg);
            
            println!("\n=== SUMMARY ===");
            println!("Fastest framework: {} with {:?}", fastest.0, fastest.1);
            
            // Calculate and display percentage differences
            println!("\nPerformance comparison (percentage slower than the fastest):");
            println!("-------------------------------------------------------");
            for (name, duration) in frameworks.iter() {
                if name != &fastest.0 {
                    let percent_slower = ((duration.as_nanos() as f64 / fastest.1.as_nanos() as f64) - 1.0) * 100.0;
                    println!("{}: {:.2}% slower than {}", name, percent_slower, fastest.0);
                }
            }
        }