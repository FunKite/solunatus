//! Calendar generation module.
//!
//! Generates astronomical calendars in HTML or JSON format for arbitrary date ranges.
//! Supports historical dates (negative years for BCE) and future dates.
//!
//! # Features
//!
//! - HTML and JSON output formats
//! - Per-day solar events (sunrise, sunset, solar noon, twilight)
//! - Per-day lunar events (moonrise, moonset, phase)
//! - Supports BCE dates (year -999 = 1000 BCE)
//! - Future dates up to year 3000

use crate::astro::{Location, moon, sun};
use anyhow::{Context, Result, anyhow};
use chrono::{Datelike, Duration, NaiveDate, TimeZone, Utc, Weekday};
use chrono_tz::Tz;
use serde::Serialize;
use std::collections::BTreeMap;

const MIN_YEAR: i32 = -999; // Astronomical year numbering => 1000 BCE
const MAX_YEAR: i32 = 3000;

/// Calendar output format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalendarFormat {
    /// HTML format with embedded CSS
    Html,
    /// JSON format for programmatic access
    Json,
    /// iCalendar format (RFC 5545) for import into calendar applications
    Ics,
}

#[derive(Debug)]
struct DailyRecord {
    date: NaiveDate,
    weekday: Weekday,
    sunrise: Option<chrono::DateTime<Tz>>,
    sunset: Option<chrono::DateTime<Tz>>,
    solar_noon: Option<chrono::DateTime<Tz>>,
    civil_dawn: Option<chrono::DateTime<Tz>>,
    civil_dusk: Option<chrono::DateTime<Tz>>,
    moonrise: Option<chrono::DateTime<Tz>>,
    moonset: Option<chrono::DateTime<Tz>>,
    illumination: f64,
    phase_angle: f64,
    phase_name: String,
    phase_emoji: String,
}

#[derive(Debug, Serialize)]
struct CalendarMetadata<'a> {
    latitude: f64,
    longitude: f64,
    timezone: String,
    city: Option<&'a str>,
    range_start: String,
    range_end: String,
    generated_at_utc: String,
}

#[derive(Debug, Serialize)]
struct CalendarDayJson {
    date: String,
    weekday: String,
    solar: SolarBlockJson,
    lunar: LunarBlockJson,
}

#[derive(Debug, Serialize)]
struct SolarBlockJson {
    sunrise: Option<String>,
    sunset: Option<String>,
    solar_noon: Option<String>,
    civil_dawn: Option<String>,
    civil_dusk: Option<String>,
    day_length_minutes: Option<f64>,
}

#[derive(Debug, Serialize)]
struct LunarBlockJson {
    moonrise: Option<String>,
    moonset: Option<String>,
    illumination_percent: f64,
    phase_angle_degrees: f64,
    phase_name: String,
    phase_emoji: String,
}

#[derive(Debug, Serialize)]
struct CalendarJson<'a> {
    metadata: CalendarMetadata<'a>,
    days: Vec<CalendarDayJson>,
}

