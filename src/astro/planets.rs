//! Planetary position calculations for the major planets
//! (Mercury through Neptune).
//!
//! Positions are computed from Keplerian mean orbital elements with the major
//! Jupiter–Saturn and Uranus perturbation terms applied, following the method
//! described by Paul Schlyter ("How to compute planetary positions") with mean
//! elements consistent with Jean Meeus, "Astronomical Algorithms", Chapter 31.
//!
//! # Accuracy
//!
//! Apparent positions are accurate to roughly 1–2 arcminutes for the inner
//! planets and a few arcminutes for the outer planets, which keeps rise/set
//! times within about ±1–2 minutes of authoritative ephemerides.

use super::*;
use chrono::{DateTime, Duration, TimeZone};

/// The major planets supported by this module.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Planet {
    /// Mercury (innermost planet, twilight object near the sun)
    Mercury,
    /// Venus (brightest planet, morning/evening star)
    Venus,
    /// Mars (the red planet)
    Mars,
    /// Jupiter (largest planet)
    Jupiter,
    /// Saturn (ringed planet)
    Saturn,
    /// Uranus (borderline naked-eye under dark skies)
    Uranus,
    /// Neptune (binocular/telescope object)
    Neptune,
}

impl Planet {
    /// All supported planets in distance order from the sun.
    pub const ALL: [Planet; 7] = [
        Planet::Mercury,
        Planet::Venus,
        Planet::Mars,
        Planet::Jupiter,
        Planet::Saturn,
        Planet::Uranus,
        Planet::Neptune,
    ];

    /// Planet name (e.g. "Venus").
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Planet::Mercury => "Mercury",
            Planet::Venus => "Venus",
            Planet::Mars => "Mars",
            Planet::Jupiter => "Jupiter",
            Planet::Saturn => "Saturn",
            Planet::Uranus => "Uranus",
            Planet::Neptune => "Neptune",
        }
    }

    /// Astronomical symbol for the planet.
    #[must_use]
    pub fn symbol(&self) -> &'static str {
        match self {
            Planet::Mercury => "☿",
            Planet::Venus => "♀",
            Planet::Mars => "♂",
            Planet::Jupiter => "♃",
            Planet::Saturn => "♄",
            Planet::Uranus => "♅",
            Planet::Neptune => "♆",
        }
    }
}

/// Apparent position and brightness of a planet as seen from a location.
#[derive(Debug, Clone, Copy)]
pub struct PlanetPosition {
    /// Altitude in degrees above the horizon (negative if below horizon)
    pub altitude: f64,
    /// Azimuth in degrees from North (0=N, 90=E, 180=S, 270=W)
    pub azimuth: f64,
    /// Geocentric distance in astronomical units
    pub distance_au: f64,
    /// Approximate visual magnitude (lower is brighter)
    pub magnitude: f64,
    /// Angular distance from the sun in degrees (solar elongation)
    pub elongation: f64,
}

/// Types of planetary events that can be calculated.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlanetEvent {
    /// Planet rises above the horizon
    Rise,
    /// Planet sets below the horizon
    Set,
}

/// Rise/set altitude threshold: standard 34' atmospheric refraction at the
/// horizon (planetary disks are small enough to ignore).
const RISE_SET_ALTITUDE: f64 = -0.5667;

/// Keplerian orbital elements at a given epoch (angles in degrees).
struct Elements {
    /// Longitude of the ascending node
    n: f64,
    /// Inclination to the ecliptic
    i: f64,
    /// Argument of perihelion
    w: f64,
    /// Semi-major axis in AU
    a: f64,
    /// Eccentricity
    e: f64,
    /// Mean anomaly
    m: f64,
}

/// Day number used by the element polynomials: days from 2000 Jan 0.0 TT.
fn day_number(jd: f64) -> f64 {
    jd - 2451543.5
}

