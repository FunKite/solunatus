//! NOAA Solar Calculations.
//!
//! This module implements solar position algorithms based on NOAA's
//! solar position calculator.
//!
//! # References
//!
//! Based on NOAA's solar position algorithms:
//! <https://gml.noaa.gov/grad/solcalc/calcdetails.html>
//!
//! # Accuracy
//!
//! These algorithms provide high precision results that match U.S. Naval Observatory
//! data within ±1-2 minutes for sunrise/sunset times.

use super::*;
use chrono::{DateTime, Duration, TimeZone};

/// Types of solar events that can be calculated.
///
/// Each event corresponds to a specific solar altitude angle:
/// - Sunrise/Sunset: -0.833° (accounting for refraction and solar disk)
/// - Civil twilight: -6°
/// - Nautical twilight: -12°
/// - Astronomical twilight: -18°
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SolarEvent {
    /// Sunrise (top edge of sun appears on horizon)
    Sunrise,
    /// Sunset (top edge of sun disappears below horizon)
    Sunset,
    /// Solar noon (sun reaches highest point in sky)
    SolarNoon,
    /// Civil dawn (sun 6° below horizon, morning)
    CivilDawn,
    /// Civil dusk (sun 6° below horizon, evening)
    CivilDusk,
    /// Nautical dawn (sun 12° below horizon, morning)
    NauticalDawn,
    /// Nautical dusk (sun 12° below horizon, evening)
    NauticalDusk,
    /// Astronomical dawn (sun 18° below horizon, morning)
    AstronomicalDawn,
    /// Astronomical dusk (sun 18° below horizon, evening)
    AstronomicalDusk,
}

impl SolarEvent {
    /// Get the solar altitude angle for this event in degrees.
    ///
    /// Returns the angle of the sun below (negative) or above (positive) the horizon.
    pub fn altitude(&self) -> f64 {
        match self {
            SolarEvent::Sunrise | SolarEvent::Sunset => -0.833, // Standard refraction + solar radius
            SolarEvent::CivilDawn | SolarEvent::CivilDusk => -6.0,
            SolarEvent::NauticalDawn | SolarEvent::NauticalDusk => -12.0,
            SolarEvent::AstronomicalDawn | SolarEvent::AstronomicalDusk => -18.0,
            SolarEvent::SolarNoon => 90.0, // Not used for altitude calculation
        }
    }
}

/// Solar position in the sky (altitude and azimuth).
///
/// This represents where the sun appears in the sky at a given time and location.
#[derive(Debug, Clone, Copy)]
pub struct SolarPosition {
    /// Altitude in degrees above the horizon (negative if below horizon)
    pub altitude: f64,
    /// Azimuth in degrees from North (0=N, 90=E, 180=S, 270=W)
    pub azimuth: f64,
}

/// Calculate geometric mean longitude of the Sun (degrees)
fn sun_geom_mean_long(t: f64) -> f64 {
    let l0 = 280.46646 + t * (36000.76983 + t * 0.0003032);
    normalize_degrees(l0)
}

/// Calculate geometric mean anomaly of the Sun (degrees)
fn sun_geom_mean_anom(t: f64) -> f64 {
    357.52911 + t * (35999.05029 - 0.0001537 * t)
}

/// Calculate eccentricity of Earth's orbit
fn earth_orbit_eccentricity(t: f64) -> f64 {
    0.016708634 - t * (0.000042037 + 0.0000001267 * t)
}

/// Calculate the equation of center for the Sun (degrees)
fn sun_eq_of_center(t: f64) -> f64 {
    let m = sun_geom_mean_anom(t) * DEG_TO_RAD;
    (1.914602 - t * (0.004817 + 0.000014 * t)) * m.sin()
        + (0.019993 - 0.000101 * t) * (2.0 * m).sin()
        + 0.000289 * (3.0 * m).sin()
}

/// Calculate true longitude of the Sun (degrees)
fn sun_true_long(t: f64) -> f64 {
    sun_geom_mean_long(t) + sun_eq_of_center(t)
}

/// Calculate apparent longitude of the Sun (degrees)
fn sun_apparent_long(t: f64) -> f64 {
    let o = sun_true_long(t);
    let omega = 125.04 - 1934.136 * t;
    o - 0.00569 - 0.00478 * (omega * DEG_TO_RAD).sin()
}

/// Calculate mean obliquity of the ecliptic (degrees)
fn mean_obliquity_of_ecliptic(t: f64) -> f64 {
    let seconds = 21.448 - t * (46.8150 + t * (0.00059 - t * 0.001813));
    23.0 + (26.0 + (seconds / 60.0)) / 60.0
}

/// Calculate corrected obliquity of the ecliptic (degrees)
fn obliquity_correction(t: f64) -> f64 {
    let e0 = mean_obliquity_of_ecliptic(t);
    let omega = 125.04 - 1934.136 * t;
    e0 + 0.00256 * (omega * DEG_TO_RAD).cos()
}