/// Generates an astronomical calendar for a date range.
///
/// Creates an HTML or JSON calendar showing daily solar and lunar events
/// (sunrise, sunset, moonrise, moonset, lunar phases) for the specified
/// date range and location.
///
/// # Arguments
///
/// * `location` - Geographic location for calculations
/// * `timezone` - Timezone for event times
/// * `city_name` - Optional city name for display
/// * `start` - First date to include (inclusive)
/// * `end` - Last date to include (inclusive)
/// * `format` - Output format (HTML or JSON)
///
/// # Returns
///
/// - HTML: Complete styled HTML document with monthly tables
/// - JSON: Structured JSON with metadata and daily event arrays
///
/// # Errors
///
/// Returns an error if:
/// - Start date is after end date
/// - Date range is outside supported bounds (1000 BCE to 3000 CE)
/// - Astronomical calculations fail for any date
///
/// # Examples
///
/// ```no_run
/// use solunatus::calendar::{generate_calendar, CalendarFormat};
/// use solunatus::astro::Location;
/// use chrono::NaiveDate;
/// use chrono_tz::America::New_York;
///
/// let location = Location::new(42.36, -71.06).unwrap();
/// let start = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
/// let end = NaiveDate::from_ymd_opt(2025, 1, 31).unwrap();
///
/// let html = generate_calendar(
///     &location,
///     &New_York,
///     Some("Boston"),
///     start,
///     end,
///     CalendarFormat::Html,
/// ).unwrap();
/// ```
pub fn generate_calendar(
    location: &Location,
    timezone: &Tz,
    city_name: Option<&str>,
    start: NaiveDate,
    end: NaiveDate,
    format: CalendarFormat,
) -> Result<String> {
    validate_range(start, end)?;

    let records = collect_records(location, timezone, start, end)?;

    match format {
        CalendarFormat::Html => Ok(render_html(
            location, timezone, city_name, start, end, &records,
        )),
        CalendarFormat::Json => render_json(location, timezone, city_name, start, end, &records),
        CalendarFormat::Ics => Ok(render_ics(
            location, timezone, city_name, start, end, &records,
        )),
    }
}

fn validate_range(start: NaiveDate, end: NaiveDate) -> Result<()> {
    if start > end {
        return Err(anyhow!("Calendar start date must be before end date"));
    }

    let start_year = start.year();
    let end_year = end.year();

    if start_year < MIN_YEAR || end_year > MAX_YEAR {
        return Err(anyhow!(
            "Calendar range must fall between astronomical years {} (1000 BCE) and {} (3000 CE)",
            MIN_YEAR,
            MAX_YEAR
        ));
    }
    Ok(())
}

fn collect_records(
    location: &Location,
    timezone: &Tz,
    start: NaiveDate,
    end: NaiveDate,
) -> Result<Vec<DailyRecord>> {
    let dates = enumerate_dates(start, end)?;
    build_records(location, timezone, dates)
}

/// Materialize every date in `[start, end]` so the per-day work can be mapped
/// (optionally in parallel) without re-deriving dates from an index.
fn enumerate_dates(start: NaiveDate, end: NaiveDate) -> Result<Vec<NaiveDate>> {
    let mut dates = Vec::new();
    let mut current = start;

    while current <= end {
        dates.push(current);
        current = current
            .checked_add_signed(Duration::days(1))
            .ok_or_else(|| anyhow!("Date overflow when iterating calendar range"))?;
    }

    Ok(dates)
}

#[cfg(feature = "parallel")]
fn build_records(
    location: &Location,
    timezone: &Tz,
    dates: Vec<NaiveDate>,
) -> Result<Vec<DailyRecord>> {
    use rayon::prelude::*;

    dates
        .into_par_iter()
        .map(|date| {
            build_record(location, timezone, date)
                .with_context(|| format!("Failed to compute ephemerides for {}", date))
        })
        .collect()
}

#[cfg(not(feature = "parallel"))]
fn build_records(
    location: &Location,
    timezone: &Tz,
    dates: Vec<NaiveDate>,
) -> Result<Vec<DailyRecord>> {
    dates
        .into_iter()
        .map(|date| {
            build_record(location, timezone, date)
                .with_context(|| format!("Failed to compute ephemerides for {}", date))
        })
        .collect()
}

