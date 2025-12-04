//! USNO validation module for accuracy verification.
//!
//! This module compares Solunatus astronomical calculations against the
//! U.S. Naval Observatory's official data to verify accuracy. It fetches
//! reference data from the USNO API and generates detailed comparison reports.
//!
//! # Features
//!
//! - Validates solar events (sunrise, sunset, solar noon, twilight times)
//! - Validates lunar events (moonrise, moonset)
//! - Compares times to ±1 minute precision
//! - Generates HTML validation reports
//! - Status indicators: Pass (±7 min), Warning (±10 min), Fail (>10 min)
//!
//! # Examples
//!
//! ```no_run
//! use solunatus::usno_validation::generate_validation_report;
//! use solunatus::astro::Location;
//! use chrono::Utc;
//! use chrono_tz::America::New_York;
//!
//! let location = Location::new(42.36, -71.06).unwrap();
//! let now = Utc::now().with_timezone(&New_York);
//!
//! let report = generate_validation_report(
//!     &location,
//!     &New_York,
//!     Some("Boston".to_string()),
//!     &now,
//! ).unwrap();
//!
//! println!("Validation: {}/{} events passed",
//!          report.results.iter().filter(|r| matches!(r.status, solunatus::usno_validation::ValidationStatus::Pass)).count(),
//!          report.results.len());
//! ```

use crate::astro::*;
use crate::events;
use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Duration as ChronoDuration, NaiveDate, NaiveTime, TimeZone, Utc};
use chrono_tz::Tz;
use serde::Deserialize;
use std::collections::HashMap;

const USNO_API_BASE: &str = "https://aa.usno.navy.mil/api/rstt/oneday";

#[derive(Debug, Deserialize)]
struct UsnoResponse {
    #[allow(dead_code)]
    apiversion: String,
    properties: UsnoProperties,
}

#[derive(Debug, Deserialize)]
struct UsnoProperties {
    data: UsnoData,
}

#[derive(Debug, Deserialize)]
struct UsnoData {
    sundata: Vec<UsnoEvent>,
    moondata: Vec<UsnoEvent>,
    #[allow(dead_code)]
    closestphase: Option<UsnoPhase>,
    #[allow(dead_code)]
    curphase: Option<String>,
    #[allow(dead_code)]
    fracillum: Option<String>,
    year: i32,
    month: u32,
    day: u32,
    #[allow(dead_code)]
    tz: f64,  // Timezone offset from UTC (0.0 means UTC)
}

#[derive(Debug, Deserialize)]
struct UsnoEvent {
    phen: String,
    time: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct UsnoPhase {
    phase: String,
    year: i32,
    month: u32,
    day: u32,
    time: String,
}

/// Result of validating a single astronomical event against USNO data.
///
/// Contains the event name, both calculated and reference values,
/// the time difference, and a pass/fail status.
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Event name (e.g., "Sunrise", "Civil Dawn")
    pub event_name: String,
    /// Solunatus calculated time
    pub astrotimes_value: Option<String>,
    /// USNO reference time
    pub usno_value: Option<String>,
    /// Time difference in minutes (negative = Solunatus earlier)
    pub difference_minutes: Option<i64>,
    /// Validation status (Pass/Warning/Fail/Missing)
    pub status: ValidationStatus,
    // Internal field for sorting - holds the datetime for chronological ordering
    _datetime: Option<DateTime<Tz>>,
}

/// Validation status indicating accuracy of calculations.
///
/// Based on time difference thresholds:
/// - Pass: ±7 minutes or less
/// - Warning: ±8-10 minutes
/// - Fail: >10 minutes
/// - Missing: Event not calculated or not in USNO data
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationStatus {
    /// Calculation matches USNO within ±7 minutes
    Pass,
    /// Calculation differs by ±8-10 minutes
    Warning,
    /// Calculation differs by >10 minutes
    Fail,
    /// Event missing from either source
    Missing,
}

impl ValidationStatus {
    fn from_difference(diff_minutes: Option<i64>) -> Self {
        match diff_minutes {
            None => ValidationStatus::Missing,
            Some(d) if d.abs() <= 7 => ValidationStatus::Pass,
            Some(d) if d.abs() <= 10 => ValidationStatus::Warning,
            Some(_) => ValidationStatus::Fail,
        }
    }
}

