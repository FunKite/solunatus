//! Conversion of event times from TT to UTC, with an explicit fallback range.

use chrono::{DateTime, Utc};

const SECONDS_PER_DAY: f64 = 86_400.0;
const TT_MINUS_TAI: f64 = 32.184;

// UTC Julian day at each effective date, and TAI−UTC in seconds.
// Source: https://maia.usno.navy.mil/ser7/tai-utc.dat (checked 2026-11-09).
// Before 1972 UTC used rate adjustments; those are outside this table.
const LEAP_OFFSETS: &[(f64, f64)] = &[
    (2441317.5, 10.0), // 1972-01-01
    (2441499.5, 11.0), // 1972-07-01
    (2441683.5, 12.0), // 1973-01-01
    (2442048.5, 13.0), // 1974-01-01
    (2442413.5, 14.0), // 1975-01-01
    (2442778.5, 15.0), // 1976-01-01
    (2443144.5, 16.0), // 1977-01-01
    (2443509.5, 17.0), // 1978-01-01
    (2443874.5, 18.0), // 1979-01-01
    (2444239.5, 19.0), // 1980-01-01
    (2444786.5, 20.0), // 1981-07-01
    (2445151.5, 21.0), // 1982-07-01
    (2445516.5, 22.0), // 1983-07-01
    (2446247.5, 23.0), // 1985-07-01
    (2447161.5, 24.0), // 1988-01-01
    (2447892.5, 25.0), // 1990-01-01
    (2448257.5, 26.0), // 1991-01-01
    (2448804.5, 27.0), // 1992-07-01
    (2449169.5, 28.0), // 1993-07-01
    (2449534.5, 29.0), // 1994-07-01
    (2450083.5, 30.0), // 1996-01-01
    (2450630.5, 31.0), // 1997-07-01
    (2451179.5, 32.0), // 1999-01-01
    (2453736.5, 33.0), // 2006-01-01
    (2454832.5, 34.0), // 2009-01-01
    (2456109.5, 35.0), // 2012-07-01
    (2457204.5, 36.0), // 2015-07-01
    (2457754.5, 37.0), // 2017-01-01
];

// IERS Bulletin C 72 (2026-07-06) announces no leap at December 2026.
// The next undecided boundary is 2027-07-01. Do not extrapolate a permanent
// 69.184-second offset into the future; refresh this bound with Bulletin C.
// https://hpiers.obspm.fr/iers/bul/bulc/bulletinc.72
const KNOWN_UTC_END_JD: f64 = 2461587.5;

fn known_tt_minus_utc(jde: f64) -> Option<f64> {
    let last_offset = TT_MINUS_TAI + LEAP_OFFSETS.last()?.1;
    if !jde.is_finite() || jde >= KNOWN_UTC_END_JD + last_offset / SECONDS_PER_DAY {
        return None;
    }
    LEAP_OFFSETS
        .iter()
        .rev()
        .find_map(|&(utc_jd, tai_minus_utc)| {
            let offset = TT_MINUS_TAI + tai_minus_utc;
            (jde >= utc_jd + offset / SECONDS_PER_DAY).then_some(offset)
        })
}