fn build_record(location: &Location, timezone: &Tz, date: NaiveDate) -> Result<DailyRecord> {
    let local_midday = resolve_midday(timezone, date)?;

    let sunrise = sun::solar_event_time(location, &local_midday, sun::SolarEvent::Sunrise);
    let sunset = sun::solar_event_time(location, &local_midday, sun::SolarEvent::Sunset);
    let solar_noon = sun::solar_event_time(location, &local_midday, sun::SolarEvent::SolarNoon);
    let civil_dawn = sun::solar_event_time(location, &local_midday, sun::SolarEvent::CivilDawn);
    let civil_dusk = sun::solar_event_time(location, &local_midday, sun::SolarEvent::CivilDusk);

    let moonrise = moon::lunar_event_time(location, &local_midday, moon::LunarEvent::Moonrise);
    let moonset = moon::lunar_event_time(location, &local_midday, moon::LunarEvent::Moonset);

    let lunar_position = moon::lunar_position(location, &local_midday);

    Ok(DailyRecord {
        date,
        weekday: local_midday.weekday(),
        sunrise,
        sunset,
        solar_noon,
        civil_dawn,
        civil_dusk,
        moonrise,
        moonset,
        illumination: lunar_position.illumination,
        phase_angle: lunar_position.phase_angle,
        phase_name: moon::phase_name(lunar_position.phase_angle).to_string(),
        phase_emoji: moon::phase_emoji(lunar_position.phase_angle).to_string(),
    })
}

fn resolve_midday(timezone: &Tz, date: NaiveDate) -> Result<chrono::DateTime<Tz>> {
    match timezone.with_ymd_and_hms(date.year(), date.month(), date.day(), 12, 0, 0) {
        chrono::LocalResult::Single(dt) => Ok(dt),
        chrono::LocalResult::Ambiguous(first, _second) => Ok(first),
        chrono::LocalResult::None => timezone
            .with_ymd_and_hms(date.year(), date.month(), date.day(), 13, 0, 0)
            .single()
            .ok_or_else(|| anyhow!("Unable to resolve local midday for {}", date)),
    }
}

fn render_json(
    location: &Location,
    timezone: &Tz,
    city_name: Option<&str>,
    start: NaiveDate,
    end: NaiveDate,
    records: &[DailyRecord],
) -> Result<String> {
    let metadata = CalendarMetadata {
        latitude: location.latitude.value(),
        longitude: location.longitude.value(),
        timezone: timezone.name().to_string(),
        city: city_name,
        range_start: start.to_string(),
        range_end: end.to_string(),
        generated_at_utc: Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
    };

    let days = records
        .iter()
        .map(|record| CalendarDayJson {
            date: record.date.to_string(),
            weekday: record.weekday.to_string(),
            solar: SolarBlockJson {
                sunrise: record.sunrise.map(format_time),
                sunset: record.sunset.map(format_time),
                solar_noon: record.solar_noon.map(format_time),
                civil_dawn: record.civil_dawn.map(format_time),
                civil_dusk: record.civil_dusk.map(format_time),
                day_length_minutes: day_length_minutes(record),
            },
            lunar: LunarBlockJson {
                moonrise: record.moonrise.map(format_time),
                moonset: record.moonset.map(format_time),
                illumination_percent: (record.illumination * 100.0 * 10.0).round() / 10.0,
                phase_angle_degrees: (record.phase_angle * 10.0).round() / 10.0,
                phase_name: record.phase_name.clone(),
                phase_emoji: record.phase_emoji.clone(),
            },
        })
        .collect();

    Ok(serde_json::to_string_pretty(&CalendarJson {
        metadata,
        days,
    })?)
}