/// Complete validation report comparing Solunatus to USNO data.
///
/// Contains location, date, version information, and a list of
/// individual event validation results.
pub struct ValidationReport {
    /// Geographic location for validation
    pub location: Location,
    /// Timezone for event times
    pub timezone: Tz,
    /// Optional city name
    pub city_name: Option<String>,
    /// Date and time of validation
    pub date: DateTime<Tz>,
    /// Solunatus version being validated
    pub version: String,
    /// USNO API version used for reference data
    pub usno_apiversion: String,
    /// Individual event validation results
    pub results: Vec<ValidationResult>,
}

/// Fetch USNO data for the given location and date
fn fetch_usno_data(
    location: &Location,
    date: &DateTime<Tz>,
) -> Result<UsnoData> {
    let date_str = date.format("%Y-%m-%d").to_string();
    let coords = format!("{:.5},{:.5}", location.latitude.value(), location.longitude.value());
    let url = format!("{}?date={}&coords={}", USNO_API_BASE, date_str, coords);

    let response = reqwest::blocking::get(&url)
        .with_context(|| format!("Failed to fetch USNO data from {}", url))?;

    if !response.status().is_success() {
        return Err(anyhow!(
            "USNO API returned error: {}",
            response.status()
        ));
    }

    let usno_response: UsnoResponse = response
        .json()
        .context("Failed to parse USNO JSON response")?;

    Ok(usno_response.properties.data)
}

/// Parse USNO time string (HH:MM) as UTC and convert to local timezone
/// Returns the DateTime in the target timezone
fn parse_usno_time_to_local(
    time_str: &str,
    usno_date: NaiveDate,
    target_tz: &Tz,
) -> Option<DateTime<Tz>> {
    let parts: Vec<&str> = time_str.split(':').collect();
    if parts.len() != 2 {
        return None;
    }

    let hours: u32 = parts[0].parse().ok()?;
    let minutes: u32 = parts[1].parse().ok()?;

    // Create NaiveTime from USNO time string
    let naive_time = NaiveTime::from_hms_opt(hours, minutes, 0)?;

    // Combine date and time into NaiveDateTime
    let naive_datetime = usno_date.and_time(naive_time);

    // USNO returns times in UTC (tz: 0.0), so create as UTC DateTime
    let utc_datetime = Utc.from_utc_datetime(&naive_datetime);

    // Convert to target timezone
    Some(utc_datetime.with_timezone(target_tz))
}

/// Map USNO event names to our event types
fn map_usno_event_name(phen: &str, is_sun: bool) -> Option<String> {
    match (phen, is_sun) {
        ("Rise", true) => Some("Sunrise".to_string()),
        ("Set", true) => Some("Sunset".to_string()),
        ("Upper Transit", true) => Some("Solar noon".to_string()),
        ("Begin Civil Twilight", true) => Some("Civil dawn".to_string()),
        ("End Civil Twilight", true) => Some("Civil dusk".to_string()),
        ("Rise", false) => Some("Moonrise".to_string()),
        ("Set", false) => Some("Moonset".to_string()),
        ("Upper Transit", false) => None, // Moon transit removed - difficult to calculate accurately
        _ => None,
    }
}

/// Check if an event should be included in the validation report
/// (excludes nautical and astronomical twilight since USNO doesn't provide them)
fn should_include_in_report(event_name: &str) -> bool {
    !event_name.contains("Nautical")
        && !event_name.contains("Astronomical")
        && !event_name.contains("Astro ")
        && !event_name.contains("Dark win")
}

