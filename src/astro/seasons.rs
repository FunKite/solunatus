//! Equinox and solstice calculations.
//!
//! Computes the instants of the equinoxes and solstices using the algorithm
//! from Jean Meeus, "Astronomical Algorithms", 2nd Edition, Chapter 27
//! (mean-event polynomials plus 24 periodic correction terms).
//!
//! # Accuracy
//!
//! Within the years 1000–3000 the underlying algorithm is accurate to well
//! under a minute. Results are converted from Terrestrial Time to UTC with a
//! ΔT estimate that is precise for 1986–2150 and approximate outside that
//! range.

use super::moon::jd_to_datetime;
use super::{DEG_TO_RAD, julian_century};
use chrono::{DateTime, Datelike, TimeZone, Utc};

/// The four seasonal events of the tropical year.
///
/// Named by month rather than season so the terms apply in both hemispheres.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeasonalEventKind {
    /// March equinox (sun crosses the celestial equator northward)
    MarchEquinox,
    /// June solstice (sun at maximum northern declination)
    JuneSolstice,
    /// September equinox (sun crosses the celestial equator southward)
    SeptemberEquinox,
    /// December solstice (sun at maximum southern declination)
    DecemberSolstice,
}

impl SeasonalEventKind {
    /// Human-readable name (e.g. "March Equinox").
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            SeasonalEventKind::MarchEquinox => "March Equinox",
            SeasonalEventKind::JuneSolstice => "June Solstice",
            SeasonalEventKind::SeptemberEquinox => "September Equinox",
            SeasonalEventKind::DecemberSolstice => "December Solstice",
        }
    }
}

/// A seasonal event (equinox or solstice) with its UTC instant.
#[derive(Debug, Clone)]
pub struct SeasonalEvent {
    /// Which equinox or solstice this is
    pub kind: SeasonalEventKind,
    /// UTC time of the event
    pub datetime: DateTime<Utc>,
}

/// Periodic correction terms from Meeus Table 27.C: (A, B, C) with the
/// contribution A·cos(B + C·T) in units of 0.00001 day.
const PERIODIC_TERMS: &[(f64, f64, f64)] = &[
    (485.0, 324.96, 1934.136),
    (203.0, 337.23, 32964.467),
    (199.0, 342.08, 20.186),
    (182.0, 27.85, 445267.112),
    (156.0, 73.14, 45036.886),
    (136.0, 171.52, 22518.443),
    (77.0, 222.54, 65928.934),
    (74.0, 296.72, 3034.906),
    (70.0, 243.58, 9037.513),
    (58.0, 119.81, 33718.147),
    (52.0, 297.17, 150.678),
    (50.0, 21.02, 2281.226),
    (45.0, 247.54, 29929.562),
    (44.0, 325.15, 31555.956),
    (29.0, 60.93, 4443.417),
    (18.0, 155.12, 67555.328),
    (17.0, 288.79, 4562.452),
    (16.0, 198.04, 62894.029),
    (14.0, 199.76, 31436.921),
    (12.0, 95.39, 14577.848),
    (12.0, 287.11, 31931.756),
    (12.0, 320.81, 34777.259),
    (9.0, 227.73, 1222.114),
    (8.0, 15.45, 16859.074),
];

/// Mean event time JDE0 (Meeus Tables 27.A/27.B).
fn mean_event_jde(kind: SeasonalEventKind, year: i32) -> f64 {
    if year >= 1000 {
        // Table 27.B, years 1000 to 3000; Y = (year - 2000) / 1000
        let y = f64::from(year - 2000) / 1000.0;
        match kind {
            SeasonalEventKind::MarchEquinox => {
                2451623.80984 + y * (365242.37404 + y * (0.05169 + y * (-0.00411 - y * 0.00057)))
            }
            SeasonalEventKind::JuneSolstice => {
                2451716.56767 + y * (365241.62603 + y * (0.00325 + y * (0.00888 - y * 0.00030)))
            }
            SeasonalEventKind::SeptemberEquinox => {
                2451810.21715 + y * (365242.01767 + y * (-0.11575 + y * (0.00337 + y * 0.00078)))
            }
            SeasonalEventKind::DecemberSolstice => {
                2451900.05952 + y * (365242.74049 + y * (-0.06223 + y * (-0.00823 + y * 0.00032)))
            }
        }
    } else {
        // Table 27.A, years -1000 to +1000; Y = year / 1000
        let y = f64::from(year) / 1000.0;
        match kind {
            SeasonalEventKind::MarchEquinox => {
                1721139.29189 + y * (365242.13740 + y * (0.06134 + y * (0.00111 - y * 0.00071)))
            }
            SeasonalEventKind::JuneSolstice => {
                1721233.25401 + y * (365241.72562 + y * (-0.05323 + y * (0.00907 + y * 0.00025)))
            }
            SeasonalEventKind::SeptemberEquinox => {
                1721325.70455 + y * (365242.49558 + y * (-0.11677 + y * (-0.00297 + y * 0.00074)))
            }
            SeasonalEventKind::DecemberSolstice => {
                1721414.39987 + y * (365242.88257 + y * (-0.00769 + y * (-0.00933 - y * 0.00006)))
            }
        }
    }
}