fn render_html(
    location: &Location,
    timezone: &Tz,
    city_name: Option<&str>,
    start: NaiveDate,
    end: NaiveDate,
    records: &[DailyRecord],
) -> String {
    let mut by_month: BTreeMap<(i32, u32), Vec<&DailyRecord>> = BTreeMap::new();
    for record in records {
        by_month
            .entry((record.date.year(), record.date.month()))
            .or_default()
            .push(record);
    }

    let mut html = String::new();
    html.push_str("<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"utf-8\"/>");
    html.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"/>");
    html.push_str("<title>Solunatus Calendar</title>");
    html.push_str("<style>");
    html.push_str("body{font-family:'Helvetica Neue',Arial,sans-serif;background:#0d1117;color:#e6edf3;margin:0;padding:0 0 3rem 0;}");
    html.push_str("header{padding:2.5rem 1.5rem 1.5rem;background:linear-gradient(135deg,#1f6feb,#8b949e);color:#fff;}");
    html.push_str("header h1{margin:0;font-size:2.2rem;font-weight:700;}");
    html.push_str(".meta{margin-top:0.5rem;font-size:1rem;opacity:0.9;}");
    html.push_str(".container{max-width:1200px;margin:0 auto;padding:1.5rem;}");
    html.push_str("h2{margin:2rem 0 0.75rem;font-size:1.6rem;color:#58a6ff;}");
    html.push_str("table{width:100%;border-collapse:collapse;background:#161b22;border-radius:0.75rem;overflow:hidden;box-shadow:0 20px 45px rgba(0,0,0,0.35);}");
    html.push_str(
        "thead{background:rgba(88,166,255,0.12);text-transform:uppercase;letter-spacing:0.05em;}",
    );
    html.push_str("th,td{padding:0.5rem 1rem;text-align:left;white-space:nowrap;}");
    html.push_str("tbody tr:nth-child(even){background:rgba(88,166,255,0.04);}");
    html.push_str("tbody tr:hover{background:rgba(88,166,255,0.14);}");
    html.push_str(".date{font-weight:600;font-size:1rem;}");
    html.push_str(".weekday{font-size:0.9rem;opacity:0.75;display:inline;margin-left:0.5rem;text-transform:uppercase;}");
    html.push_str(".emoji{font-size:1.2rem;margin-right:0.5rem;}");
    html.push_str(".illum{font-size:0.9rem;opacity:0.8;}");
    html.push_str(".badge{display:inline-block;padding:0.25rem 0.65rem;border-radius:999px;font-size:0.8rem;background:rgba(88,166,255,0.2);color:#58a6ff;margin-left:0.4rem;}");
    html.push_str(".location{font-size:1rem;margin-top:0.25rem;}");
    html.push_str(".footer-note{margin-top:3rem;font-size:0.85rem;opacity:0.7;text-align:center;}");
    html.push_str("</style></head><body>");

    html.push_str("<header>");
    html.push_str("<h1>Solunatus Astronomical Calendar</h1>");
    html.push_str("<div class=\"meta\">");
    if let Some(city) = city_name {
        html.push_str(&escape_html(city));
        html.push_str(" • ");
    }
    html.push_str(&format!(
        "{} • {} • {}",
        format_lat(location.latitude.value()),
        format_lon(location.longitude.value()),
        timezone.name()
    ));
    html.push_str("</div>");
    html.push_str("<div class=\"meta\">");
    html.push_str(&format!(
        "Range: {} → {} • Generated {}",
        start,
        end,
        Utc::now().format("%Y-%m-%d %H:%M UTC")
    ));
    html.push_str("</div>");
    html.push_str("</header>");

    html.push_str("<div class=\"container\">");

    for ((year, month), days) in by_month {
        let month_name = month_name(month);
        html.push_str(&format!("<h2>{} {}</h2>", month_name, year));
        html.push_str("<table><thead><tr>");
        html.push_str("<th>Date</th><th>Sunrise</th><th>Sunset</th><th>Daylight</th>");
        html.push_str("<th>Civil Dawn</th><th>Civil Dusk</th>");
        html.push_str("<th>Moonrise</th><th>Moonset</th><th>Lunar Phase</th>");
        html.push_str("</tr></thead><tbody>");

        for record in days {
            html.push_str("<tr>");

            html.push_str("<td class=\"date\">");
            html.push_str(&format!(
                "{} <span class=\"weekday\">{}</span>",
                record.date.format("%b %d"),
                record.weekday
            ));
            html.push_str("</td>");

            html.push_str(&format!(
                "<td>{}</td>",
                record.sunrise.map_or("—".to_string(), format_time)
            ));
            html.push_str(&format!(
                "<td>{}</td>",
                record.sunset.map_or("—".to_string(), format_time)
            ));
            html.push_str(&format!(
                "<td>{}</td>",
                day_length_minutes(record)
                    .map(|mins| format!(
                        "{:02} h {:02} m",
                        (mins / 60.0).floor() as i64,
                        (mins % 60.0).round() as i64
                    ))
                    .unwrap_or_else(|| "—".to_string())
            ));
            html.push_str(&format!(
                "<td>{}</td>",
                record.civil_dawn.map_or("—".to_string(), format_time)
            ));
            html.push_str(&format!(
                "<td>{}</td>",
                record.civil_dusk.map_or("—".to_string(), format_time)
            ));
            html.push_str(&format!(
                "<td>{}</td>",
                record.moonrise.map_or("—".to_string(), format_time)
            ));
            html.push_str(&format!(
                "<td>{}</td>",
                record.moonset.map_or("—".to_string(), format_time)
            ));
            let illum = (record.illumination * 1000.0).round() / 10.0;
            html.push_str("<td>");
            html.push_str(&format!(
                "<span class=\"emoji\">{}</span>{}<span class=\"badge\">{:.1}%</span>",
                record.phase_emoji,
                escape_html(&record.phase_name),
                illum
            ));
            html.push_str("</td>");

            html.push_str("</tr>");
        }

        html.push_str("</tbody></table>");
    }

    html.push_str("<div class=\"footer-note\">Times shown in local apparent clock time. Daylight savings and high-latitude edge cases are handled gracefully; missing events are marked with an em dash.</div>");
    html.push_str("</div></body></html>");

    html
}

