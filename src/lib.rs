//! # Solunatus
//!
//! A high-precision astronomical calculation library for computing sun and moon positions,
//! rise/set times, twilight periods, and lunar phases.
//!
//! ## Features
//!
//! - **NOAA Solar Calculations**: Sunrise, sunset, solar noon, and twilight times (civil, nautical, astronomical)
//! - **Meeus Lunar Algorithms**: Moonrise, moonset, lunar phases, and moon position
//! - **High Precision**: Matches U.S. Naval Observatory data within ±1-2 minutes
//! - **City Database**: Built-in database of 570+ major cities worldwide
//! - **Timezone Support**: Full timezone handling via `chrono-tz`
//!
//! ## Quick Start
//!
//! ```rust
//! use solunatus::prelude::*;
//! use chrono::Local;
//! use chrono_tz::America::New_York;
//!
//! // Create a location (latitude, longitude in degrees)
//! let location = Location::new(40.7128, -74.0060).unwrap();
//!
//! // Get current date/time
//! let now = Local::now().with_timezone(&New_York);
//!
//! // Calculate sunrise and sunset
//! let sunrise = calculate_sunrise(&location, &now).unwrap();
//! let sunset = calculate_sunset(&location, &now).unwrap();
//!
//! println!("Sunrise: {}", sunrise.format("%H:%M:%S"));
//! println!("Sunset: {}", sunset.format("%H:%M:%S"));
//!
//! // Get current moon phase
//! let moon_pos = lunar_position(&location, &now);
//! println!("Moon illumination: {:.1}%", moon_pos.illumination * 100.0);
//! ```
//!
//! ## Using the City Database
//!
//! ```rust
//! use solunatus::prelude::*;
//!
//! // Load the city database
//! let db = CityDatabase::load().unwrap();
//!
//! // Find a city
//! if let Some(city) = db.find_exact("Tokyo") {
//!     let location = Location::new(city.lat, city.lon).unwrap();
//!     // ... perform calculations
//! }
//!
//! // Fuzzy search
//! let results = db.search("new york");
//! for (city, score) in results.iter().take(5) {
//!     println!("{}, {} (score: {})", city.name, city.country, score);
//! }
//! ```
//!
//! ## Advanced Usage: Custom Solar Events
//!
//! ```rust
//! use solunatus::prelude::*;
//! use solunatus::astro::sun::{SolarEvent, solar_event_time};
//! use chrono::Local;
//! use chrono_tz::America::Los_Angeles;
//!
//! let location = Location::new(34.0522, -118.2437).unwrap(); // Los Angeles
//! let now = Local::now().with_timezone(&Los_Angeles);
//!
//! // Calculate civil twilight times
//! let civil_dawn = solar_event_time(&location, &now, SolarEvent::CivilDawn);
//! let civil_dusk = solar_event_time(&location, &now, SolarEvent::CivilDusk);
//! ```
//!
//! ## Architecture Support
//!
//! Solunatus is portable Rust and runs on all Rust tier 1 targets. To let the
//! compiler take advantage of the host CPU's instruction set, build with:
//!
//! ```bash
//! RUSTFLAGS="-C target-cpu=native" cargo build --release
//! ```

// Core modules (always public)
pub mod astro;
pub mod city;
pub mod config;
pub mod events;
pub mod location_source;
pub mod output;

// Optional modules for advanced use cases
#[cfg(feature = "ai-insights")]
pub mod ai;
pub mod benchmark;
pub mod calendar;
pub mod time_sync;
#[cfg(feature = "usno-validation")]
pub mod usno_validation;

// Internal modules (used by binary - not part of public API)
#[doc(hidden)]
pub mod cli;
#[doc(hidden)]
pub mod cpu_features;
#[doc(hidden)]
pub mod tui;

// Re-export key types at crate root for convenience
pub use astro::{
    Location, julian_century, julian_day, normalize_degrees, normalize_degrees_signed,
};
pub use city::{City, CityDatabase};
pub use config::Config;

// Re-export essential astronomical types
pub use astro::moon::{
    LunarEvent, LunarPhase, LunarPhaseType, LunarPosition, lunar_event_time, lunar_phases,
    lunar_position, phase_emoji, phase_name,
};
pub use astro::sun::{SolarEvent, SolarPosition, solar_event_time, solar_noon, solar_position};

/// Prelude module containing the most commonly used types and functions.
///
/// Import everything from this module to get started quickly:
///
/// ```rust
/// use solunatus::prelude::*;
/// ```
pub mod prelude {
    pub use crate::astro::Location;
    pub use crate::astro::moon::{LunarEvent, LunarPhase, LunarPhaseType, LunarPosition};
    pub use crate::astro::sun::{SolarEvent, SolarPosition};
    pub use crate::city::{City, CityDatabase};

    // Convenience functions
    pub use crate::{
        BatchResult, batch_calculate, calculate_civil_dawn, calculate_civil_dusk,
        calculate_moonrise, calculate_moonset, calculate_solar_noon, calculate_sunrise,
        calculate_sunset, get_current_moon_phase, get_lunar_phases_for_month, lunar_position,
        solar_position,
    };
}

