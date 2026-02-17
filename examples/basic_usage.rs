//! Basic usage example: Calculate sunrise and sunset for a location
//!
//! Run with:
//! ```bash
//! cargo run --example basic_usage
//! ```

use chrono::Local;
use chrono_tz::America::New_York;
use solunatus::prelude::*;

fn main() {
    println!("=== Solunatus Library - Basic Usage ===\n");

    // Create a location (New York City)
    let location = Location::new(40.7128, -74.0060).expect("Invalid coordinates");

    // Get current date/time in New York timezone
    let now = Local::now().with_timezone(&New_York);

    println!("Location: New York City");
    println!("Latitude: {:.4}°N", location.lat_degrees());
    println!("Longitude: {:.4}°W", location.lon_degrees().abs());
    println!("Date: {}\n", now.format("%Y-%m-%d"));

    // Calculate sunrise and sunset
    println!("--- Solar Events ---");
    if let Some(sunrise) = calculate_sunrise(&location, &now) {
        println!("Sunrise:     {}", sunrise.format("%H:%M:%S %Z"));
    } else {
        println!("Sunrise:     No sunrise today (polar day/night)");
    }

    if let Some(sunset) = calculate_sunset(&location, &now) {
        println!("Sunset:      {}", sunset.format("%H:%M:%S %Z"));
    } else {
        println!("Sunset:      No sunset today (polar day/night)");
    }

    let solar_noon = calculate_solar_noon(&location, &now);
    println!("Solar Noon:  {}", solar_noon.format("%H:%M:%S %Z"));

    // Calculate twilight times
    if let Some(civil_dawn) = calculate_civil_dawn(&location, &now) {
        println!("Civil Dawn:  {}", civil_dawn.format("%H:%M:%S %Z"));
    }

    if let Some(civil_dusk) = calculate_civil_dusk(&location, &now) {
        println!("Civil Dusk:  {}", civil_dusk.format("%H:%M:%S %Z"));
    }

    // Calculate current sun position
    let sun_pos = solar_position(&location, &now);
    println!("\n--- Current Sun Position ---");
    println!("Altitude:    {:.2}°", sun_pos.altitude);
    println!(
        "Azimuth:     {:.2}° ({})",
        sun_pos.azimuth,
        solunatus::astro::coordinates::azimuth_to_compass(sun_pos.azimuth)
    );

    // Calculate moon information
    println!("\n--- Lunar Events ---");
    if let Some(moonrise) = calculate_moonrise(&location, &now) {
        println!("Moonrise:    {}", moonrise.format("%H:%M:%S %Z"));
    } else {
        println!("Moonrise:    No moonrise today");
    }

    if let Some(moonset) = calculate_moonset(&location, &now) {
        println!("Moonset:     {}", moonset.format("%H:%M:%S %Z"));
    } else {
        println!("Moonset:     No moonset today");
    }

    // Current moon phase
    let (phase_name, phase_emoji) = get_current_moon_phase(&location, &now);
    println!("\n--- Current Moon Phase ---");
    println!("{} {}", phase_emoji, phase_name);

    let moon_pos = lunar_position(&location, &now);
    println!("Illumination: {:.1}%", moon_pos.illumination * 100.0);
    println!("Distance:     {:.0} km", moon_pos.distance);
    println!("Altitude:     {:.2}°", moon_pos.altitude);
    println!(
        "Azimuth:      {:.2}° ({})",
        moon_pos.azimuth,
        solunatus::astro::coordinates::azimuth_to_compass(moon_pos.azimuth)
    );

    println!("\n✓ Calculations complete!");
}
