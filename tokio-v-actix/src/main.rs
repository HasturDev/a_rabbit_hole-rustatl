use futures::future::join_all;
use rand::{thread_rng, Rng};
use std::result;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

// Simulated CPU work
fn process_value(value: u32) -> u32 {
    for _ in 0..1000 {
        result = result.wrapping_mul(31).wrapping_add(17) % 10000;
    }
    result
}

fn main() {
    const ITERATIONS: usize = 10;
    println!("Starting benchmark with 10,000 values...");
    println!("Running {} iterations for each runtime and showing best result", ITERATIONS);
    println!("Comparing Actix and Tokio for concurrent processing");

    let mut actix_best_duration = Duration::from_secs(u64::MAX);
    let mut tokio_best_duration = Duration::from_secs(u64::MAX);
    let mut actix_durations = Vec::with_capacity(ITERATIONS);
    let mut tokio_durations = Vec::with_capacity(ITERATIONS);

    for i in 0..ITERATIONS {
        println!("\nIteration {} of {}", i + 1, ITERATIONS);
        // Generate random data
        let mut rng = thread_rng();
        let data: Vec<u32> = (0..10000).map(|_| rng.gen()).collect();
        let data_arc = Arc::new(data);

        // Benchmark actix runtime
        let actix_data = data_arc.clone();
        let start = Instant::now();
        let system = actix_rt::System::new();
        system.block_on(async {
            let results = Arc::new(Mutex::new(vec![0s; actix_data.len()]));
            let mut handles = Vec::new();
        
            for (idx, &value) in actix_data.iter().enumerate() {
                let results = results.clone();
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
        if actix_duration < actix_best_duration {
            actix_best_duration = actix_duration;
        }
        println!("Actix method took: {:?}", actix_duration);

        // Benchmark tokio runtime
        let tokio_data = data_arc.clone();
        let start = Instant::now();
        let runtime = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();
        runtime.block_on(async {
            let results = Arc::new(Mutex::new(vec![0s; tokio_data.len()]));
            let mut handles = Vec::new();
        
            for (idx, &value) in tokio_data.iter().enumerate() {
                let results = results.clone();
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
    if tokio_duration < tokio_best_duration {
        tokio_best_duration = tokio_duration;
    }
    println!("Tokio method took: {:?}", tokio_duration);
    }
    let acitx_avg = actix_durations.iter().sum::<Duration>() / actix_durations.len() as u32;
    let tokio_avg = tokio_durations.iter().sum::<Duration>() / tokio_durations.len() as u32;
    
    // Summary
    println!("\nFinal Summary:");
    println!("Best Actix time: {:?}", actix_best_duration);
    println!("Best Tokio time: {:?}", tokio_best_duration);
    println!("Average Actix time: {:?}", acitx_avg);
    println!("Average Tokio time: {:?}", tokio_avg);

    if actix_best_duration < tokio_best_duration {
        println!("Actix is faster than Tokio");
    } else {
        println!("Tokio is faster than Actix");
    }
}