fn planet_elements(planet: Planet, d: f64) -> Elements {
    match planet {
        Planet::Mercury => Elements {
            n: 48.3313 + 3.24587e-5 * d,
            i: 7.0047 + 5.00e-8 * d,
            w: 29.1241 + 1.01444e-5 * d,
            a: 0.387098,
            e: 0.205635 + 5.59e-10 * d,
            m: 168.6562 + 4.0923344368 * d,
        },
        Planet::Venus => Elements {
            n: 76.6799 + 2.46590e-5 * d,
            i: 3.3946 + 2.75e-8 * d,
            w: 54.8910 + 1.38374e-5 * d,
            a: 0.723330,
            e: 0.006773 - 1.302e-9 * d,
            m: 48.0052 + 1.6021302244 * d,
        },
        Planet::Mars => Elements {
            n: 49.5574 + 2.11081e-5 * d,
            i: 1.8497 - 1.78e-8 * d,
            w: 286.5016 + 2.92961e-5 * d,
            a: 1.523688,
            e: 0.093405 + 2.516e-9 * d,
            m: 18.6021 + 0.5240207766 * d,
        },
        Planet::Jupiter => Elements {
            n: 100.4542 + 2.76854e-5 * d,
            i: 1.3030 - 1.557e-7 * d,
            w: 273.8777 + 1.64505e-5 * d,
            a: 5.20256,
            e: 0.048498 + 4.469e-9 * d,
            m: 19.8950 + 0.0830853001 * d,
        },
        Planet::Saturn => Elements {
            n: 113.6634 + 2.38980e-5 * d,
            i: 2.4886 - 1.081e-7 * d,
            w: 339.3939 + 2.97661e-5 * d,
            a: 9.55475,
            e: 0.055546 - 9.499e-9 * d,
            m: 316.9670 + 0.0334442282 * d,
        },
        Planet::Uranus => Elements {
            n: 74.0005 + 1.3978e-5 * d,
            i: 0.7733 + 1.9e-8 * d,
            w: 96.6612 + 3.0565e-5 * d,
            a: 19.18171 - 1.55e-8 * d,
            e: 0.047318 + 7.45e-9 * d,
            m: 142.5905 + 0.011725806 * d,
        },
        Planet::Neptune => Elements {
            n: 131.7806 + 3.0173e-5 * d,
            i: 1.7700 - 2.55e-7 * d,
            w: 272.8461 - 6.027e-6 * d,
            a: 30.05826 + 3.313e-8 * d,
            e: 0.008606 + 2.15e-9 * d,
            m: 260.2471 + 0.005995147 * d,
        },
    }
}

/// Solve Kepler's equation E - e·sin(E) = M by Newton iteration (radians).
fn eccentric_anomaly(m_rad: f64, e: f64) -> f64 {
    let mut big_e = m_rad + e * m_rad.sin() * (1.0 + e * m_rad.cos());
    for _ in 0..20 {
        let delta = (big_e - e * big_e.sin() - m_rad) / (1.0 - e * big_e.cos());
        big_e -= delta;
        if delta.abs() < 1e-9 {
            break;
        }
    }
    big_e
}