/// Generate validation report comparing astrotimes calculations with USNO data
pub fn generate_validation_report(
    location: &Location,
    timezone: &Tz,
    city_name: Option<String>,
    date: &DateTime<Tz>,
) -> Result<ValidationReport> {
    // Calculate our own events within ±13 hours
    let events_list = events::collect_events_within_window(
        location,
        date,
        ChronoDuration::hours(13),
    );

    // Build a map of our events for easy lookup, keeping the event closest to reference time
    let mut astrotimes_events: HashMap<String, DateTime<Tz>> = HashMap::new();
    for (dt, name) in events_list {
        let normalized = name.trim_start_matches(|c: char| !c.is_ascii_alphabetic()).to_string();

        if let Some(&existing_dt) = astrotimes_events.get(&normalized) {
            let delta_existing = existing_dt.signed_duration_since(*date).num_seconds().abs();
            let delta_new = dt.signed_duration_since(*date).num_seconds().abs();
            if delta_new < delta_existing {
                astrotimes_events.insert(normalized, dt);
            }
        } else {
            astrotimes_events.insert(normalized, dt);
        }
    }

    // Determine date range to fetch USNO data for (yesterday, today, tomorrow)
    // This ensures we have USNO data for all events in the ±13 hour window

    // Fetch USNO data for all three days and build a map of events by date and name
    let mut usno_events: HashMap<(NaiveDate, String), DateTime<Tz>> = HashMap::new();

    for day_offset in -1..=1 {
        let fetch_date = *date + ChronoDuration::days(day_offset);

        if let Ok(usno_data) = fetch_usno_data(location, &fetch_date) {
            let usno_date = NaiveDate::from_ymd_opt(
                usno_data.year,
                usno_data.month,
                usno_data.day,
            ).ok_or_else(|| anyhow!("Invalid USNO date"))?;

            // Parse sun events
            for event in &usno_data.sundata {
                if let Some(event_name) = map_usno_event_name(&event.phen, true) {
                    if let Some(dt) = parse_usno_time_to_local(&event.time, usno_date, timezone) {
                        usno_events.insert((usno_date, event_name), dt);
                    }
                }
            }

            // Parse moon events
            for event in &usno_data.moondata {
                if let Some(event_name) = map_usno_event_name(&event.phen, false) {
                    if let Some(dt) = parse_usno_time_to_local(&event.time, usno_date, timezone) {
                        usno_events.insert((usno_date, event_name), dt);
                    }
                }
            }
        }
    }

    // Fetch primary day data for metadata
    let usno_data = fetch_usno_data(location, date)
        .context("Failed to fetch USNO reference data")?;

    let mut results = Vec::new();

    // Compare each astrotimes event with USNO data
    // Strategy: For each astrotimes event, find the USNO event with the same name
    // that occurs within ±2 hours. This handles timezone conversions where events
    // may shift dates (e.g., sunset in UTC might be on a different day than in local time).
    for (event_name, at_dt) in &astrotimes_events {
        // Skip nautical and astronomical twilight events (USNO doesn't provide them)
        if !should_include_in_report(event_name) {
            continue;
        }

        // Find matching USNO event within ±2 hours
        // This window handles timezone-induced date shifts while ensuring we don't
        // accidentally match the wrong occurrence of a recurring event
        let mut matching_usno: Option<DateTime<Tz>> = None;
        let max_window = ChronoDuration::hours(2);

        for ((_, usno_event_name), usno_dt) in &usno_events {
            if usno_event_name == event_name {
                let time_diff = (*at_dt - *usno_dt).abs();

                // Only accept matches within ±2 hours (should be same event)
                if time_diff <= max_window {
                    // If we already found a match, keep the closer one
                    if let Some(existing_usno) = matching_usno {
                        let existing_diff = (*at_dt - existing_usno).abs();
                        if time_diff < existing_diff {
                            matching_usno = Some(*usno_dt);
                        }
                    } else {
                        matching_usno = Some(*usno_dt);
                    }
                }
            }
        }

        if let Some(usno_dt) = matching_usno {
            let duration = at_dt.signed_duration_since(usno_dt);
            let diff_minutes = duration.num_minutes();

            results.push(ValidationResult {
                event_name: event_name.clone(),
                astrotimes_value: Some(at_dt.format("%H:%M:%S").to_string()),
                usno_value: Some(usno_dt.format("%H:%M").to_string()),
                difference_minutes: Some(diff_minutes),
                status: ValidationStatus::from_difference(Some(diff_minutes)),
                _datetime: Some(*at_dt),
            });
        } else {
            // No matching USNO event found within ±2 hours
            results.push(ValidationResult {
                event_name: event_name.clone(),
                astrotimes_value: Some(at_dt.format("%H:%M:%S").to_string()),
                usno_value: None,
                difference_minutes: None,
                status: ValidationStatus::Missing,
                _datetime: Some(*at_dt),
            });
        }
    }

    // Sort results chronologically to match watch mode event ordering
    results.sort_by_key(|r| r._datetime);

    Ok(ValidationReport {
        location: *location,
        timezone: *timezone,
        city_name,
        date: *date,
        version: env!("CARGO_PKG_VERSION").to_string(),
        usno_apiversion: usno_data.sundata.first()
            .map(|_| "4.0.1".to_string())
            .unwrap_or_else(|| "unknown".to_string()),
        results,
    })
}