/// Convert a TT Julian ephemeris day to a civil event timestamp.
///
/// Within the verified 1972-01-01 through 2027-06-30 interval this applies
/// TT−UTC = 32.184 + (TAI−UTC), with each leap boundary compared in TT.
/// Outside it, retain the existing ΔT estimate as an approximate UT1 value
/// carried by DateTime<Utc>. That fallback is not exact historical/future UTC.
pub(crate) fn tt_jde_to_datetime(jde: f64, year: i32) -> Option<DateTime<Utc>> {
    if !jde.is_finite() {
        return None;
    }
    if let Some(offset) = known_tt_minus_utc(jde) {
        // During an inserted second, POSIX/Julian-day arithmetic alone would
        // label it as the next midnight. Chrono can represent 23:59:60 using
        // a nanosecond value >= 1e9 on the preceding second.
        for &(utc_jd, tai_minus_utc) in LEAP_OFFSETS.iter().skip(1) {
            let boundary_tt = utc_jd + (TT_MINUS_TAI + tai_minus_utc) / SECONDS_PER_DAY;
            let remaining = (boundary_tt - jde) * SECONDS_PER_DAY;
            if remaining > 0.0 && remaining <= 1.0 {
                let midnight = ((utc_jd - 2440587.5) * SECONDS_PER_DAY).round() as i64;
                return DateTime::from_timestamp(midnight - 1, 1_000_000_000);
            }
        }
        super::moon::jd_to_datetime(jde - offset / SECONDS_PER_DAY)
    } else {
        let delta_t = super::seasons::delta_t_seconds(year);
        super::moon::jd_to_datetime(jde - delta_t / SECONDS_PER_DAY)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Timelike};

    #[test]
    fn modern_offset_uses_atomic_time_not_the_delta_t_prediction() {
        let tt = 2460676.5 + 0.5; // 2025-01-01 12:00 TT
        assert_eq!(known_tt_minus_utc(tt), Some(69.184));
        let converted = tt_jde_to_datetime(tt, 2025).unwrap();
        assert_eq!(
            converted.format("%Y-%m-%d %H:%M:%S").to_string(),
            "2025-01-01 11:58:50"
        );
        assert!((super::super::seasons::delta_t_seconds(2025) - 74.467375).abs() < 1e-8);
    }

    #[test]
    fn every_leap_offset_takes_effect_at_the_tt_boundary() {
        for (index, &(utc_jd, tai_minus_utc)) in LEAP_OFFSETS.iter().enumerate() {
            let offset = TT_MINUS_TAI + tai_minus_utc;
            let boundary_tt = utc_jd + offset / SECONDS_PER_DAY;
            assert_eq!(known_tt_minus_utc(boundary_tt), Some(offset));
            let before = known_tt_minus_utc(boundary_tt - 2.0 / SECONDS_PER_DAY);
            let expected = index
                .checked_sub(1)
                .map(|previous| TT_MINUS_TAI + LEAP_OFFSETS[previous].1);
            assert_eq!(before, expected);
        }
    }

    #[test]
    fn inserted_second_is_not_mislabeled_as_next_midnight() {
        let boundary = 2457754.5 + 69.184 / SECONDS_PER_DAY;
        let leap = tt_jde_to_datetime(boundary - 0.5 / SECONDS_PER_DAY, 2017).unwrap();
        assert_eq!(
            leap.format("%Y-%m-%d %H:%M:%S").to_string(),
            "2016-12-31 23:59:60"
        );
        assert_eq!(leap.nanosecond(), 1_000_000_000);
        let after = tt_jde_to_datetime(boundary + 0.5 / SECONDS_PER_DAY, 2017).unwrap();
        assert_eq!(after, Utc.with_ymd_and_hms(2017, 1, 1, 0, 0, 0).unwrap());
    }

    #[test]
    fn known_interval_has_an_explicit_expiry_and_preserves_approximate_fallback() {
        let end = super::super::julian_day(&Utc.with_ymd_and_hms(2027, 7, 1, 0, 0, 0).unwrap());
        assert_eq!(end, KNOWN_UTC_END_JD);
        let end_tt = end + 69.184 / SECONDS_PER_DAY;
        assert_eq!(
            known_tt_minus_utc(end_tt - 1.0 / SECONDS_PER_DAY),
            Some(69.184)
        );
        assert_eq!(known_tt_minus_utc(end_tt), None);
        for (jde, year) in [(2433282.5, 1950), (2469807.5, 2050)] {
            assert_eq!(known_tt_minus_utc(jde), None);
            assert_eq!(
                tt_jde_to_datetime(jde, year),
                super::super::moon::jd_to_datetime(
                    jde - super::super::seasons::delta_t_seconds(year) / SECONDS_PER_DAY
                )
            );
        }
        assert_eq!(tt_jde_to_datetime(f64::NAN, 2025), None);
    }
}