/// Heliocentric ecliptic position (longitude °, latitude °, radius AU).
fn heliocentric_position(planet: Planet, d: f64) -> (f64, f64, f64) {
    let el = planet_elements(planet, d);
    let m_rad = normalize_degrees(el.m) * DEG_TO_RAD;
    let big_e = eccentric_anomaly(m_rad, el.e);

    let xv = el.a * (big_e.cos() - el.e);
    let yv = el.a * ((1.0 - el.e * el.e).sqrt() * big_e.sin());
    let v = yv.atan2(xv); // true anomaly (radians)
    let r = (xv * xv + yv * yv).sqrt();

    let n_rad = el.n * DEG_TO_RAD;
    let i_rad = el.i * DEG_TO_RAD;
    let vw = v + el.w * DEG_TO_RAD;

    let xh = r * (n_rad.cos() * vw.cos() - n_rad.sin() * vw.sin() * i_rad.cos());
    let yh = r * (n_rad.sin() * vw.cos() + n_rad.cos() * vw.sin() * i_rad.cos());
    let zh = r * (vw.sin() * i_rad.sin());

    let mut lon = yh.atan2(xh) * RAD_TO_DEG;
    let mut lat = (zh / r).asin() * RAD_TO_DEG;

    // Major perturbations among Jupiter, Saturn, and Uranus (degrees).
    if matches!(planet, Planet::Jupiter | Planet::Saturn | Planet::Uranus) {
        let mj = normalize_degrees(planet_elements(Planet::Jupiter, d).m) * DEG_TO_RAD;
        let ms = normalize_degrees(planet_elements(Planet::Saturn, d).m) * DEG_TO_RAD;
        let deg = DEG_TO_RAD;
        match planet {
            Planet::Jupiter => {
                lon += -0.332 * (2.0 * mj - 5.0 * ms - 67.6 * deg).sin()
                    - 0.056 * (2.0 * mj - 2.0 * ms + 21.0 * deg).sin()
                    + 0.042 * (3.0 * mj - 5.0 * ms + 21.0 * deg).sin()
                    - 0.036 * (mj - 2.0 * ms).sin()
                    + 0.022 * (mj - ms).cos()
                    + 0.023 * (2.0 * mj - 3.0 * ms + 52.0 * deg).sin()
                    - 0.016 * (mj - 5.0 * ms - 69.0 * deg).sin();
            }
            Planet::Saturn => {
                lon += 0.812 * (2.0 * mj - 5.0 * ms - 67.6 * deg).sin()
                    - 0.229 * (2.0 * mj - 4.0 * ms - 2.0 * deg).cos()
                    + 0.119 * (mj - 2.0 * ms - 3.0 * deg).sin()
                    + 0.046 * (2.0 * mj - 6.0 * ms - 69.0 * deg).sin()
                    + 0.014 * (mj - 3.0 * ms + 32.0 * deg).sin();
                lat += -0.020 * (2.0 * mj - 4.0 * ms - 2.0 * deg).cos()
                    + 0.018 * (2.0 * mj - 6.0 * ms - 49.0 * deg).sin();
            }
            Planet::Uranus => {
                let mu = normalize_degrees(planet_elements(Planet::Uranus, d).m) * DEG_TO_RAD;
                lon += 0.040 * (ms - 2.0 * mu + 6.0 * deg).sin()
                    + 0.035 * (ms - 3.0 * mu + 33.0 * deg).sin()
                    - 0.015 * (mj - mu + 20.0 * deg).sin();
            }
            _ => unreachable!(),
        }
    }

    (normalize_degrees(lon), lat, r)
}

/// Earth–sun vector: geocentric ecliptic longitude of the sun (°) and
/// the Earth–sun distance (AU).
fn sun_position_ecliptic(d: f64) -> (f64, f64) {
    let w = 282.9404 + 4.70935e-5 * d;
    let e = 0.016709 - 1.151e-9 * d;
    let m = normalize_degrees(356.0470 + 0.9856002585 * d);

    let m_rad = m * DEG_TO_RAD;
    let big_e = eccentric_anomaly(m_rad, e);

    let xv = big_e.cos() - e;
    let yv = (1.0 - e * e).sqrt() * big_e.sin();
    let v = yv.atan2(xv) * RAD_TO_DEG;
    let r = (xv * xv + yv * yv).sqrt();

    (normalize_degrees(v + w), r)
}

/// Geocentric ecliptic state of a planet at a Julian Day:
/// (longitude °, latitude °, geocentric distance AU, heliocentric distance AU,
/// Earth–sun distance AU).
fn geocentric_ecliptic(planet: Planet, jd: f64) -> (f64, f64, f64, f64, f64) {
    let d = day_number(jd);

    let (hl, hb, r) = heliocentric_position(planet, d);
    let (sun_lon, rs) = sun_position_ecliptic(d);

    let hl_rad = hl * DEG_TO_RAD;
    let hb_rad = hb * DEG_TO_RAD;
    let xh = r * hb_rad.cos() * hl_rad.cos();
    let yh = r * hb_rad.cos() * hl_rad.sin();
    let zh = r * hb_rad.sin();

    let sun_lon_rad = sun_lon * DEG_TO_RAD;
    let xs = rs * sun_lon_rad.cos();
    let ys = rs * sun_lon_rad.sin();

    let xg = xh + xs;
    let yg = yh + ys;
    let zg = zh;

    let dist = (xg * xg + yg * yg + zg * zg).sqrt();
    let lon = normalize_degrees(yg.atan2(xg) * RAD_TO_DEG);
    let lat = (zg / dist).asin() * RAD_TO_DEG;

    (lon, lat, dist, r, rs)
}

