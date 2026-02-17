//! Performance benchmarking across the city database.
//!
//! This module provides functionality to benchmark astronomical calculations
//! across all cities in the built-in database, measuring performance and
//! identifying any computational failures.
//!
//! # Features
//!
//! - Cycles through all cities in the database
//! - Calculates complete astronomical data (sun/moon positions, events, phases)
//! - Measures individual and aggregate performance metrics
//! - Tracks failures and generates detailed HTML reports
//!
//! # Usage
//!
//! ```no_run
//! use solunatus::benchmark::run_benchmark;
//!
//! let result = run_benchmark();
//! println!("Processed {} cities in {:.2}s",
//!          result.total_cities,
//!          result.total_duration_ms as f64 / 1000.0);
//! ```

use crate::astro::*;
use crate::city::CityDatabase;
use chrono::{DateTime, Datelike, Utc};
use chrono_tz::Tz;
use std::time::Instant;

/// Results from a complete benchmark run across all cities.
///
/// Contains aggregate statistics including timing data, success rates,
/// and detailed failure information.
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    /// Total number of cities processed.
    pub total_cities: usize,
    /// Number of cities that completed successfully.
    pub successful: usize,
    /// Number of cities that failed during calculation.
    pub failed: usize,
    /// Total wall-clock time for the entire benchmark (milliseconds).
    pub total_duration_ms: u128,
    /// Average time per city (milliseconds).
    pub avg_duration_per_city_ms: f64,
    /// Minimum time for any single city (milliseconds).
    pub min_duration_ms: u128,
    /// Maximum time for any single city (milliseconds).
    pub max_duration_ms: u128,
    /// Throughput in cities processed per second.
    pub cities_per_second: f64,
    /// Names of cities that failed, with error messages.
    pub failed_cities: Vec<String>,
}

#[derive(Debug, Clone)]
struct CityBenchmark {
    pub city_name: String,
    pub duration_ms: u128,
    pub success: bool,
    pub error: Option<String>,
}

/// Runs a comprehensive benchmark across all cities in the database.
///
/// This function:
/// - Loads the city database
/// - Iterates through every city
/// - Calculates complete astronomical data (positions, events, phases)
/// - Measures timing for each city
/// - Aggregates statistics and failures
///
/// # Returns
///
/// A [`BenchmarkResult`] containing performance metrics and failure information.
///
/// # Examples
///
/// ```no_run
/// use solunatus::benchmark::run_benchmark;
///
/// let result = run_benchmark();
/// println!("Processed {} cities:", result.total_cities);
/// println!("  Success: {}", result.successful);
/// println!("  Failed: {}", result.failed);
/// println!("  Throughput: {:.2} cities/sec", result.cities_per_second);
/// ```
pub fn run_benchmark() -> BenchmarkResult {
    let db = match CityDatabase::load() {
        Ok(db) => db,
        Err(e) => {
            return BenchmarkResult {
                total_cities: 0,
                successful: 0,
                failed: 1,
                total_duration_ms: 0,
                avg_duration_per_city_ms: 0.0,
                min_duration_ms: 0,
                max_duration_ms: 0,
                cities_per_second: 0.0,
                failed_cities: vec![format!("Failed to load city database: {}", e)],
            };
        }
    };

    let cities = db.cities();
    let total_cities = cities.len();
    let mut results = Vec::with_capacity(total_cities);

    let benchmark_start = Instant::now();

    for city in cities {
        let city_start = Instant::now();

        let result = std::panic::catch_unwind(|| {
            benchmark_city(city.name.as_str(), city.lat, city.lon, &city.tz)
        });

        let duration_ms = city_start.elapsed().as_millis();

        match result {
            Ok(Ok(_)) => {
                results.push(CityBenchmark {
                    city_name: city.name.clone(),
                    duration_ms,
                    success: true,
                    error: None,
                });
            }
            Ok(Err(e)) => {
                results.push(CityBenchmark {
                    city_name: city.name.clone(),
                    duration_ms,
                    success: false,
                    error: Some(e.to_string()),
                });
            }
            Err(_) => {
                results.push(CityBenchmark {
                    city_name: city.name.clone(),
                    duration_ms,
                    success: false,
                    error: Some("Panic occurred".to_string()),
                });
            }
        }
    }

    let total_duration_ms = benchmark_start.elapsed().as_millis();

    // Calculate statistics
    let successful = results.iter().filter(|r| r.success).count();
    let failed = results.iter().filter(|r| !r.success).count();

    let durations: Vec<u128> = results.iter().map(|r| r.duration_ms).collect();
    let min_duration_ms = *durations.iter().min().unwrap_or(&0);
    let max_duration_ms = *durations.iter().max().unwrap_or(&0);

    let avg_duration_per_city_ms = if !results.is_empty() {
        total_duration_ms as f64 / total_cities as f64
    } else {
        0.0
    };

    let cities_per_second = if total_duration_ms > 0 {
        (total_cities as f64 / total_duration_ms as f64) * 1000.0
    } else {
        0.0
    };

    let failed_cities: Vec<String> = results
        .iter()
        .filter(|r| !r.success)
        .map(|r| {
            if let Some(err) = &r.error {
                format!("{}: {}", r.city_name, err)
            } else {
                r.city_name.clone()
            }
        })
        .collect();

    BenchmarkResult {
        total_cities,
        successful,
        failed,
        total_duration_ms,
        avg_duration_per_city_ms,
        min_duration_ms,
        max_duration_ms,
        cities_per_second,
        failed_cities,
    }
}