/// Calculate the Sun's declination (degrees)
fn sun_declination(t: f64) -> f64 {
    let e = obliquity_correction(t);
    let lambda = sun_apparent_long(t);
    let sint = (e * DEG_TO_RAD).sin() * (lambda * DEG_TO_RAD).sin();
    sint.asin() * RAD_TO_DEG
}

/// Calculate the equation of time in minutes.
///
/// The equation of time represents the difference between apparent solar time
/// (sundial time) and mean solar time (clock time). This varies throughout the
/// year due to Earth's elliptical orbit and axial tilt.
///
/// # Arguments
///
/// * `t` - Julian Century from J2000.0
///
/// # Returns
///
/// The equation of time in minutes. Positive values mean the sundial is ahead
/// of clock time, negative values mean it's behind.
#[must_use]
pub fn equation_of_time(t: f64) -> f64 {
    let epsilon = obliquity_correction(t);
    let l0 = sun_geom_mean_long(t);
    let e = earth_orbit_eccentricity(t);
    let m = sun_geom_mean_anom(t);

    let y = (epsilon * DEG_TO_RAD / 2.0).tan().powi(2);

    let sin2l0 = (2.0 * l0 * DEG_TO_RAD).sin();
    let sinm = (m * DEG_TO_RAD).sin();
    let cos2l0 = (2.0 * l0 * DEG_TO_RAD).cos();
    let sin4l0 = (4.0 * l0 * DEG_TO_RAD).sin();
    let sin2m = (2.0 * m * DEG_TO_RAD).sin();

    let etime = y * sin2l0 - 2.0 * e * sinm + 4.0 * e * y * sinm * cos2l0
        - 0.5 * y * y * sin4l0
        - 1.25 * e * e * sin2m;

    4.0 * etime * RAD_TO_DEG // in minutes of time
}

/// Calculate hour angle for a given solar altitude (degrees)
fn hour_angle_for_altitude(lat: f64, dec: f64, altitude: f64) -> Option<f64> {
    let lat_rad = lat * DEG_TO_RAD;
    let dec_rad = dec * DEG_TO_RAD;
    let alt_rad = altitude * DEG_TO_RAD;

    let cos_ha = (alt_rad.sin() - lat_rad.sin() * dec_rad.sin()) / (lat_rad.cos() * dec_rad.cos());

    if !(-1.0..=1.0).contains(&cos_ha) {
        None // Event doesn't occur (polar day/night)
    } else {
        Some(cos_ha.acos() * RAD_TO_DEG)
    }
}

/// Calculate solar noon time for a given location and date.
///
/// Solar noon is when the sun reaches its highest point in the sky for the day.
/// This is not necessarily 12:00 PM clock time due to:
/// - Longitude within the timezone
/// - Equation of time variations
///
/// # Arguments
///
/// * `location` - Geographic location
/// * `date` - Date for calculation (time component is ignored)
///
/// # Returns
///
/// DateTime of solar noon in the input timezone.
///
/// # Examples
///
/// ```
/// use solunatus::prelude::*;
/// use chrono::Local;
/// use chrono_tz::America::New_York;
///
/// let location = Location::new(40.7128, -74.0060).unwrap();
/// let now = Local::now().with_timezone(&New_York);
/// let noon = solunatus::astro::sun::solar_noon(&location, &now);
/// println!("Solar noon: {}", noon.format("%H:%M:%S"));
/// ```
#[must_use]
pub fn solar_noon<T: TimeZone>(location: &Location, date: &DateTime<T>) -> DateTime<T> {
    // Use noon UTC as reference for calculations
    let base_date = date.date_naive().and_hms_opt(12, 0, 0).unwrap();
    let utc_noon = chrono::Utc.from_local_datetime(&base_date).unwrap();

    let jd = julian_day(&utc_noon);
    let t = julian_century(jd);
    let eqtime = equation_of_time(t);

    // Solar noon in minutes from midnight UTC
    let solar_noon_offset = 720.0 - 4.0 * location.longitude.value() - eqtime;

    let utc_midnight = date.date_naive().and_hms_opt(0, 0, 0).unwrap();
    let solar_noon_utc = chrono::Utc.from_local_datetime(&utc_midnight).unwrap()
        + Duration::seconds((solar_noon_offset * 60.0) as i64);

    solar_noon_utc.with_timezone(&date.timezone())
}

