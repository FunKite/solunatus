//! Batch processing example: Calculate data for multiple dates efficiently
//!
//! Run with:
//! ```bash
//! cargo run --example batch_processing --release
//! ```

use chrono::{Duration, Local};
use chrono_tz::America::New_York;
use solunatus::prelude::*;
use std::time::Instant;

fn main() {
    println!("=== Solunatus Library - Batch Processing ===\n");

    // Location: New York City
    let location = Location::new(40.7128, -74.0060).expect("Invalid coordinates");

    // Generate 30 days of dates
    let start_date = Local::now().with_timezone(&New_York);
    let dates: Vec<_> = (0..30).map(|i| start_date + Duration::days(i)).collect();

    println!(
        "Calculating astronomical data for {} days...\n",
        dates.len()
    );

    // Measure performance
    let start = Instant::now();
    let results = batch_calculate(&location, &dates);
    let elapsed = start.elapsed();

    println!(
        "✓ Calculations completed in {:.2}ms",
        elapsed.as_secs_f64() * 1000.0
    );
    println!(
        "  ({:.2}ms per day)\n",
        elapsed.as_secs_f64() * 1000.0 / dates.len() as f64
    );

    // Print results in table format
    println!("Date         Sunrise  Sunset   Day Length  Moon Phase");
    println!("────────────────────────────────────────────────────────────");

    for result in results.iter().take(30) {
        let date_str = result.date.format("%Y-%m-%d");

        let sunrise_str = result
            .sunrise
            .map(|t| t.format("%H:%M").to_string())
            .unwrap_or_else(|| "--:--".to_string());

        let sunset_str = result
            .sunset
            .map(|t| t.format("%H:%M").to_string())
            .unwrap_or_else(|| "--:--".to_string());

        let day_length = if let (Some(rise), Some(set)) = (result.sunrise, result.sunset) {
            let duration = set.signed_duration_since(rise);
            format!(
                "{:02}h {:02}m",
                duration.num_hours(),
                duration.num_minutes() % 60
            )
        } else {
            "N/A".to_string()
        };

        let illum_pct = (result.moon_position.illumination * 100.0) as u8;
        let moon_bar = "█".repeat((illum_pct / 10) as usize);

        println!(
            "{} {} {} {:>11}  {}% {}",
            date_str, sunrise_str, sunset_str, day_length, illum_pct, moon_bar
        );
    }

    // Calculate statistics
    println!("\n--- Statistics ---");

    let avg_day_length = results
        .iter()
        .filter_map(|r| {
            if let (Some(rise), Some(set)) = (r.sunrise, r.sunset) {
                Some(set.signed_duration_since(rise).num_minutes())
            } else {
                None
            }
        })
        .sum::<i64>() as f64
        / results.len() as f64;

    println!(
        "Average day length: {:.0}h {:02.0}m",
        (avg_day_length / 60.0).floor(),
        avg_day_length % 60.0
    );

    let avg_illumination = results
        .iter()
        .map(|r| r.moon_position.illumination * 100.0)
        .sum::<f64>()
        / results.len() as f64;

    println!("Average moon illumination: {:.1}%", avg_illumination);

    println!("\n✓ Batch processing complete!");
}
