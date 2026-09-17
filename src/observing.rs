//! A compact, offline observing plan for one local noon-to-noon interval.

use anyhow::{Context, Result, ensure};
use chrono::{DateTime, Datelike, Duration, NaiveDate, TimeZone};
use chrono_tz::Tz;
use serde::Serialize;
use solunatus::{
    astro::{Location, moon, planets, sun},
    events,
};
use std::fmt::Write;

#[derive(Debug, Serialize)]
struct Period {
    start: DateTime<Tz>,
    end: DateTime<Tz>,
}

#[derive(Serialize)]
struct PlanetSnapshot {
    name: &'static str,
    altitude_degrees: f64,
    azimuth_degrees: f64,
}

#[derive(Serialize)]
struct NightPlan {
    night_of: NaiveDate,
    place: String,
    timezone: String,
    interval: Period,
    evening_golden_hour: Option<Period>,
    evening_blue_hour: Option<Period>,
    sunset: Option<DateTime<Tz>>,
    astronomical_dusk: Option<DateTime<Tz>>,
    astronomical_dawn: Option<DateTime<Tz>>,
    sunrise: Option<DateTime<Tz>>,
    first_moon_free_window: Option<Period>,
    moon_free_window_clipped: bool,
    snapshot_at: DateTime<Tz>,
    moon_phase: String,
    moon_illumination_percent: f64,
    moon_altitude_degrees: f64,
    planets_above_horizon: Vec<PlanetSnapshot>,
}

fn local_noon(tz: Tz, date: NaiveDate) -> Result<DateTime<Tz>> {
    tz.from_local_datetime(&date.and_hms_opt(12, 0, 0).context("Invalid noon")?)
        .single()
        .context("Local noon is missing or ambiguous for this date and timezone")
}

fn build_plan(
    location: &Location,
    tz: Tz,
    city: Option<&str>,
    date: NaiveDate,
) -> Result<NightPlan> {
    ensure!(
        (-999..=3000).contains(&date.year()),
        "Night date must be between astronomical years -0999 and 3000"
    );
    let start = local_noon(tz, date)?;
    // Resolve the next local noon separately: DST nights can be 23 or 25 hours.
    let end = local_noon(tz, date.succ_opt().context("Date overflow")?)?;
    // Solar routines use longitude-based calculation dates, which can differ
    // from the civil date across the international date line. Search adjacent
    // dates and select crossings in the requested local interval.
    let crossing = |event| {
        (-2..=2)
            .filter_map(|day| start.checked_add_signed(Duration::days(day)))
            .filter_map(|reference| sun::solar_event_time(location, &reference, event))
            .filter(|t| *t >= start && *t < end)
            .min()
    };
    let sunset = crossing(sun::SolarEvent::Sunset);
    let dusk = crossing(sun::SolarEvent::AstronomicalDusk);
    let dawn = crossing(sun::SolarEvent::AstronomicalDawn);
    let sunrise = crossing(sun::SolarEvent::Sunrise);
    let period = |start_event, end_event| match (crossing(start_event), crossing(end_event)) {
        (Some(start), Some(end)) if start < end => Some(Period { start, end }),
        _ => None,
    };
    let evening_golden_hour = period(
        sun::SolarEvent::GoldenDuskStart,
        sun::SolarEvent::GoldenDuskEnd,
    );
    let evening_blue_hour = period(sun::SolarEvent::GoldenDuskEnd, sun::SolarEvent::CivilDusk);
    // The shared search scans 36 hours. Never advertise tomorrow night's
    // darkness as belonging to this plan; clip continuous polar darkness.
    let window = events::next_dark_window(location, &start).filter(|(s, _)| *s < end);
    let clipped = window.is_some_and(|(_, e)| e.is_none_or(|e| e > end));
    let first_moon_free_window = window.map(|(s, e)| Period {
        start: s,
        end: e.unwrap_or(end).min(end),
    });
    let snapshot_at = first_moon_free_window.as_ref().map_or_else(
        || {
            dusk.map(|t| (t + Duration::hours(1)).min(end - Duration::seconds(1)))
                .unwrap_or(start + (end - start) / 2)
        },
        |p| p.start + (p.end - p.start) / 2,
    );
    let moon = moon::lunar_position(location, &snapshot_at);
    let planets_above_horizon = planets::Planet::ALL
        .into_iter()
        .filter_map(|planet| {
            let position = planets::planet_position(planet, location, &snapshot_at);
            (position.altitude > 0.0).then_some(PlanetSnapshot {
                name: planet.name(),
                altitude_degrees: position.altitude,
                azimuth_degrees: position.azimuth,
            })
        })
        .collect();
    Ok(NightPlan {
        night_of: date,
        place: city.map(str::to_owned).unwrap_or_else(|| {
            format!(
                "{:.4}, {:.4}",
                location.latitude.value(),
                location.longitude.value()
            )
        }),
        timezone: tz.name().to_owned(),
        interval: Period { start, end },
        evening_golden_hour,
        evening_blue_hour,
        sunset,
        astronomical_dusk: dusk,
        astronomical_dawn: dawn,
        sunrise,
        first_moon_free_window,
        moon_free_window_clipped: clipped,
        snapshot_at,
        moon_phase: moon::phase_name(moon.phase_angle).to_owned(),
        moon_illumination_percent: moon.illumination * 100.0,
        moon_altitude_degrees: moon.altitude,
        planets_above_horizon,
    })
}