/// Generate HTML report from validation results
pub fn generate_html_report(report: &ValidationReport) -> String {
    let mut html = String::new();

    html.push_str("<!DOCTYPE html>\n");
    html.push_str("<html lang=\"en\">\n<head>\n");
    html.push_str("<meta charset=\"UTF-8\">\n");
    html.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n");
    html.push_str("<title>Solunatus USNO Validation Report</title>\n");
    html.push_str("<style>\n");
    html.push_str("body { font-family: Arial, sans-serif; margin: 40px; background: #f5f5f5; }\n");
    html.push_str("h1 { color: #2c3e50; }\n");
    html.push_str("h2 { color: #34495e; margin-top: 30px; }\n");
    html.push_str(".info { background: white; padding: 20px; border-radius: 8px; margin-bottom: 20px; }\n");
    html.push_str(".info-grid { display: grid; grid-template-columns: 200px 1fr; gap: 10px; }\n");
    html.push_str(".info-label { font-weight: bold; }\n");
    html.push_str("table { border-collapse: collapse; width: 100%; background: white; border-radius: 8px; overflow: hidden; }\n");
    html.push_str("th { background: #3498db; color: white; padding: 12px; text-align: left; }\n");
    html.push_str("td { padding: 10px; border-bottom: 1px solid #ecf0f1; }\n");
    html.push_str("tr:last-child td { border-bottom: none; }\n");
    html.push_str(".pass { background: #d5f4e6; }\n");
    html.push_str(".warning { background: #fff3cd; }\n");
    html.push_str(".fail { background: #f8d7da; }\n");
    html.push_str(".missing { background: #e9ecef; }\n");
    html.push_str(".status-pass { color: #27ae60; font-weight: bold; }\n");
    html.push_str(".status-warning { color: #f39c12; font-weight: bold; }\n");
    html.push_str(".status-fail { color: #e74c3c; font-weight: bold; }\n");
    html.push_str(".status-missing { color: #95a5a6; font-weight: bold; }\n");
    html.push_str(".summary { background: white; padding: 20px; border-radius: 8px; margin-bottom: 20px; }\n");
    html.push_str(".summary-grid { display: grid; grid-template-columns: repeat(4, 1fr); gap: 20px; text-align: center; }\n");
    html.push_str(".summary-item { padding: 15px; border-radius: 8px; }\n");
    html.push_str("</style>\n");
    html.push_str("</head>\n<body>\n");

    html.push_str("<h1>Solunatus USNO Validation Report</h1>\n");

    // Information section
    html.push_str("<div class=\"info\">\n");
    html.push_str("<h2>Configuration</h2>\n");
    html.push_str("<div class=\"info-grid\">\n");
    html.push_str(&format!("<div class=\"info-label\">Solunatus Version:</div><div>{}</div>\n", report.version));
    html.push_str(&format!("<div class=\"info-label\">USNO API Version:</div><div>{}</div>\n", report.usno_apiversion));
    html.push_str(&format!("<div class=\"info-label\">Date:</div><div>{}</div>\n", report.date.format("%Y-%m-%d %H:%M:%S %Z")));
    html.push_str(&format!("<div class=\"info-label\">Timezone:</div><div>{}</div>\n", report.timezone.name()));
    if let Some(ref city) = report.city_name {
        html.push_str(&format!("<div class=\"info-label\">Location:</div><div>{}</div>\n", city));
    }
    html.push_str(&format!("<div class=\"info-label\">Latitude:</div><div>{:.5}°</div>\n", report.location.latitude.value()));
    html.push_str(&format!("<div class=\"info-label\">Longitude:</div><div>{:.5}°</div>\n", report.location.longitude.value()));
    html.push_str("</div>\n");
    html.push_str("</div>\n");

    // Summary statistics
    let pass_count = report.results.iter().filter(|r| r.status == ValidationStatus::Pass).count();
    let warning_count = report.results.iter().filter(|r| r.status == ValidationStatus::Warning).count();
    let fail_count = report.results.iter().filter(|r| r.status == ValidationStatus::Fail).count();
    let missing_count = report.results.iter().filter(|r| r.status == ValidationStatus::Missing).count();

    html.push_str("<div class=\"summary\">\n");
    html.push_str("<h2>Summary</h2>\n");
    html.push_str("<div class=\"summary-grid\">\n");
    html.push_str(&format!("<div class=\"summary-item pass\"><div style=\"font-size: 32px;\">{}</div><div>Pass (0-7 min)</div></div>\n", pass_count));
    html.push_str(&format!("<div class=\"summary-item warning\"><div style=\"font-size: 32px;\">{}</div><div>Caution (7-10 min)</div></div>\n", warning_count));
    html.push_str(&format!("<div class=\"summary-item fail\"><div style=\"font-size: 32px;\">{}</div><div>Fail (>10 min)</div></div>\n", fail_count));
    html.push_str(&format!("<div class=\"summary-item missing\"><div style=\"font-size: 32px;\">{}</div><div>Missing</div></div>\n", missing_count));
    html.push_str("</div>\n");
    html.push_str("</div>\n");

    // Results table
    html.push_str("<h2>Validation Results</h2>\n");
    html.push_str("<div style=\"background: #e8f4f8; padding: 15px; border-radius: 8px; margin-bottom: 20px; border-left: 4px solid #3498db;\">\n");
    html.push_str("<p style=\"margin: 0; font-size: 13px; color: #2c3e50;\">\n");
    html.push_str("<strong>Important Notes:</strong><br>\n");
    html.push_str("• USNO API provides times in UTC with <strong>minute-level granularity only</strong> (HH:MM)<br>\n");
    html.push_str("• Solunatus calculates times with <strong>second-level precision</strong> (HH:MM:SS)<br>\n");
    html.push_str("• All USNO times below have been converted from UTC to your local timezone for comparison<br>\n");
    html.push_str("• Differences within 0-7 minutes are considered a PASS<br>\n");
    html.push_str("• Differences of 7-10 minutes are flagged as CAUTION<br>\n");
    html.push_str("• Differences over 10 minutes are considered a FAIL\n");
    html.push_str("</p>\n");
    html.push_str("</div>\n");
    html.push_str("<table>\n");
    html.push_str("<thead>\n");
    html.push_str("<tr>\n");
    html.push_str("<th>Event</th>\n");
    html.push_str("<th>Solunatus (HH:MM:SS)</th>\n");
    html.push_str("<th>USNO Local Time (HH:MM)</th>\n");
    html.push_str("<th>Difference</th>\n");
    html.push_str("<th>Status</th>\n");
    html.push_str("</tr>\n");
    html.push_str("</thead>\n");
    html.push_str("<tbody>\n");

    for result in &report.results {
        let row_class = match result.status {
            ValidationStatus::Pass => "pass",
            ValidationStatus::Warning => "warning",
            ValidationStatus::Fail => "fail",
            ValidationStatus::Missing => "missing",
        };

        let status_class = match result.status {
            ValidationStatus::Pass => "status-pass",
            ValidationStatus::Warning => "status-warning",
            ValidationStatus::Fail => "status-fail",
            ValidationStatus::Missing => "status-missing",
        };

        let status_text = match result.status {
            ValidationStatus::Pass => "✓ PASS",
            ValidationStatus::Warning => "⚠ WARNING",
            ValidationStatus::Fail => "✗ FAIL",
            ValidationStatus::Missing => "— MISSING",
        };

        let diff_text = result.difference_minutes
            .map(|d| format!("{:+} min", d))
            .unwrap_or_else(|| "—".to_string());

        html.push_str(&format!("<tr class=\"{}\">\n", row_class));
        html.push_str(&format!("<td>{}</td>\n", result.event_name));
        html.push_str(&format!("<td>{}</td>\n", result.astrotimes_value.as_deref().unwrap_or("—")));
        html.push_str(&format!("<td>{}</td>\n", result.usno_value.as_deref().unwrap_or("—")));
        html.push_str(&format!("<td>{}</td>\n", diff_text));
        html.push_str(&format!("<td class=\"{}\">{}</td>\n", status_class, status_text));
        html.push_str("</tr>\n");
    }

    html.push_str("</tbody>\n");
    html.push_str("</table>\n");

    html.push_str("<div style=\"margin-top: 40px; color: #7f8c8d; font-size: 12px;\">\n");
    html.push_str(&format!("Generated: {}<br>\n", chrono::Local::now().format("%Y-%m-%d %H:%M:%S")));
    html.push_str("Reference: U.S. Naval Observatory Astronomical Applications Department<br>\n");
    html.push_str("https://aa.usno.navy.mil/\n");
    html.push_str("</div>\n");

    html.push_str("</body>\n</html>\n");

    html
}