/// Benchmark a single city by calculating all astronomical data
fn benchmark_city(_name: &str, lat: f64, lon: f64, tz_str: &str) -> anyhow::Result<()> {
    // Parse timezone
    let tz: Tz = tz_str.parse()?;

    // Create location (use default elevation of 0)
    let location =
        Location::new(lat, lon).map_err(|e| anyhow::anyhow!("Invalid location: {}", e))?;

    // Get current time in timezone
    let now_utc: DateTime<Utc> = Utc::now();
    let now_tz = now_utc.with_timezone(&tz);

    // Calculate solar position
    let _solar_pos = sun::solar_position(&location, &now_tz);

    // Calculate lunar position
    let _lunar_pos = moon::lunar_position(&location, &now_tz);

    // Calculate sunrise/sunset
    let _sunrise = sun::solar_event_time(&location, &now_tz, sun::SolarEvent::Sunrise);
    let _sunset = sun::solar_event_time(&location, &now_tz, sun::SolarEvent::Sunset);
    let _solar_noon = sun::solar_noon(&location, &now_tz);

    // Calculate twilight times
    let _civil_dawn = sun::solar_event_time(&location, &now_tz, sun::SolarEvent::CivilDawn);
    let _civil_dusk = sun::solar_event_time(&location, &now_tz, sun::SolarEvent::CivilDusk);
    let _nautical_dawn = sun::solar_event_time(&location, &now_tz, sun::SolarEvent::NauticalDawn);
    let _nautical_dusk = sun::solar_event_time(&location, &now_tz, sun::SolarEvent::NauticalDusk);
    let _astro_dawn = sun::solar_event_time(&location, &now_tz, sun::SolarEvent::AstronomicalDawn);
    let _astro_dusk = sun::solar_event_time(&location, &now_tz, sun::SolarEvent::AstronomicalDusk);

    // Calculate moonrise/moonset
    let _moonrise = moon::lunar_event_time(&location, &now_tz, moon::LunarEvent::Moonrise);
    let _moonset = moon::lunar_event_time(&location, &now_tz, moon::LunarEvent::Moonset);

    // Calculate lunar phases for the month
    let _phases = moon::lunar_phases(now_tz.year(), now_tz.month());

    Ok(())
}