/// Estimate ΔT = TT − UT in seconds for a given year.
///
/// Uses the Espenak–Meeus polynomial expressions for 1986–2150 and the
/// long-term parabolic fit elsewhere.
fn delta_t_seconds(year: i32) -> f64 {
    let y = f64::from(year);
    if (1986..2005).contains(&year) {
        let t = y - 2000.0;
        63.86
            + t * (0.3345
                + t * (-0.060374 + t * (0.0017275 + t * (0.000651814 + t * 0.00002373599))))
    } else if (2005..2050).contains(&year) {
        let t = y - 2000.0;
        62.92 + t * (0.32217 + t * 0.005589)
    } else if (2050..=2150).contains(&year) {
        let u = (y - 1820.0) / 100.0;
        -20.0 + 32.0 * u * u - 0.5628 * (2150.0 - y)
    } else {
        let u = (y - 1820.0) / 100.0;
        -20.0 + 32.0 * u * u
    }
}

/// Julian Ephemeris Day (TT) of a seasonal event.
fn seasonal_event_jde(kind: SeasonalEventKind, year: i32) -> f64 {
    let jde0 = mean_event_jde(kind, year);
    let t = julian_century(jde0);
    let w = (35999.373 * t - 2.47) * DEG_TO_RAD;
    let delta_lambda = 1.0 + 0.0334 * w.cos() + 0.0007 * (2.0 * w).cos();

    let s: f64 = PERIODIC_TERMS
        .iter()
        .map(|&(a, b, c)| a * ((b + c * t) * DEG_TO_RAD).cos())
        .sum();

    jde0 + 0.00001 * s / delta_lambda
}

/// Calculate all four seasonal events (equinoxes and solstices) for a year.
///
/// Returns the events in chronological order with UTC times.
///
/// # Examples
///
/// ```
/// use solunatus::astro::seasons::seasonal_events;
///
/// for event in seasonal_events(2026) {
///     println!("{}: {}", event.kind.name(), event.datetime.format("%Y-%m-%d %H:%M UTC"));
/// }
/// ```
#[must_use]
pub fn seasonal_events(year: i32) -> Vec<SeasonalEvent> {
    [
        SeasonalEventKind::MarchEquinox,
        SeasonalEventKind::JuneSolstice,
        SeasonalEventKind::SeptemberEquinox,
        SeasonalEventKind::DecemberSolstice,
    ]
    .into_iter()
    .filter_map(|kind| {
        let jde = seasonal_event_jde(kind, year);
        let jd_utc = jde - delta_t_seconds(year) / 86400.0;
        jd_to_datetime(jd_utc).map(|datetime| SeasonalEvent { kind, datetime })
    })
    .collect()
}

/// Find the next `count` seasonal events strictly after a given time.
///
/// # Examples
///
/// ```
/// use solunatus::astro::seasons::next_seasonal_events;
/// use chrono::Utc;
///
/// let upcoming = next_seasonal_events(&Utc::now(), 2);
/// assert_eq!(upcoming.len(), 2);
/// ```
#[must_use]
pub fn next_seasonal_events<T: TimeZone>(after: &DateTime<T>, count: usize) -> Vec<SeasonalEvent> {
    let after_utc = after.with_timezone(&Utc);
    let year = after_utc.year();

    let mut events: Vec<SeasonalEvent> = (year..=year + 1)
        .flat_map(seasonal_events)
        .filter(|event| event.datetime > after_utc)
        .collect();
    events.sort_by_key(|event| event.datetime);
    events.truncate(count);
    events
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Meeus Example 27.a: the June solstice of 1962 occurs at
    /// JDE 2437837.39245 (1962 June 21, 21:24:42 TT).
    #[test]
    fn meeus_example_27a_june_solstice_1962() {
        let jde = seasonal_event_jde(SeasonalEventKind::JuneSolstice, 1962);
        assert!(
            (jde - 2437837.39245).abs() < 0.0005,
            "JDE {jde} differs from Meeus reference 2437837.39245"
        );
    }

    #[test]
    fn seasonal_events_are_chronological_and_in_expected_months() {
        let events = seasonal_events(2026);
        assert_eq!(events.len(), 4);
        assert!(
            events
                .windows(2)
                .all(|pair| pair[0].datetime < pair[1].datetime)
        );

        let months: Vec<u32> = events.iter().map(|e| e.datetime.month()).collect();
        assert_eq!(months, vec![3, 6, 9, 12]);

        // Equinoxes/solstices always fall on days 19-23 of their month.
        for event in &events {
            let day = event.datetime.day();
            assert!((19..=23).contains(&day), "{:?} on day {day}", event.kind);
        }
    }

    #[test]
    fn next_seasonal_events_crosses_year_boundary() {
        let after = Utc.with_ymd_and_hms(2026, 11, 1, 0, 0, 0).unwrap();
        let upcoming = next_seasonal_events(&after, 2);
        assert_eq!(upcoming.len(), 2);
        assert_eq!(upcoming[0].kind, SeasonalEventKind::DecemberSolstice);
        assert_eq!(upcoming[1].kind, SeasonalEventKind::MarchEquinox);
        assert_eq!(upcoming[0].datetime.year(), 2026);
        assert_eq!(upcoming[1].datetime.year(), 2027);
    }
}