/// Escape text per RFC 5545 (backslash, semicolon, comma, newline).
fn escape_ics(input: &str) -> String {
    input
        .replace('\\', "\\\\")
        .replace(';', "\\;")
        .replace(',', "\\,")
        .replace('\n', "\\n")
}

/// Fold a content line at 75 octets per RFC 5545 (continuation lines start
/// with a single space). Folds on character boundaries.
fn fold_ics_line(line: &str, out: &mut String) {
    const LIMIT: usize = 75;
    let mut budget = LIMIT;
    let mut len = 0usize;

    for ch in line.chars() {
        let ch_len = ch.len_utf8();
        if len + ch_len > budget {
            out.push_str("\r\n ");
            budget = LIMIT - 1; // continuation lines lose one octet to the leading space
            len = 0;
        }
        out.push(ch);
        len += ch_len;
    }
    out.push_str("\r\n");
}

fn push_ics_event(
    out: &mut String,
    dtstamp: &str,
    uid_suffix: &str,
    event_utc: chrono::DateTime<Utc>,
    summary: &str,
) {
    fold_ics_line("BEGIN:VEVENT", out);
    fold_ics_line(
        &format!(
            "UID:{}-{}@solunatus",
            event_utc.format("%Y%m%dT%H%M%SZ"),
            uid_suffix
        ),
        out,
    );
    fold_ics_line(&format!("DTSTAMP:{}", dtstamp), out);
    fold_ics_line(
        &format!("DTSTART:{}", event_utc.format("%Y%m%dT%H%M%SZ")),
        out,
    );
    fold_ics_line(&format!("SUMMARY:{}", escape_ics(summary)), out);
    fold_ics_line("END:VEVENT", out);
}

