// Simple example to test the benchmark functionality
use solunatus::benchmark;

fn main() {
    println!("Running benchmark across all cities...");
    println!();

    let result = benchmark::run_benchmark();

    println!("Benchmark Results:");
    println!("==================");
    println!("Total cities:       {}", result.total_cities);
    println!("Successful:         {}", result.successful);
    println!("Failed:             {}", result.failed);
    println!(
        "Total duration:     {:.2} s",
        result.total_duration_ms as f64 / 1000.0
    );
    println!(
        "Avg per city:       {:.2} ms",
        result.avg_duration_per_city_ms
    );
    println!("Min duration:       {} ms", result.min_duration_ms);
    println!("Max duration:       {} ms", result.max_duration_ms);
    println!(
        "Throughput:         {:.2} cities/sec",
        result.cities_per_second
    );

    if !result.failed_cities.is_empty() {
        println!();
        println!("Failed cities: {}", result.failed_cities.len());
        for failed in result.failed_cities.iter().take(5) {
            println!("  - {}", failed);
        }
        if result.failed_cities.len() > 5 {
            println!("  ... and {} more", result.failed_cities.len() - 5);
        }
    }

    // Generate HTML report
    println!();
    println!("Generating HTML report...");
    let html = benchmark::generate_html_report(&result);
    let filename = "benchmark_test.html";
    if let Err(e) = std::fs::write(filename, html) {
        println!("Error writing report: {}", e);
    } else {
        println!("Report saved to: {}", filename);
    }
}