fn time(t: DateTime<Tz>) -> String {
    t.format("%a %H:%M %Z").to_string()
}

fn render(plan: &NightPlan) -> String {
    let mut out = format!(
        "SOLUNATUS / NIGHT PLAN\n{} · Night of {}\n{} · local noon to next noon\n\n",
        plan.place, plan.night_of, plan.timezone
    );
    let period_text = |p: &Period| format!("{} → {}", time(p.start), time(p.end));
    for (label, value) in [
        ("Golden hour", plan.evening_golden_hour.as_ref()),
        ("Blue hour", plan.evening_blue_hour.as_ref()),
    ] {
        let _ = writeln!(
            out,
            "{label:<18}{}",
            value
                .map(&period_text)
                .unwrap_or_else(|| "No interval in this plan".into())
        );
    }
    for (label, value) in [
        ("Sunset", plan.sunset),
        ("Darkness begins", plan.astronomical_dusk),
        ("Darkness ends", plan.astronomical_dawn),
        ("Sunrise", plan.sunrise),
    ] {
        let _ = writeln!(
            out,
            "{label:<18}{}",
            value
                .map(time)
                .unwrap_or_else(|| "No crossing in this plan".into())
        );
    }
    out.push_str("\nFIRST MOON-FREE DARK WINDOW\n");
    if let Some(p) = &plan.first_moon_free_window {
        // Match the minute precision of the displayed endpoints. Unix minutes
        // retain elapsed-time semantics across DST; Euclidean division also
        // handles timestamps before 1970 correctly. JSON retains exact times.
        let mins = p.end.timestamp().div_euclid(60) - p.start.timestamp().div_euclid(60);
        let _ = writeln!(
            out,
            "{}  ({}h {:02}m)",
            period_text(p),
            mins / 60,
            mins % 60
        );
        if plan.moon_free_window_clipped {
            out.push_str("Darkness continues beyond the plan; end is clipped to local noon.\n");
        }
    } else {
        out.push_str("None in this noon-to-noon interval.\n");
    }
    out.push_str("Sun below -18°; Moon below horizon with a 15-minute buffer.\n");
    let _ = writeln!(out, "\nSKY SNAPSHOT / {}", time(plan.snapshot_at));
    let _ = writeln!(
        out,
        "Moon: {} · {:.0}% illuminated · {:.0}° altitude",
        plan.moon_phase, plan.moon_illumination_percent, plan.moon_altitude_degrees
    );
    out.push_str("Planets above the horizon at this time:\n");
    if plan.planets_above_horizon.is_empty() {
        out.push_str("  None\n");
    }
    for p in &plan.planets_above_horizon {
        let _ = writeln!(
            out,
            "  {:<9} {:>5.1}° altitude   {:>5.1}° azimuth",
            p.name, p.altitude_degrees, p.azimuth_degrees
        );
    }
    out.push_str("\nTimes are estimates. Weather, terrain and light pollution are not included.\n");
    out
}