fn render_ics(
    location: &Location,
    timezone: &Tz,
    city_name: Option<&str>,
    start: NaiveDate,
    end: NaiveDate,
    records: &[DailyRecord],
) -> String {
    let dtstamp = Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
    let place = city_name.map_or_else(
        || {
            format!(
                "{}, {}",
                format_lat(location.latitude.value()),
                format_lon(location.longitude.value())
            )
        },
        ToString::to_string,
    );

    let mut out = String::new();
    fold_ics_line("BEGIN:VCALENDAR", &mut out);
    fold_ics_line("VERSION:2.0", &mut out);
    fold_ics_line(
        &format!(
            "PRODID:-//Solunatus//solunatus {}//EN",
            env!("CARGO_PKG_VERSION")
        ),
        &mut out,
    );
    fold_ics_line("CALSCALE:GREGORIAN", &mut out);
    fold_ics_line("METHOD:PUBLISH", &mut out);
    fold_ics_line(
        &format!("X-WR-CALNAME:Sun & Moon — {}", escape_ics(&place)),
        &mut out,
    );

    type DailyEventAccessor = fn(&DailyRecord) -> Option<chrono::DateTime<Tz>>;
    let daily_events: [(&str, DailyEventAccessor); 4] = [
        ("Sunrise 🌅", |r| r.sunrise),
        ("Sunset 🌇", |r| r.sunset),
        ("Moonrise 🌕", |r| r.moonrise),
        ("Moonset 🌑", |r| r.moonset),
    ];

    let uid_coords = format!(
        "{:.2}{:.2}",
        location.latitude.value(),
        location.longitude.value()
    );

    for record in records {
        for (summary, accessor) in daily_events {
            if let Some(event_time) = accessor(record) {
                let slug = summary
                    .split_whitespace()
                    .next()
                    .unwrap_or("event")
                    .to_lowercase();
                push_ics_event(
                    &mut out,
                    &dtstamp,
                    &format!("{}-{}", slug, uid_coords),
                    event_time.with_timezone(&Utc),
                    summary,
                );
            }
        }
    }

    // Quarter lunar phases that fall (in local time) within the range. Phases
    // are tabulated by UTC month, and one near a month boundary can have a
    // local date in the neighboring month, so scan one UTC month past each
    // end of the range and let the local-date filter decide.
    let first_month = start.with_day(1).unwrap_or(start);
    let mut cursor = if first_month.month() == 1 {
        NaiveDate::from_ymd_opt(first_month.year() - 1, 12, 1)
    } else {
        NaiveDate::from_ymd_opt(first_month.year(), first_month.month() - 1, 1)
    }
    .unwrap_or(first_month);
    let scan_end = if end.month() == 12 {
        NaiveDate::from_ymd_opt(end.year() + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(end.year(), end.month() + 1, 1)
    }
    .unwrap_or(end);
    while cursor <= scan_end {
        for phase in moon::lunar_phases(cursor.year(), cursor.month()) {
            let local_date = phase.datetime.with_timezone(timezone).date_naive();
            if local_date < start || local_date > end {
                continue;
            }
            let summary = match phase.phase_type {
                moon::LunarPhaseType::NewMoon => "New Moon 🌑".to_string(),
                moon::LunarPhaseType::FirstQuarter => "First Quarter 🌓".to_string(),
                moon::LunarPhaseType::FullMoon => {
                    if moon::is_supermoon(&phase) {
                        "Full Moon (Supermoon) 🌕".to_string()
                    } else {
                        "Full Moon 🌕".to_string()
                    }
                }
                moon::LunarPhaseType::LastQuarter => "Last Quarter 🌗".to_string(),
            };
            push_ics_event(&mut out, &dtstamp, "phase", phase.datetime, &summary);
        }

        cursor = if cursor.month() == 12 {
            match NaiveDate::from_ymd_opt(cursor.year() + 1, 1, 1) {
                Some(next) => next,
                None => break,
            }
        } else {
            match NaiveDate::from_ymd_opt(cursor.year(), cursor.month() + 1, 1) {
                Some(next) => next,
                None => break,
            }
        };
    }

    fold_ics_line("END:VCALENDAR", &mut out);
    out
}

fn format_time(dt: chrono::DateTime<Tz>) -> String {
    dt.format("%H:%M").to_string()
}

fn day_length_minutes(record: &DailyRecord) -> Option<f64> {
    match (record.sunrise, record.sunset) {
        (Some(rise), Some(set)) => {
            let diff = set.signed_duration_since(rise);
            if diff.num_seconds() >= 0 {
                Some(diff.num_seconds() as f64 / 60.0)
            } else {
                None
            }
        }
        _ => None,
    }
}

fn month_name(month: u32) -> &'static str {
    match month {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => "Unknown",
    }
}

fn escape_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn format_lat(lat: f64) -> String {
    if lat >= 0.0 {
        format!("{:.4}° N", lat)
    } else {
        format!("{:.4}° S", -lat)
    }
}