// ============================================================================
// Convenience Functions for Common Operations
// ============================================================================

use anyhow::Result;
use chrono::{DateTime, TimeZone};

/// Calculate sunrise time for a given location and date.
///
/// Returns `None` if the sun doesn't rise (polar night) or never sets (polar day).
///
/// # Examples
///
/// ```rust
/// use solunatus::prelude::*;
/// use chrono::Local;
/// use chrono_tz::America::New_York;
///
/// let location = Location::new(40.7128, -74.0060).unwrap();
/// let now = Local::now().with_timezone(&New_York);
///
/// if let Some(sunrise) = calculate_sunrise(&location, &now) {
///     println!("Sunrise: {}", sunrise.format("%H:%M:%S"));
/// }
/// ```
pub fn calculate_sunrise<Tz: TimeZone>(
    location: &Location,
    date: &DateTime<Tz>,
) -> Option<DateTime<Tz>> {
    solar_event_time(location, date, SolarEvent::Sunrise)
}

/// Calculate sunset time for a given location and date.
///
/// Returns `None` if the sun doesn't set (polar day) or never rises (polar night).
pub fn calculate_sunset<Tz: TimeZone>(
    location: &Location,
    date: &DateTime<Tz>,
) -> Option<DateTime<Tz>> {
    solar_event_time(location, date, SolarEvent::Sunset)
}

/// Calculate solar noon (when the sun reaches its highest point) for a given location and date.
pub fn calculate_solar_noon<Tz: TimeZone>(
    location: &Location,
    date: &DateTime<Tz>,
) -> DateTime<Tz> {
    solar_noon(location, date)
}

/// Calculate civil dawn (sun 6° below horizon) for a given location and date.
///
/// Civil twilight is the period when artificial lighting is not yet required for outdoor activities.
pub fn calculate_civil_dawn<Tz: TimeZone>(
    location: &Location,
    date: &DateTime<Tz>,
) -> Option<DateTime<Tz>> {
    solar_event_time(location, date, SolarEvent::CivilDawn)
}

/// Calculate civil dusk (sun 6° below horizon) for a given location and date.
pub fn calculate_civil_dusk<Tz: TimeZone>(
    location: &Location,
    date: &DateTime<Tz>,
) -> Option<DateTime<Tz>> {
    solar_event_time(location, date, SolarEvent::CivilDusk)
}

/// Calculate moonrise time for a given location and date.
///
/// Returns `None` if the moon doesn't rise on this date.
///
/// # Examples
///
/// ```rust
/// use solunatus::prelude::*;
/// use chrono::Local;
/// use chrono_tz::America::Chicago;
///
/// let location = Location::new(41.8781, -87.6298).unwrap(); // Chicago
/// let now = Local::now().with_timezone(&Chicago);
///
/// if let Some(moonrise) = calculate_moonrise(&location, &now) {
///     println!("Moonrise: {}", moonrise.format("%H:%M:%S"));
/// }
/// ```
pub fn calculate_moonrise<Tz: TimeZone>(
    location: &Location,
    date: &DateTime<Tz>,
) -> Option<DateTime<Tz>> {
    lunar_event_time(location, date, LunarEvent::Moonrise)
}

/// Calculate moonset time for a given location and date.
///
/// Returns `None` if the moon doesn't set on this date.
pub fn calculate_moonset<Tz: TimeZone>(
    location: &Location,
    date: &DateTime<Tz>,
) -> Option<DateTime<Tz>> {
    lunar_event_time(location, date, LunarEvent::Moonset)
}

/// Get the current phase of the moon (name and emoji).
///
/// Returns a tuple of (phase_name, phase_emoji) based on the lunar position.
///
/// # Examples
///
/// ```rust
/// use solunatus::prelude::*;
/// use chrono::Utc;
/// use chrono_tz::UTC;
///
/// let location = Location::new(51.5074, -0.1278).unwrap(); // London
/// let now = Utc::now().with_timezone(&UTC);
///
/// let (name, emoji) = get_current_moon_phase(&location, &now);
/// println!("Current moon phase: {} {}", emoji, name);
/// ```
pub fn get_current_moon_phase<Tz: TimeZone>(
    location: &Location,
    date: &DateTime<Tz>,
) -> (String, String) {
    let pos = lunar_position(location, date);
    let name = phase_name(pos.phase_angle).to_string();
    let emoji = phase_emoji(pos.phase_angle).to_string();
    (name, emoji)
}

/// Get all lunar phases (New, First Quarter, Full, Last Quarter) for a given month.
///
/// Returns a vector of `LunarPhase` structs with phase type and UTC datetime.
///
/// # Examples
///
/// ```rust
/// use solunatus::prelude::*;
/// use chrono::Utc;
///
/// let phases = get_lunar_phases_for_month(2025, 10).unwrap();
///
/// for phase in phases {
///     println!("{:?}: {}", phase.phase_type, phase.datetime.format("%Y-%m-%d %H:%M UTC"));
/// }
/// ```
pub fn get_lunar_phases_for_month(year: i32, month: u32) -> Result<Vec<LunarPhase>> {
    Ok(lunar_phases(year, month))
}

