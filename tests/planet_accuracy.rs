//! Regression tests pinning planet positions to JPL Horizons reference data.
//!
//! Reference values were obtained from the JPL Horizons API
//! (<https://ssd.jpl.nasa.gov/api/horizons.api>) for an observer at
//! Boston, USA (42.3601 N, 71.0589 W, 0 m), quantities 4 and 20
//! (airless apparent azimuth/elevation and observer range), at three
//! epochs spanning the validity range of the Keplerian mean elements.
//!
//! A live drift check against Horizons also runs on a schedule in CI
//! (`.github/workflows/planet-validation.yml`, via
//! `scripts/planet_drift_check.py`); these offline tests guard the
//! algorithm itself on every build without network access.

use chrono::TimeZone;
use chrono_tz::UTC;
use solunatus::astro::Location;
use solunatus::astro::planets::{Planet, planet_position};

/// Maximum allowed angular deviation from Horizons, in degrees.
///
/// Observed deviations across 1990-2049 are below 0.06 degrees; 0.1 leaves
/// margin without masking a real regression (0.1 degrees of sky motion is
/// roughly 24 seconds of rise/set time).
const MAX_ANGLE_DEG: f64 = 0.1;

/// Maximum allowed relative error in the observer-to-planet distance.
const MAX_DIST_FRAC: f64 = 0.005;

const BOSTON_LAT: f64 = 42.3601;
const BOSTON_LON: f64 = -71.0589;

/// (planet, azimuth deg, altitude deg, distance AU) from JPL Horizons.
type Reference = (Planet, f64, f64, f64);

fn angle_delta(a: f64, b: f64) -> f64 {
    let d = (a - b).abs() % 360.0;
    d.min(360.0 - d)
}

fn check_epoch(epoch_utc: (i32, u32, u32, u32, u32, u32), references: &[Reference]) {
    let (y, mo, d, h, mi, s) = epoch_utc;
    let location = Location::new(BOSTON_LAT, BOSTON_LON).unwrap();
    let datetime = UTC.with_ymd_and_hms(y, mo, d, h, mi, s).unwrap();

    for &(planet, ref_az, ref_alt, ref_dist) in references {
        let pos = planet_position(planet, &location, &datetime);
        let d_az = angle_delta(pos.azimuth, ref_az);
        let d_alt = (pos.altitude - ref_alt).abs();
        let d_dist = (pos.distance_au - ref_dist).abs() / ref_dist;

        assert!(
            d_az <= MAX_ANGLE_DEG,
            "{} azimuth off by {d_az:.4} deg at {datetime} (got {:.4}, Horizons {ref_az:.4})",
            planet.name(),
            pos.azimuth,
        );
        assert!(
            d_alt <= MAX_ANGLE_DEG,
            "{} altitude off by {d_alt:.4} deg at {datetime} (got {:.4}, Horizons {ref_alt:.4})",
            planet.name(),
            pos.altitude,
        );
        assert!(
            d_dist <= MAX_DIST_FRAC,
            "{} distance off by {:.3}% at {datetime} (got {:.6} AU, Horizons {ref_dist:.6} AU)",
            planet.name(),
            d_dist * 100.0,
            pos.distance_au,
        );
    }
}

#[test]
fn matches_horizons_1990() {
    check_epoch(
        (1990, 1, 15, 17, 0, 0),
        &[
            (Planet::Mercury, 197.8003, 26.0687, 0.711146),
            (Planet::Venus, 176.8579, 32.8724, 0.268181),
            (Planet::Mars, 217.3456, 14.8610, 2.223391),
            (Planet::Jupiter, 24.6306, -20.3256, 4.230633),
            (Planet::Saturn, 190.2963, 24.9273, 10.999333),
            (Planet::Uranus, 201.2080, 21.1390, 20.311998),
            (Planet::Neptune, 195.4946, 24.1108, 31.168747),
        ],
    );
}

#[test]
fn matches_horizons_2026() {
    check_epoch(
        (2026, 6, 15, 16, 0, 0),
        &[
            (Planet::Mercury, 109.3974, 53.1921, 0.824931),
            (Planet::Venus, 98.5039, 41.7830, 1.155799),
            (Planet::Mars, 227.6540, 57.3981, 2.148972),
            (Planet::Jupiter, 104.0132, 46.0910, 6.100534),
            (Planet::Saturn, 250.8763, 24.1420, 9.745155),
            (Planet::Uranus, 208.6130, 65.9907, 20.400931),
            (Planet::Neptune, 255.4212, 16.0938, 30.035179),
        ],
    );
}

#[test]
fn matches_horizons_2049() {
    check_epoch(
        (2049, 12, 1, 17, 0, 0),
        &[
            (Planet::Mercury, 166.4200, 20.5501, 1.172121),
            (Planet::Venus, 194.3629, 26.1557, 1.693783),
            (Planet::Mars, 232.0988, 21.5957, 2.249341),
            (Planet::Jupiter, 310.9630, -11.9117, 4.592380),
            (Planet::Saturn, 140.3109, 15.6994, 10.669645),
            (Planet::Uranus, 268.8551, 7.9026, 18.438850),
            (Planet::Neptune, 24.2892, -26.9989, 28.866532),
        ],
    );
}