/// Approximate visual magnitude from heliocentric distance `r`, geocentric
/// distance `dist`, phase angle `fv` (degrees), and for Saturn the ring
/// opening angle derived from the geocentric ecliptic coordinates.
fn visual_magnitude(planet: Planet, r: f64, dist: f64, fv: f64, lon: f64, lat: f64, d: f64) -> f64 {
    let base = 5.0 * (r * dist).log10();
    match planet {
        Planet::Mercury => -0.36 + base + 0.027 * fv + 2.2e-13 * fv.powi(6),
        Planet::Venus => -4.34 + base + 0.013 * fv + 4.2e-7 * fv.powi(3),
        Planet::Mars => -1.51 + base + 0.016 * fv,
        Planet::Jupiter => -9.25 + base + 0.014 * fv,
        Planet::Saturn => {
            // Ring plane geometry (Schlyter): tilt of the rings as seen from Earth.
            let ir = 28.06 * DEG_TO_RAD;
            let nr = (169.51 + 3.82e-5 * d) * DEG_TO_RAD;
            let lat_rad = lat * DEG_TO_RAD;
            let lon_rad = lon * DEG_TO_RAD;
            let b =
                (lat_rad.sin() * ir.cos() - lat_rad.cos() * ir.sin() * (lon_rad - nr).sin()).asin();
            let ring_magn = -2.6 * b.sin().abs() + 1.2 * b.sin().powi(2);
            -9.0 + base + 0.044 * fv + ring_magn
        }
        Planet::Uranus => -7.15 + base + 0.001 * fv,
        Planet::Neptune => -6.90 + base + 0.001 * fv,
    }
}

/// Calculate the apparent position of a planet at a specific time and location.
///
/// # Examples
///
/// ```
/// use solunatus::prelude::*;
/// use solunatus::astro::planets::{planet_position, Planet};
/// use chrono::Local;
/// use chrono_tz::America::New_York;
///
/// let location = Location::new(40.7128, -74.0060).unwrap();
/// let now = Local::now().with_timezone(&New_York);
/// let venus = planet_position(Planet::Venus, &location, &now);
///
/// println!("Venus: Alt {:.1}°, Az {:.0}°, mag {:.1}", venus.altitude, venus.azimuth, venus.magnitude);
/// ```
#[must_use]
pub fn planet_position<T: TimeZone>(
    planet: Planet,
    location: &Location,
    dt: &DateTime<T>,
) -> PlanetPosition {
    let jd = julian_day(dt);
    let t = julian_century(jd);
    let d = day_number(jd);

    let (lon, lat, dist, r, rs) = geocentric_ecliptic(planet, jd);

    // Phase angle from the triangle sun–planet–earth.
    let cos_fv = ((r * r + dist * dist - rs * rs) / (2.0 * r * dist)).clamp(-1.0, 1.0);
    let fv = cos_fv.acos() * RAD_TO_DEG;

    let magnitude = visual_magnitude(planet, r, dist, fv, lon, lat, d);

    // Solar elongation: angle between the geocentric directions of planet and sun.
    let (sun_lon, _) = sun_position_ecliptic(d);
    let lat_rad = lat * DEG_TO_RAD;
    let cos_elong = (lat_rad.cos() * ((lon - sun_lon) * DEG_TO_RAD).cos()).clamp(-1.0, 1.0);
    let elongation = cos_elong.acos() * RAD_TO_DEG;

    // Ecliptic → equatorial.
    let ecl = (23.4393 - 3.563e-7 * d) * DEG_TO_RAD;
    let lon_rad = lon * DEG_TO_RAD;
    let xg = lat_rad.cos() * lon_rad.cos();
    let yg = lat_rad.cos() * lon_rad.sin();
    let zg = lat_rad.sin();

    let xe = xg;
    let ye = yg * ecl.cos() - zg * ecl.sin();
    let ze = yg * ecl.sin() + zg * ecl.cos();

    let ra = ye.atan2(xe) * RAD_TO_DEG;
    let dec = ze.atan2((xe * xe + ye * ye).sqrt());

    // Equatorial → horizontal via Greenwich Mean Sidereal Time.
    let gmst = 280.46061837 + 360.98564736629 * (jd - 2451545.0) + 0.000387933 * t * t
        - t * t * t / 38710000.0;
    let lst = normalize_degrees(gmst + location.longitude.value());
    let ha = normalize_degrees_signed(lst - ra) * DEG_TO_RAD;

    let phi = location.latitude.value() * DEG_TO_RAD;
    let sin_alt = phi.sin() * dec.sin() + phi.cos() * dec.cos() * ha.cos();
    let altitude = sin_alt.asin() * RAD_TO_DEG;

    let altitude_rad = altitude * DEG_TO_RAD;
    let cos_az = (dec.sin() - phi.sin() * altitude_rad.sin()) / (phi.cos() * altitude_rad.cos());
    let sin_az = -ha.sin() * dec.cos() / altitude_rad.cos();
    let mut azimuth = sin_az.atan2(cos_az) * RAD_TO_DEG;
    if azimuth < 0.0 {
        azimuth += 360.0;
    }

    PlanetPosition {
        altitude,
        azimuth,
        distance_au: dist,
        magnitude,
        elongation,
    }
}

