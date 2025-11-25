//! Advanced example: Custom solar events and detailed position calculations
//!
//! Run with:
//! ```bash
//! cargo run --example custom_events
//! ```

use solunatus::astro::sun::{SolarEvent, solar_event_time};
use solunatus::astro::coordinates::azimuth_to_compass;
use solunatus::prelude::*;
use chrono::Local;
use chrono_tz::America::Denver;

fn main() {
    println!("=== Solunatus Library - Custom Solar Events ===\n");

    // Location: Denver, Colorado (Mile High City)
    let location = Location::new(39.7392, -104.9903).expect("Invalid coordinates");
    let now = Local::now().with_timezone(&Denver);

    println!("Location: Denver, Colorado");
    println!("Date: {}\n", now.format("%Y-%m-%d"));

    // Calculate all twilight periods
    println!("--- Twilight Periods ---\n");

    let events = [
        (SolarEvent::AstronomicalDawn, "Astronomical Dawn", "🔭"),
        (SolarEvent::NauticalDawn, "Nautical Dawn", "⚓"),
        (SolarEvent::CivilDawn, "Civil Dawn", "🏙️"),
        (SolarEvent::Sunrise, "Sunrise", "🌅"),
        (SolarEvent::SolarNoon, "Solar Noon", "☀️"),
        (SolarEvent::Sunset, "Sunset", "🌇"),
        (SolarEvent::CivilDusk, "Civil Dusk", "🏙️"),
        (SolarEvent::NauticalDusk, "Nautical Dusk", "⚓"),
        (SolarEvent::AstronomicalDusk, "Astronomical Dusk", "🔭"),
    ];

    for (event, name, emoji) in &events {
        if *event == SolarEvent::SolarNoon {
            let time = calculate_solar_noon(&location, &now);
            let pos = solar_position(&location, &time);
            println!("{} {:20} {} (altitude: {:.1}°)",
                emoji, name, time.format("%H:%M:%S"), pos.altitude);
        } else if let Some(time) = solar_event_time(&location, &now, *event) {
            let pos = solar_position(&location, &time);
            println!("{} {:20} {} (azimuth: {:.0}° {})",
                emoji, name, time.format("%H:%M:%S"),
                pos.azimuth, azimuth_to_compass(pos.azimuth));
        } else {
            println!("{} {:20} Not occurring today", emoji, name);
        }
    }

    // Calculate twilight durations
    println!("\n--- Twilight Durations ---\n");

    if let (Some(civil_dawn), Some(sunrise)) = (
        solar_event_time(&location, &now, SolarEvent::CivilDawn),
        solar_event_time(&location, &now, SolarEvent::Sunrise)
    ) {
        let duration = sunrise.signed_duration_since(civil_dawn);
        println!("Civil twilight (morning):     {:2} min", duration.num_minutes());
    }

    if let (Some(nautical_dawn), Some(civil_dawn)) = (
        solar_event_time(&location, &now, SolarEvent::NauticalDawn),
        solar_event_time(&location, &now, SolarEvent::CivilDawn)
    ) {
        let duration = civil_dawn.signed_duration_since(nautical_dawn);
        println!("Nautical twilight (morning):  {:2} min", duration.num_minutes());
    }

    if let (Some(astro_dawn), Some(nautical_dawn)) = (
        solar_event_time(&location, &now, SolarEvent::AstronomicalDawn),
        solar_event_time(&location, &now, SolarEvent::NauticalDawn)
    ) {
        let duration = nautical_dawn.signed_duration_since(astro_dawn);
        println!("Astronomical twilight (AM):   {:2} min", duration.num_minutes());
    }

    // Sun tracking throughout the day
    println!("\n--- Sun Position Throughout Day ---\n");
    println!("Time     Altitude  Azimuth  Direction");
    println!("─────────────────────────────────────");

    for hour in [6, 9, 12, 15, 18] {
        if let Some(time) = now.date_naive().and_hms_opt(hour, 0, 0) {
            if let Some(datetime) = time.and_local_timezone(Denver).single() {
                let pos = solar_position(&location, &datetime);
                if pos.altitude > -18.0 {  // Only show if sun is above astronomical twilight
                    println!("{:02}:00    {:>6.1}°  {:>6.0}°  {}",
                        hour,
                        pos.altitude,
                        pos.azimuth,
                        azimuth_to_compass(pos.azimuth)
                    );
                }
            }
        }
    }

    // Day/night information
    println!("\n--- Day/Night Information ---\n");

    if let (Some(sunrise), Some(sunset)) = (
        solar_event_time(&location, &now, SolarEvent::Sunrise),
        solar_event_time(&location, &now, SolarEvent::Sunset)
    ) {
        let daylight = sunset.signed_duration_since(sunrise);
        println!("Daylight duration:   {:2}h {:02}min",
            daylight.num_hours(), daylight.num_minutes() % 60);

        let darkness = chrono::Duration::hours(24) - daylight;
        println!("Darkness duration:   {:2}h {:02}min",
            darkness.num_hours(), darkness.num_minutes() % 60);
    }

    println!("\n✓ Custom event calculations complete!");
}