/// Generates an HTML report from benchmark results.
///
/// Creates a styled, self-contained HTML document with:
/// - Summary statistics (cities processed, success rate)
/// - Performance metrics (duration, throughput, min/max times)
/// - List of failed cities with error messages
///
/// The report uses dark mode styling and is designed for readability.
///
/// # Arguments
///
/// * `result` - The benchmark results to format as HTML
///
/// # Returns
///
/// A complete HTML document as a `String`.
///
/// # Examples
///
/// ```no_run
/// use solunatus::benchmark::{run_benchmark, generate_html_report};
/// use std::fs;
///
/// let result = run_benchmark();
/// let html = generate_html_report(&result);
/// fs::write("benchmark_report.html", html).expect("Failed to write report");
/// ```
pub fn generate_html_report(result: &BenchmarkResult) -> String {
    let success_rate = if result.total_cities > 0 {
        (result.successful as f64 / result.total_cities as f64) * 100.0
    } else {
        0.0
    };

    let mut html = String::new();
    html.push_str("<!DOCTYPE html>\n");
    html.push_str("<html>\n");
    html.push_str("<head>\n");
    html.push_str("<meta charset=\"UTF-8\">\n");
    html.push_str("<title>Solunatus Benchmark Report</title>\n");
    html.push_str("<style>\n");
    html.push_str("body { font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, 'Courier New', monospace; max-width: 1200px; margin: 40px auto; padding: 20px; background: #0a0e1a; color: #e0e6f0; }\n");
    html.push_str(
        "h1 { color: #60a5fa; border-bottom: 2px solid #1e40af; padding-bottom: 10px; }\n",
    );
    html.push_str("h2 { color: #34d399; margin-top: 30px; }\n");
    html.push_str(".summary { background: #1e293b; padding: 20px; border-radius: 8px; margin: 20px 0; border: 1px solid #334155; }\n");
    html.push_str(".stat { display: flex; justify-content: space-between; padding: 8px 0; border-bottom: 1px solid #334155; }\n");
    html.push_str(".stat:last-child { border-bottom: none; }\n");
    html.push_str(".label { color: #94a3b8; }\n");
    html.push_str(".value { color: #e0e6f0; font-weight: bold; }\n");
    html.push_str(".success { color: #34d399; }\n");
    html.push_str(".failure { color: #f87171; }\n");
    html.push_str(".warning { background: #451a03; border: 1px solid #ea580c; padding: 15px; border-radius: 8px; margin: 20px 0; }\n");
    html.push_str(".warning h3 { color: #fb923c; margin-top: 0; }\n");
    html.push_str(".error-list { background: #1e293b; padding: 10px; border-radius: 4px; max-height: 400px; overflow-y: auto; }\n");
    html.push_str(".error-item { padding: 5px 0; font-size: 0.9em; color: #f87171; }\n");
    html.push_str("</style>\n");
    html.push_str("</head>\n");
    html.push_str("<body>\n");

    html.push_str("<h1>🚀 Solunatus Benchmark Report</h1>\n");

    html.push_str("<div class=\"summary\">\n");
    html.push_str("<h2>Summary</h2>\n");

    html.push_str(&format!("<div class=\"stat\"><span class=\"label\">Total Cities</span><span class=\"value\">{}</span></div>\n", result.total_cities));
    html.push_str(&format!("<div class=\"stat\"><span class=\"label\">Successful</span><span class=\"value success\">{}</span></div>\n", result.successful));
    html.push_str(&format!("<div class=\"stat\"><span class=\"label\">Failed</span><span class=\"value failure\">{}</span></div>\n", result.failed));
    html.push_str(&format!("<div class=\"stat\"><span class=\"label\">Success Rate</span><span class=\"value success\">{:.2}%</span></div>\n", success_rate));
    html.push_str("</div>\n");

    html.push_str("<div class=\"summary\">\n");
    html.push_str("<h2>Performance</h2>\n");
    html.push_str(&format!("<div class=\"stat\"><span class=\"label\">Total Duration</span><span class=\"value\">{:.2} s</span></div>\n", result.total_duration_ms as f64 / 1000.0));
    html.push_str(&format!("<div class=\"stat\"><span class=\"label\">Average per City</span><span class=\"value\">{:.2} ms</span></div>\n", result.avg_duration_per_city_ms));
    html.push_str(&format!("<div class=\"stat\"><span class=\"label\">Min Duration</span><span class=\"value\">{} ms</span></div>\n", result.min_duration_ms));
    html.push_str(&format!("<div class=\"stat\"><span class=\"label\">Max Duration</span><span class=\"value\">{} ms</span></div>\n", result.max_duration_ms));
    html.push_str(&format!("<div class=\"stat\"><span class=\"label\">Throughput</span><span class=\"value\">{:.2} cities/sec</span></div>\n", result.cities_per_second));
    html.push_str("</div>\n");

    if !result.failed_cities.is_empty() {
        html.push_str("<div class=\"warning\">\n");
        html.push_str("<h3>⚠️ Failed Cities</h3>\n");
        html.push_str("<div class=\"error-list\">\n");
        for error in &result.failed_cities {
            html.push_str(&format!("<div class=\"error-item\">{}</div>\n", error));
        }
        html.push_str("</div>\n");
        html.push_str("</div>\n");
    }

    html.push_str("</body>\n");
    html.push_str("</html>\n");

    html
}