pub fn generate(
    location: &Location,
    tz: Tz,
    city: Option<&str>,
    date: NaiveDate,
    json: bool,
) -> Result<String> {
    let plan = build_plan(location, tz, city, date)?;
    if json {
        Ok(serde_json::to_string_pretty(&plan)?)
    } else {
        Ok(render(&plan))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono_tz::{America::New_York, Europe::Oslo};

    #[test]
    fn tucson_duration_matches_displayed_endpoints() {
        let p = build_plan(
            &Location::new(32.2226, -110.9747).unwrap(),
            chrono_tz::America::Phoenix,
            Some("Tucson"),
            NaiveDate::from_ymd_opt(2026, 9, 15).unwrap(),
        )
        .unwrap();
        assert!(render(&p).contains("Tue 21:08 MST → Wed 04:46 MST  (7h 38m)"));
    }

    #[test]
    fn displayed_duration_handles_dst_and_pre_epoch_times() {
        let mut p = build_plan(
            &Location::new(40.7128, -74.0060).unwrap(),
            New_York,
            None,
            NaiveDate::from_ymd_opt(2026, 3, 7).unwrap(),
        )
        .unwrap();
        for (start, end, expected) in [
            (
                "2026-03-08T01:59:50-05:00",
                "2026-03-08T03:01:10-04:00",
                "Sun 01:59 EST → Sun 03:01 EDT  (0h 02m)",
            ),
            (
                "1969-12-31T18:59:59-05:00",
                "1969-12-31T19:00:01-05:00",
                "Wed 18:59 EST → Wed 19:00 EST  (0h 01m)",
            ),
        ] {
            p.first_moon_free_window = Some(Period {
                start: DateTime::parse_from_rfc3339(start)
                    .unwrap()
                    .with_timezone(&New_York),
                end: DateTime::parse_from_rfc3339(end)
                    .unwrap()
                    .with_timezone(&New_York),
            });
            assert!(render(&p).contains(expected));
        }
    }

    #[test]
    fn interval_follows_local_noon_across_dst() {
        let location = Location::new(40.7128, -74.0060).unwrap();
        for (month, day, hours) in [(3, 7, 23), (10, 31, 25)] {
            let date = NaiveDate::from_ymd_opt(2026, month, day).unwrap();
            let p = build_plan(&location, New_York, None, date).unwrap();
            assert_eq!((p.interval.end - p.interval.start).num_hours(), hours);
            assert_eq!(p.interval.end.date_naive(), date.succ_opt().unwrap());
            if let Some(w) = p.first_moon_free_window {
                assert!(w.start >= p.interval.start && w.end <= p.interval.end && w.start < w.end);
            }
        }
    }

    #[test]
    fn date_line_uses_local_evening_and_following_morning() {
        let date = NaiveDate::from_ymd_opt(2026, 9, 15).unwrap();
        let p = build_plan(
            &Location::new(1.8721, -157.4278).unwrap(),
            chrono_tz::Pacific::Kiritimati,
            None,
            date,
        )
        .unwrap();
        assert_eq!(p.sunset.unwrap().date_naive(), date);
        assert_eq!(p.astronomical_dusk.unwrap().date_naive(), date);
        assert_eq!(p.sunrise.unwrap().date_naive(), date.succ_opt().unwrap());
        assert_eq!(
            p.astronomical_dawn.unwrap().date_naive(),
            date.succ_opt().unwrap()
        );
        assert!(p.evening_golden_hour.is_some());
        assert!(p.evening_blue_hour.is_some());
    }

    #[test]
    fn polar_day_has_no_dark_window_or_sunset() {
        let p = build_plan(
            &Location::new(69.6492, 18.9553).unwrap(),
            Oslo,
            Some("Tromsø"),
            NaiveDate::from_ymd_opt(2026, 6, 21).unwrap(),
        )
        .unwrap();
        assert!(p.first_moon_free_window.is_none());
        assert!(p.sunset.is_none());
        assert!(p.astronomical_dusk.is_none());
        assert!(render(&p).contains("None in this noon-to-noon interval"));
    }

    #[test]
    fn continuous_polar_darkness_is_clipped_and_labeled() {
        let p = build_plan(
            &Location::new(89.0, 0.0).unwrap(),
            chrono_tz::UTC,
            None,
            NaiveDate::from_ymd_opt(2026, 12, 10).unwrap(),
        )
        .unwrap();
        let w = p.first_moon_free_window.as_ref().unwrap();
        assert_eq!(w.start, p.interval.start);
        assert_eq!(w.end, p.interval.end);
        assert!(p.moon_free_window_clipped);
        assert!(render(&p).contains("end is clipped to local noon"));
    }

    #[test]
    fn bright_moon_can_leave_no_moon_free_window() {
        let p = build_plan(
            &Location::new(40.7128, -74.0060).unwrap(),
            New_York,
            None,
            NaiveDate::from_ymd_opt(2026, 9, 26).unwrap(),
        )
        .unwrap();
        assert!(p.astronomical_dusk.is_some());
        assert!(p.first_moon_free_window.is_none());
        assert!(!p.moon_free_window_clipped);
    }

    #[test]
    fn ordinary_night_uses_next_mornings_dawn_and_shared_darkness() {
        let location = Location::new(40.7128, -74.0060).unwrap();
        let date = NaiveDate::from_ymd_opt(2026, 9, 15).unwrap();
        let p = build_plan(&location, New_York, Some("New York"), date).unwrap();
        assert_eq!(p.astronomical_dusk.unwrap().date_naive(), date);
        assert_eq!(
            p.astronomical_dawn.unwrap().date_naive(),
            date.succ_opt().unwrap()
        );
        let w = p.first_moon_free_window.as_ref().unwrap();
        let midpoint = w.start + (w.end - w.start) / 2;
        assert!(sun::solar_position(&location, &midpoint).altitude < -18.0);
        assert!(moon::lunar_position(&location, &midpoint).altitude < 0.0);
        let json: serde_json::Value =
            serde_json::from_str(&generate(&location, New_York, None, date, true).unwrap())
                .unwrap();
        assert_eq!(json["night_of"], "2026-09-15");
        assert!(
            json["interval"]["end"]
                .as_str()
                .unwrap()
                .ends_with("-04:00")
        );
    }
}