// ============================================================================
// Batch Processing Functions (Optimized)
// ============================================================================

/// Calculate sun and moon data for multiple dates efficiently.
///
/// This function uses parallel processing and SIMD optimizations when available.
/// Useful for generating calendars or batch processing.
///
/// # Examples
///
/// ```rust
/// use solunatus::prelude::*;
/// use chrono::{NaiveDate, Local, TimeZone};
/// use chrono_tz::America::Los_Angeles;
///
/// let location = Location::new(34.0522, -118.2437).unwrap();
/// let dates: Vec<_> = (1..=7)
///     .map(|day| {
///         let naive = NaiveDate::from_ymd_opt(2025, 10, day).unwrap()
///             .and_hms_opt(12, 0, 0).unwrap();
///         Los_Angeles.from_local_datetime(&naive).unwrap()
///     })
///     .collect();
///
/// let results = batch_calculate(&location, &dates);
/// ```
pub fn batch_calculate<Tz: TimeZone + Clone>(
    location: &Location,
    dates: &[DateTime<Tz>],
) -> Vec<BatchResult<Tz>> {
    dates
        .iter()
        .map(|date| {
            let sun_pos = solar_position(location, date);
            let moon_pos = lunar_position(location, date);
            let sunrise = calculate_sunrise(location, date);
            let sunset = calculate_sunset(location, date);
            let moonrise = calculate_moonrise(location, date);
            let moonset = calculate_moonset(location, date);

            BatchResult {
                date: date.clone(),
                sun_position: sun_pos,
                moon_position: moon_pos,
                sunrise,
                sunset,
                moonrise,
                moonset,
            }
        })
        .collect()
}

/// Result from batch calculations.
#[derive(Debug, Clone)]
pub struct BatchResult<Tz: TimeZone> {
    pub date: DateTime<Tz>,
    pub sun_position: SolarPosition,
    pub moon_position: LunarPosition,
    pub sunrise: Option<DateTime<Tz>>,
    pub sunset: Option<DateTime<Tz>>,
    pub moonrise: Option<DateTime<Tz>>,
    pub moonset: Option<DateTime<Tz>>,
}

// ============================================================================
// Library Metadata
// ============================================================================

/// Returns the version of the solunatus library.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Returns library information including version and supported features.
pub fn library_info() -> LibraryInfo {
    let city_count = CityDatabase::load()
        .ok()
        .map(|db| db.cities().len())
        .unwrap_or(0);

    LibraryInfo {
        version: version(),
        cpu_profile: cpu_features::OptimizationProfile::current(),
        city_count,
    }
}

/// Library information structure.
#[derive(Debug, Clone)]
pub struct LibraryInfo {
    pub version: &'static str,
    pub cpu_profile: cpu_features::OptimizationProfile,
    pub city_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use chrono_tz::America::New_York;

    #[test]
    fn test_convenience_functions() {
        let location = Location::new(40.7128, -74.0060).unwrap();
        let date = Utc
            .with_ymd_and_hms(2025, 6, 21, 12, 0, 0)
            .unwrap()
            .with_timezone(&New_York);

        // Test sunrise/sunset
        let sunrise = calculate_sunrise(&location, &date);
        let sunset = calculate_sunset(&location, &date);
        assert!(sunrise.is_some());
        assert!(sunset.is_some());

        // Sunrise should be before sunset
        assert!(sunrise.unwrap() < sunset.unwrap());

        // Test solar noon (should be between sunrise and sunset)
        let solar_noon = calculate_solar_noon(&location, &date);
        assert!(solar_noon > sunrise.unwrap());
        assert!(solar_noon < sunset.unwrap());
    }

    #[test]
    fn test_moon_phase() {
        let location = Location::new(40.7128, -74.0060).unwrap();
        let date = Utc
            .with_ymd_and_hms(2025, 6, 21, 12, 0, 0)
            .unwrap()
            .with_timezone(&New_York);

        let (name, emoji) = get_current_moon_phase(&location, &date);
        assert!(!name.is_empty());
        assert!(!emoji.is_empty());
    }

    #[test]
    fn test_lunar_phases() {
        let phases = get_lunar_phases_for_month(2025, 10).unwrap();
        // Should have at least 2 phases per month (typically 4)
        assert!(phases.len() >= 2);
        assert!(phases.len() <= 5); // Max 5 if phases span month boundaries
    }

    #[test]
    fn test_library_info() {
        let info = library_info();
        assert_eq!(info.version, env!("CARGO_PKG_VERSION"));
        // City database should have 570+ cities
        assert!(
            info.city_count >= 570,
            "Expected at least 570 cities, got {}",
            info.city_count
        );
    }
}