fn format_lon(lon: f64) -> String {
    if lon >= 0.0 {
        format!("{:.4}° E", lon)
    } else {
        format!("{:.4}° W", -lon)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::astro::moon;
    use chrono_tz::America::New_York;

    #[test]
    fn ics_calendar_is_well_formed() {
        let location = Location::new(40.7128, -74.0060).unwrap();
        let tz = New_York;
        let start = NaiveDate::from_ymd_opt(2025, 10, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2025, 10, 31).unwrap();

        let ics = generate_calendar(
            &location,
            &tz,
            Some("New York"),
            start,
            end,
            CalendarFormat::Ics,
        )
        .unwrap();

        assert!(ics.starts_with("BEGIN:VCALENDAR\r\n"));
        assert!(ics.ends_with("END:VCALENDAR\r\n"));
        assert!(ics.contains("VERSION:2.0\r\n"));

        let begins = ics.matches("BEGIN:VEVENT").count();
        let ends = ics.matches("END:VEVENT").count();
        assert_eq!(begins, ends, "unbalanced VEVENT blocks");
        // 31 days × up to 4 daily events, minus the occasional missing
        // moonrise/moonset, plus 4-5 quarter phases.
        assert!(begins >= 120, "only {begins} events generated");

        // Every event needs UID, DTSTAMP, DTSTART, SUMMARY.
        for field in ["UID:", "DTSTAMP:", "DTSTART:", "SUMMARY:"] {
            assert_eq!(ics.matches(field).count(), begins, "missing {field} fields");
        }

        // Full moon of Oct 2025 (Oct 7, 03:47 UTC) must be present.
        assert!(
            ics.contains("DTSTART:20251007T03"),
            "full moon event missing"
        );

        // RFC 5545: all lines CRLF-terminated and at most 75 octets.
        for line in ics.split("\r\n") {
            assert!(!line.contains('\n'), "bare LF found");
            assert!(line.len() <= 75, "line exceeds 75 octets: {line}");
        }
    }

    /// A phase early in a UTC month can fall on the last local day of the
    /// previous month: the 2026-12-01 06:14 UTC last quarter is 2026-11-30 in
    /// Los Angeles, so a November export there must still include it.
    #[test]
    fn ics_includes_phases_across_utc_month_boundary() {
        let location = Location::new(34.0522, -118.2437).unwrap();
        let tz = chrono_tz::America::Los_Angeles;
        let start = NaiveDate::from_ymd_opt(2026, 11, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2026, 11, 30).unwrap();

        let ics = generate_calendar(
            &location,
            &tz,
            Some("Los Angeles"),
            start,
            end,
            CalendarFormat::Ics,
        )
        .unwrap();

        assert!(
            ics.contains("DTSTART:20261201T06"),
            "last quarter on the local Nov 30 (Dec 1 UTC) missing from November export"
        );
    }

    /// The calendar must report the same moonrise/moonset as the canonical
    /// `lunar_event_time` algorithm. This guards against reintroducing a separate
    /// (and historically divergent) "optimized" calendar code path.
    #[test]
    fn calendar_lunar_events_match_canonical() {
        let location = Location::new(40.7128, -74.0060).unwrap();
        let tz = New_York;
        let date = NaiveDate::from_ymd_opt(2025, 10, 15).unwrap();

        let json = generate_calendar(
            &location,
            &tz,
            Some("New York"),
            date,
            date,
            CalendarFormat::Json,
        )
        .unwrap();

        let midday = resolve_midday(&tz, date).unwrap();

        for (event, key) in [
            (moon::LunarEvent::Moonrise, "moonrise"),
            (moon::LunarEvent::Moonset, "moonset"),
        ] {
            let expected = moon::lunar_event_time(&location, &midday, event);
            let needle = match expected {
                Some(dt) => format!("\"{}\": \"{}\"", key, dt.format("%H:%M")),
                None => format!("\"{}\": null", key),
            };
            assert!(
                json.contains(&needle),
                "calendar {key} should match canonical ({needle}); json:\n{json}"
            );
        }
    }
}
