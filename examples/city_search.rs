//! City search example: Find cities and calculate astronomical data
//!
//! Run with:
//! ```bash
//! cargo run --example city_search
//! ```

use chrono::Local;
use chrono_tz::Tz;
use solunatus::prelude::*;

fn main() {
    println!("=== Solunatus Library - City Search ===\n");

    // Load the city database
    let db = CityDatabase::load().expect("Failed to load city database");

    // Example 1: Exact city lookup
    println!("--- Exact City Lookup ---");
    if let Some(city) = db.find_exact("Tokyo") {
        print_city_info(city);
    } else {
        println!("City not found");
    }

    // Example 2: Fuzzy search
    println!("\n--- Fuzzy Search: 'paris' ---");
    let results = db.search("paris");

    println!("Found {} matching cities:\n", results.len());
    for (city, score) in results.iter().take(5) {
        println!(
            "  {} - {}, {} (score: {})",
            city.name,
            city.state.as_deref().unwrap_or("N/A"),
            city.country,
            score
        );
    }

    // Example 3: Calculate sunrise/sunset for multiple cities
    println!("\n--- Sunrise/Sunset Times for Major Cities ---\n");
    let major_cities = ["New York", "London", "Tokyo", "Sydney", "Mumbai"];

    for city_name in &major_cities {
        if let Some(city) = db.find_exact(city_name) {
            let location = Location::new(city.lat, city.lon).expect("Invalid coordinates");
            let tz: Tz = city.tz.parse().expect("Invalid timezone");
            let now = Local::now().with_timezone(&tz);

            print!("{:15} ", city.name);

            if let Some(sunrise) = calculate_sunrise(&location, &now) {
                print!("☀️ ↑ {} ", sunrise.format("%H:%M"));
            } else {
                print!("☀️ ↑ --:-- ");
            }

            if let Some(sunset) = calculate_sunset(&location, &now) {
                print!("☀️ ↓ {} ", sunset.format("%H:%M"));
            } else {
                print!("☀️ ↓ --:-- ");
            }

            let (_, emoji) = get_current_moon_phase(&location, &now);
            print!("{}", emoji);

            println!();
        }
    }

    println!("\n✓ City search complete!");
}

fn print_city_info(city: &City) {
    println!("City:      {}", city.name);
    if let Some(ref state) = city.state {
        println!("State:     {}", state);
    }
    println!("Country:   {}", city.country);
    println!("Latitude:  {:.4}°", city.lat);
    println!("Longitude: {:.4}°", city.lon);
    println!("Timezone:  {}", city.tz);
}