/// Calculate the time of a solar event for a given location and date.
///
/// Calculates when specific solar events occur (sunrise, sunset, twilight times, etc.).
/// Returns `None` if the event doesn't occur on this date (e.g., polar day/night).
///
/// # Arguments
///
/// * `location` - Geographic location
/// * `date` - Date for calculation (time component is ignored)
/// * `event` - Type of solar event to calculate
///
/// # Returns
///
/// - `Some(DateTime)` - The time when the event occurs in the input timezone
/// - `None` - The event doesn't occur on this date (polar conditions)
///
/// # Examples
///
/// ```
/// use solunatus::prelude::*;
/// use solunatus::astro::sun::{solar_event_time, SolarEvent};
/// use chrono::Local;
/// use chrono_tz::America::New_York;
///
/// let location = Location::new(40.7128, -74.0060).unwrap();
/// let now = Local::now().with_timezone(&New_York);
///
/// if let Some(sunrise) = solar_event_time(&location, &now, SolarEvent::Sunrise) {
///     println!("Sunrise: {}", sunrise.format("%H:%M:%S"));
/// }
/// ```
pub fn solar_event_time<T: TimeZone>(
    location: &Location,
    date: &DateTime<T>,
    event: SolarEvent,
) -> Option<DateTime<T>> {
    if event == SolarEvent::SolarNoon {
        return Some(solar_noon(location, date));
    }

    // Use noon UTC as reference for calculations
    let base_date = date.date_naive().and_hms_opt(12, 0, 0).unwrap();
    let utc_noon = chrono::Utc.from_local_datetime(&base_date).unwrap();

    let jd = julian_day(&utc_noon);
    let t = julian_century(jd);
    let dec = sun_declination(t);
    let eqtime = equation_of_time(t);

    let ha = hour_angle_for_altitude(location.latitude.value(), dec, event.altitude())?;

    let is_rising = matches!(
        event,
        SolarEvent::Sunrise
            | SolarEvent::CivilDawn
            | SolarEvent::NauticalDawn
            | SolarEvent::AstronomicalDawn
    );

    let offset = if is_rising {
        720.0 - 4.0 * (location.longitude.value() + ha) - eqtime
    } else {
        720.0 - 4.0 * (location.longitude.value() - ha) - eqtime
    };

    let utc_midnight = date.date_naive().and_hms_opt(0, 0, 0).unwrap();
    let event_utc = chrono::Utc.from_local_datetime(&utc_midnight).unwrap()
        + Duration::seconds((offset * 60.0) as i64);

    Some(event_utc.with_timezone(&date.timezone()))
}

/// Calculate the solar position (altitude and azimuth) at a specific time.
///
/// Computes where the sun appears in the sky at a given moment.
///
/// # Arguments
///
/// * `location` - Geographic location
/// * `dt` - Date and time for calculation
///
/// # Returns
///
/// A `SolarPosition` containing:
/// - `altitude`: Degrees above horizon (negative if below horizon)
/// - `azimuth`: Degrees from North (0=N, 90=E, 180=S, 270=W)
///
/// # Examples
///
/// ```
/// use solunatus::prelude::*;
/// use chrono::Local;
/// use chrono_tz::America::Los_Angeles;
///
/// let location = Location::new(34.0522, -118.2437).unwrap();
/// let now = Local::now().with_timezone(&Los_Angeles);
/// let pos = solar_position(&location, &now);
///
/// println!("Sun altitude: {:.2}°", pos.altitude);
/// println!("Sun azimuth: {:.2}°", pos.azimuth);
/// ```
#[must_use]
pub fn solar_position<T: TimeZone>(location: &Location, dt: &DateTime<T>) -> SolarPosition {
    let jd = julian_day(dt);
    let t = julian_century(jd);

    let dec = sun_declination(t);
    let eqtime = equation_of_time(t);

    // Convert to UTC for calculation (CRITICAL: must use UTC, not local time)
    let utc_dt = dt.with_timezone(&chrono::Utc);

    // Calculate true solar time (using UTC hours)
    let time_offset = eqtime + 4.0 * location.longitude.value();
    let true_solar_time = utc_dt.hour() as f64 * 60.0
        + utc_dt.minute() as f64
        + utc_dt.second() as f64 / 60.0
        + time_offset;

    // Hour angle in degrees
    let ha = (true_solar_time / 4.0) - 180.0;

    let lat_rad = location.latitude.value() * DEG_TO_RAD;
    let dec_rad = dec * DEG_TO_RAD;
    let ha_rad = ha * DEG_TO_RAD;

    // Calculate altitude
    let sin_alt = lat_rad.sin() * dec_rad.sin() + lat_rad.cos() * dec_rad.cos() * ha_rad.cos();
    let altitude = sin_alt.asin() * RAD_TO_DEG;

    // Calculate azimuth using atan2 for numerical stability
    let altitude_rad = altitude * DEG_TO_RAD;
    let cos_az =
        (dec_rad.sin() - lat_rad.sin() * altitude_rad.sin()) / (lat_rad.cos() * altitude_rad.cos());
    let sin_az = -ha_rad.sin() * dec_rad.cos() / altitude_rad.cos();

    let mut azimuth = sin_az.atan2(cos_az) * RAD_TO_DEG;
    if azimuth < 0.0 {
        azimuth += 360.0;
    }

    SolarPosition { altitude, azimuth }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_equation_of_time() {
        // Test equation of time calculation
        let dt = Utc.with_ymd_and_hms(2000, 1, 1, 12, 0, 0).unwrap();
        let jd = julian_day(&dt);
        let t = julian_century(jd);
        let eqtime = equation_of_time(t);

        // Should be approximately -3 minutes on Jan 1, 2000
        assert!((eqtime - (-3.0)).abs() < 1.0);
    }
}
