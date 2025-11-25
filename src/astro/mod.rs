//! Astronomical calculations module.
//!
//! This module implements high-precision astronomical algorithms for calculating
//! solar and lunar positions, rise/set times, and related phenomena.
//!
//! ## Algorithms
//!
//! - **Solar calculations**: Based on NOAA solar position algorithms
//! - **Lunar calculations**: Based on Jean Meeus "Astronomical Algorithms"
//!
//! ## Modules
//!
//! - [`sun`] - Solar position and event calculations
//! - [`moon`] - Lunar position, phases, and event calculations
//! - [`units`] - Type-safe angle and coordinate units
//! - [`coordinates`] - Coordinate system transformations
//! - [`time_utils`] - Time and Julian Day utilities
//! - [`simd_math`] - SIMD-optimized mathematical operations
//! - [`m1_optimizations`] - Apple Silicon specific optimizations
//! - [`moon_batch_optimized`] - Batch lunar calculations with parallelization

pub mod coordinates;
pub mod m1_optimizations;
pub mod moon;
pub mod moon_batch_optimized;
pub mod simd_math;
pub mod sun;
pub mod time_utils;
pub mod units;

use chrono::{DateTime, Datelike, TimeZone, Timelike};
use units::{Latitude, Longitude};

// Re-export commonly used types
pub use units::{Altitude, Azimuth, Degrees, Radians, DEG_TO_RAD, RAD_TO_DEG};

/// Location on Earth
/// All calculations assume sea level (0m elevation) per USNO celestial navigation convention
#[derive(Debug, Clone, Copy)]
pub struct Location {
    pub latitude: Latitude,  // positive North
    pub longitude: Longitude, // positive East
}

impl Location {
    /// Create a new location with validation
    pub fn new(lat: f64, lon: f64) -> Result<Self, String> {
        Ok(Self {
            latitude: Latitude::new(lat)?,
            longitude: Longitude::new(lon)?,
        })
    }

    /// Create a new location without validation (use only when values are known to be valid)
    pub fn new_unchecked(lat: f64, lon: f64) -> Self {
        Self {
            latitude: Latitude::new_unchecked(lat),
            longitude: Longitude::new_unchecked(lon),
        }
    }

    /// Get latitude in degrees
    pub fn lat_degrees(&self) -> f64 {
        self.latitude.value()
    }

    /// Get longitude in degrees
    pub fn lon_degrees(&self) -> f64 {
        self.longitude.value()
    }
}

/// Calculate Julian Day from a given date and time.
///
/// Julian Day is a continuous count of days since the beginning of the Julian Period.
/// This function converts any timezone-aware DateTime to UTC before calculating the Julian Day.
///
/// # Arguments
///
/// * `dt` - A timezone-aware DateTime
///
/// # Returns
///
/// The Julian Day number as a floating-point value.
///
/// # Notes
///
/// - Julian Day is always defined in UTC
/// - The algorithms are formulated for Universal Time (UT), not Terrestrial Time (TT)
/// - The epoch (JD 0) corresponds to January 1, 4713 BC at noon
///
/// # Examples
///
/// ```
/// use solunatus::astro::julian_day;
/// use chrono::{TimeZone, Utc};
///
/// let dt = Utc.with_ymd_and_hms(2000, 1, 1, 12, 0, 0).unwrap();
/// let jd = julian_day(&dt);
/// assert!((jd - 2451545.0).abs() < 0.001); // J2000.0 epoch
/// ```
pub fn julian_day<T: TimeZone>(dt: &DateTime<T>) -> f64 {
    // Convert to UTC for Julian Day calculation
    let utc_dt = dt.with_timezone(&chrono::Utc);

    let year = utc_dt.year() as f64;
    let month = utc_dt.month() as f64;
    let day = utc_dt.day() as f64
        + utc_dt.hour() as f64 / 24.0
        + utc_dt.minute() as f64 / 1440.0
        + utc_dt.second() as f64 / 86400.0;

    let mut y = year;
    let mut m = month;

    if month <= 2.0 {
        y -= 1.0;
        m += 12.0;
    }

    let a = (y / 100.0).floor();
    let b = 2.0 - a + (a / 4.0).floor();

    (365.25 * (y + 4716.0)).floor() + (30.6001 * (m + 1.0)).floor() + day + b - 1524.5
}

/// Calculate Julian Century from a Julian Day number.
///
/// Julian Century is the number of centuries since the J2000.0 epoch (JD 2451545.0),
/// which corresponds to January 1, 2000, 12:00 TT.
///
/// # Arguments
///
/// * `jd` - Julian Day number
///
/// # Returns
///
/// The Julian Century value (number of centuries since J2000.0).
///
/// # Examples
///
/// ```
/// use solunatus::astro::julian_century;
///
/// // J2000.0 epoch
/// let t = julian_century(2451545.0);
/// assert_eq!(t, 0.0);
/// ```
pub fn julian_century(jd: f64) -> f64 {
    (jd - 2451545.0) / 36525.0
}

/// Normalize an angle to the range [0, 360) degrees.
///
/// # Arguments
///
/// * `angle` - Angle in degrees (can be any value)
///
/// # Returns
///
/// The normalized angle in the range [0, 360).
///
/// # Examples
///
/// ```
/// use solunatus::astro::normalize_degrees;
///
/// assert_eq!(normalize_degrees(370.0), 10.0);
/// assert_eq!(normalize_degrees(-10.0), 350.0);
/// assert_eq!(normalize_degrees(0.0), 0.0);
/// ```
pub fn normalize_degrees(angle: f64) -> f64 {
    Degrees::new(angle).normalized().value()
}

/// Normalize an angle to the range [-180, 180) degrees.
///
/// This is useful for representing angles relative to a reference direction,
/// where negative values indicate one direction and positive values indicate the other.
///
/// # Arguments
///
/// * `angle` - Angle in degrees (can be any value)
///
/// # Returns
///
/// The normalized angle in the range [-180, 180).
///
/// # Examples
///
/// ```
/// use solunatus::astro::normalize_degrees_signed;
///
/// assert_eq!(normalize_degrees_signed(190.0), -170.0);
/// assert_eq!(normalize_degrees_signed(-190.0), 170.0);
/// ```
pub fn normalize_degrees_signed(angle: f64) -> f64 {
    Degrees::new(angle).normalized_signed().value()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_julian_day() {
        // Test known Julian Day values
        // January 1, 2000, 12:00:00 UTC = JD 2451545.0
        let dt = Utc.with_ymd_and_hms(2000, 1, 1, 12, 0, 0).unwrap();
        let jd = julian_day(&dt);
        assert!((jd - 2451545.0).abs() < 0.001);
    }

    #[test]
    fn test_normalize_degrees() {
        assert_eq!(normalize_degrees(370.0), 10.0);
        assert_eq!(normalize_degrees(-10.0), 350.0);
        assert_eq!(normalize_degrees(0.0), 0.0);
    }
}
