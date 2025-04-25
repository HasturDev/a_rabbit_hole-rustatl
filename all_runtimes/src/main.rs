mod async_check;
mod parallel_check; 
mod hybrid_check;

use crate::async_check::{benchmark_async_libraries, AsyncBenchmarkResult};
use crate::parallel_check::{benchmark_parallel_libraries, ParallelBenchmarkResult};
use crate::hybrid_check::{benchmark_hybrid_libraries, HybridBenchmarkResult};
use std::time::Duration;

// Generic benchmark result for unified processing
#[derive(Clone)]  // Add Clone trait
struct BenchmarkResult {
    category: String,
    library: String,
    best_time: Duration,
    avg_time: Duration,
    all_times: Vec<Duration>,
}

fn main() {
    const DATA_SIZE: usize = 10000;
    const ITERATIONS: usize = 5;
    
    println!("=== RUST CONCURRENCY LIBRARIES BENCHMARK ===");
    println!("Benchmarking with {} data points, {} iterations each", DATA_SIZE, ITERATIONS);
    println!("--------------------------------------------------------");
    
    // Run all the benchmarks
    let async_results = async_std::task::block_on(
        benchmark_async_libraries(DATA_SIZE, ITERATIONS)
    );
    
    let parallel_results = benchmark_parallel_libraries(DATA_SIZE, ITERATIONS);
    let hybrid_results = benchmark_hybrid_libraries(DATA_SIZE, ITERATIONS);
    
    // Combine all results for analysis
    let mut all_results: Vec<BenchmarkResult> = Vec::new();
    
    // Process async results
    for result in async_results {
        all_results.push(BenchmarkResult {
            category: "Asynchronous".to_string(),
            library: result.library,
            best_time: result.best_time,
            avg_time: result.avg_time,
            all_times: result.all_times,
        });
    }
    
    // Process parallel results
    for result in parallel_results {
        all_results.push(BenchmarkResult {
            category: "Parallel".to_string(),
            library: result.library,
            best_time: result.best_time,
            avg_time: result.avg_time,
            all_times: result.all_times,
        });
    }
    
    // Process hybrid results
    for result in hybrid_results {
        all_results.push(BenchmarkResult {
            category: "Hybrid".to_string(),
            library: result.library,
            best_time: result.best_time,
            avg_time: result.avg_time,
            all_times: result.all_times,
        });
    }
    
    // Find overall best performer
    all_results.sort_by_key(|r| r.best_time);
    let overall_best = &all_results[0];
    
    println!("\n=== OVERALL RESULTS ===");
    println!("Best overall performer: {} ({}) with {:?}", 
             overall_best.library, overall_best.category, overall_best.best_time);
    println!("--------------------------------------------------------");
    
    // Calculate percentages relative to the best performer
    let best_time_nanos = overall_best.best_time.as_nanos() as f64;
    
    // Output average times
    println!("\n=== AVERAGE TIMES ===");
    println!("{:<20} {:<20} {:<15} {:<15}", "Category", "Library", "Avg Time", "vs Best (%)");
    println!("{:-<75}", "");
    
    let mut sorted_by_avg = all_results.clone();
    sorted_by_avg.sort_by_key(|r| r.avg_time);
    
    for result in &sorted_by_avg {
        let percent_slower = ((result.avg_time.as_nanos() as f64 / best_time_nanos) - 1.0) * 100.0;
        println!("{:<20} {:<20} {:<15?} {:<15.2}%", 
                 result.category, result.library, result.avg_time, percent_slower);
    }
        
        // Output best times
        println!("\n=== BEST TIMES ===");
        println!("{:<20} {:<20} {:<15} {:<15}", "Category", "Library", "Best Time", "vs Best (%)");
        println!("{:-<75}", "");
        
        for result in &all_results {  // already sorted by best_time
            let percent_slower = ((result.best_time.as_nanos() as f64 / best_time_nanos) - 1.0) * 100.0;
            println!("{:<20} {:<20} {:<15?} {:<15.2}%", 
                     result.category, result.library, result.best_time, percent_slower);
        }
        
        // Group results by category
        println!("\n=== RESULTS BY CATEGORY ===");
        
        // Asynchronous category
        println!("\n--- ASYNCHRONOUS LIBRARIES ---");
        println!("{:<20} {:<15} {:<15} {:<15}", "Library", "Best Time", "Avg Time", "vs Category Best (%)");
        println!("{:-<70}", "");
        
        let mut async_libs: Vec<&BenchmarkResult> = all_results.iter()
            .filter(|r| r.category == "Asynchronous")
            .collect();
        async_libs.sort_by_key(|r| r.best_time);
        
        let async_best_time = if !async_libs.is_empty() {
            async_libs[0].best_time.as_nanos() as f64
        } else {
            0.0
        };
        
        for result in async_libs {
            let percent_vs_category_best = ((result.best_time.as_nanos() as f64 / async_best_time) - 1.0) * 100.0;
            println!("{:<20} {:<15?} {:<15?} {:<15.2}%", 
                     result.library, result.best_time, result.avg_time, percent_vs_category_best);
        }
        
        // Parallel category
        println!("\n--- PARALLEL LIBRARIES ---");
        println!("{:<20} {:<15} {:<15} {:<15}", "Library", "Best Time", "Avg Time", "vs Category Best (%)");
        println!("{:-<70}", "");
        
        let mut parallel_libs: Vec<&BenchmarkResult> = all_results.iter()
            .filter(|r| r.category == "Parallel")
            .collect();
        parallel_libs.sort_by_key(|r| r.best_time);
        
        let parallel_best_time = if !parallel_libs.is_empty() {
            parallel_libs[0].best_time.as_nanos() as f64
        } else {
            0.0
        };
        
        for result in parallel_libs {
            let percent_vs_category_best = ((result.best_time.as_nanos() as f64 / parallel_best_time) - 1.0) * 100.0;
            println!("{:<20} {:<15?} {:<15?} {:<15.2}%", 
                     result.library, result.best_time, result.avg_time, percent_vs_category_best);
        }
        
        // Hybrid category
        println!("\n--- HYBRID LIBRARIES ---");
        println!("{:<20} {:<15} {:<15} {:<15}", "Library", "Best Time", "Avg Time", "vs Category Best (%)");
        println!("{:-<70}", "");
        
        let mut hybrid_libs: Vec<&BenchmarkResult> = all_results.iter()
            .filter(|r| r.category == "Hybrid")
            .collect();
        hybrid_libs.sort_by_key(|r| r.best_time);
        
        let hybrid_best_time = if !hybrid_libs.is_empty() {
            hybrid_libs[0].best_time.as_nanos() as f64
        } else {
            0.0
        };
        
        for result in hybrid_libs {
            let percent_vs_category_best = ((result.best_time.as_nanos() as f64 / hybrid_best_time) - 1.0) * 100.0;
            println!("{:<20} {:<15?} {:<15?} {:<15.2}%", 
                     result.library, result.best_time, result.avg_time, percent_vs_category_best);
        }
        
        println!("\n=== BENCHMARK COMPLETE ===");
        println!("Note: These results are specific to CPU-bound workloads. Different workload types (e.g., I/O-bound) may yield different results.");
    }