fn refine_planet_crossing<T: TimeZone>(
    planet: Planet,
    location: &Location,
    mut low: DateTime<T>,
    mut high: DateTime<T>,
    seek_rising: bool,
) -> DateTime<T> {
    while (high.timestamp() - low.timestamp()).abs() > 1 {
        let span_secs = high.timestamp() - low.timestamp();
        let mid = low
            .clone()
            .checked_add_signed(Duration::seconds(span_secs / 2))
            .unwrap();
        let mid_alt = planet_position(planet, location, &mid).altitude - RISE_SET_ALTITUDE;

        if seek_rising {
            if mid_alt >= 0.0 {
                high = mid;
            } else {
                low = mid;
            }
        } else if mid_alt <= 0.0 {
            high = mid;
        } else {
            low = mid;
        }
    }

    high
}

/// Calculate the time of a planetary event (rise or set) for a given date.
///
/// Finds when the planet crosses the refracted horizon, sweeping the local
/// day in 10-minute steps and refining each crossing to one-second resolution.
///
/// # Returns
///
/// - `Some(DateTime)` - The time when the event occurs in the input timezone
/// - `None` - The event doesn't occur on this date (e.g. circumpolar)
///
/// # Examples
///
/// ```
/// use solunatus::prelude::*;
/// use solunatus::astro::planets::{planet_event_time, Planet, PlanetEvent};
/// use chrono::Local;
/// use chrono_tz::America::New_York;
///
/// let location = Location::new(40.7128, -74.0060).unwrap();
/// let now = Local::now().with_timezone(&New_York);
///
/// if let Some(rise) = planet_event_time(Planet::Jupiter, &location, &now, PlanetEvent::Rise) {
///     println!("Jupiter rises at {}", rise.format("%H:%M"));
/// }
/// ```
pub fn planet_event_time<T: TimeZone>(
    planet: Planet,
    location: &Location,
    date: &DateTime<T>,
    event: PlanetEvent,
) -> Option<DateTime<T>> {
    let seek_rising = event == PlanetEvent::Rise;

    let tz = date.timezone();
    let start_naive = date.date_naive().and_hms_opt(0, 0, 0)?;
    let start = match tz.from_local_datetime(&start_naive) {
        chrono::LocalResult::Single(dt) => dt,
        chrono::LocalResult::Ambiguous(early, _) => early,
        chrono::LocalResult::None => tz
            .from_local_datetime(&(start_naive + Duration::hours(1)))
            .earliest()?,
    };
    let end = start.clone() + Duration::hours(24);

    let step = Duration::minutes(10);
    let mut prev_dt = start.clone();
    let mut prev_alt = planet_position(planet, location, &prev_dt).altitude - RISE_SET_ALTITUDE;

    loop {
        let current = prev_dt.clone().checked_add_signed(step)?;
        if current > end {
            break;
        }
        let alt = planet_position(planet, location, &current).altitude - RISE_SET_ALTITUDE;
        let crossing = if seek_rising {
            prev_alt <= 0.0 && alt >= 0.0
        } else {
            prev_alt >= 0.0 && alt <= 0.0
        };

        if crossing {
            return Some(refine_planet_crossing(
                planet,
                location,
                prev_dt,
                current,
                seek_rising,
            ));
        }

        prev_dt = current;
        prev_alt = alt;
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    /// The Jupiter–Saturn great conjunction: on 2020-12-21 the two planets
    /// were separated by only ~0.1°. The truncated theory should agree on
    /// their geocentric ecliptic longitudes to well within half a degree.
    #[test]
    fn great_conjunction_2020() {
        let dt = Utc.with_ymd_and_hms(2020, 12, 21, 18, 0, 0).unwrap();
        let jd = julian_day(&dt);

        let (jup_lon, _, _, _, _) = geocentric_ecliptic(Planet::Jupiter, jd);
        let (sat_lon, _, _, _, _) = geocentric_ecliptic(Planet::Saturn, jd);

        let separation = normalize_degrees_signed(jup_lon - sat_lon).abs();
        assert!(
            separation < 0.5,
            "Jupiter–Saturn separation {separation:.3}° too large for the great conjunction"
        );
    }

    /// Venus never strays more than ~47° from the sun.
    #[test]
    fn venus_elongation_bounded() {
        let location = Location::new(40.7128, -74.0060).unwrap();
        for month in 1..=12 {
            let dt = Utc.with_ymd_and_hms(2025, month, 15, 0, 0, 0).unwrap();
            let pos = planet_position(Planet::Venus, &location, &dt);
            assert!(
                pos.elongation <= 48.5,
                "Venus elongation {:.1}° in month {month} exceeds maximum",
                pos.elongation
            );
        }
    }

    /// Mercury never strays more than ~28° from the sun.
    #[test]
    fn mercury_elongation_bounded() {
        let location = Location::new(40.7128, -74.0060).unwrap();
        for month in 1..=12 {
            let dt = Utc.with_ymd_and_hms(2025, month, 15, 0, 0, 0).unwrap();
            let pos = planet_position(Planet::Mercury, &location, &dt);
            assert!(
                pos.elongation <= 28.5,
                "Mercury elongation {:.1}° in month {month} exceeds maximum",
                pos.elongation
            );
        }
    }

    #[test]
    fn geocentric_distances_within_physical_bounds() {
        let location = Location::new(0.0, 0.0).unwrap();
        let ranges = [
            (Planet::Mercury, 0.50, 1.50),
            (Planet::Venus, 0.25, 1.75),
            (Planet::Mars, 0.36, 2.70),
            (Planet::Jupiter, 3.9, 6.5),
            (Planet::Saturn, 7.9, 11.1),
            (Planet::Uranus, 17.0, 21.2),
            (Planet::Neptune, 28.7, 31.5),
        ];
        for month in [1u32, 4, 7, 10] {
            let dt = Utc.with_ymd_and_hms(2026, month, 1, 0, 0, 0).unwrap();
            for (planet, min_au, max_au) in ranges {
                let pos = planet_position(planet, &location, &dt);
                assert!(
                    (min_au..=max_au).contains(&pos.distance_au),
                    "{} distance {:.2} AU outside [{min_au}, {max_au}]",
                    planet.name(),
                    pos.distance_au
                );
            }
        }
    }

    #[test]
    fn rise_event_crosses_horizon_threshold() {
        use chrono_tz::America::New_York;

        let location = Location::new(40.7128, -74.0060).unwrap();
        let date = Utc
            .with_ymd_and_hms(2026, 3, 15, 12, 0, 0)
            .unwrap()
            .with_timezone(&New_York);

        for planet in Planet::ALL {
            let rise = planet_event_time(planet, &location, &date, PlanetEvent::Rise);
            let set = planet_event_time(planet, &location, &date, PlanetEvent::Set);
            // At 40°N every planet rises and sets daily.
            let rise = rise.unwrap_or_else(|| panic!("{} should rise", planet.name()));
            let set = set.unwrap_or_else(|| panic!("{} should set", planet.name()));

            let alt_at_rise = planet_position(planet, &location, &rise).altitude;
            assert!(
                (alt_at_rise - RISE_SET_ALTITUDE).abs() < 0.05,
                "{} altitude at rise = {alt_at_rise:.3}°, expected ≈ {RISE_SET_ALTITUDE}",
                planet.name()
            );

            let alt_at_set = planet_position(planet, &location, &set).altitude;
            assert!(
                (alt_at_set - RISE_SET_ALTITUDE).abs() < 0.05,
                "{} altitude at set = {alt_at_set:.3}°",
                planet.name()
            );
        }
    }
}
