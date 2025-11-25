//! JSON output formatting.
//!
//! Provides structured JSON output for astronomical data including
//! positions, events, phases, and optional AI insights.

#[cfg(feature = "ai-insights")]
use crate::ai;
use crate::astro::*;
use crate::events;
use crate::time_sync;
use anyhow::Result;
use chrono::{DateTime, Datelike, Utc};
use chrono_tz::Tz;
use serde::Serialize;

#[derive(Serialize)]
pub struct JsonOutput {
    pub location: LocationData,
    pub datetime: DateTimeData,
    pub sun: SunData,
    pub moon: MoonData,
    pub lunar_phases: Vec<LunarPhaseData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ai_insights: Option<AiInsightsData>,
}

#[derive(Serialize)]
pub struct LocationData {
    pub latitude: f64,
    pub longitude: f64,
    pub timezone: String,
    pub city: Option<String>,
}

#[derive(Serialize)]
pub struct DateTimeData {
    pub local: String,
    pub utc: String,
    pub timezone_offset: String,
    pub time_sync: TimeSyncData,
}

#[derive(Serialize)]
pub struct SunData {
    pub position: PositionData,
    pub events: SunEvents,
}

#[derive(Serialize)]
pub struct MoonData {
    pub position: MoonPositionData,
    pub events: MoonEvents,
    pub phase: PhaseData,
}

#[derive(Serialize)]
pub struct PositionData {
    pub altitude: f64,
    pub azimuth: f64,
    pub azimuth_compass: String,
}

#[derive(Serialize)]
pub struct MoonPositionData {
    pub altitude: f64,
    pub azimuth: f64,
    pub azimuth_compass: String,
    pub distance_km: f64,
    pub angular_diameter_arcmin: f64,
}

#[derive(Serialize)]
pub struct SunEvents {
    pub sunrise: Option<String>,
    pub sunset: Option<String>,
    pub solar_noon: Option<String>,
    pub civil_dawn: Option<String>,
    pub civil_dusk: Option<String>,
    pub nautical_dawn: Option<String>,
    pub nautical_dusk: Option<String>,
    pub astronomical_dawn: Option<String>,
    pub astronomical_dusk: Option<String>,
}

#[derive(Serialize)]
pub struct MoonEvents {
    pub moonrise: Option<String>,
    pub moonset: Option<String>,
}

#[derive(Serialize)]
pub struct PhaseData {
    pub name: String,
    pub emoji: String,
    pub angle_degrees: f64,
    pub illumination_percent: f64,
}

#[derive(Serialize)]
pub struct LunarPhaseData {
    pub phase_type: String,
    pub datetime: String,
}

#[derive(Serialize)]
pub struct TimeSyncData {
    pub source: String,
    pub delta_seconds: Option<f64>,
    pub offset_display: Option<String>,
    pub status: String,
    pub error: Option<String>,
}

#[derive(Serialize)]
pub struct AiInsightsData {
    pub model: String,
    pub updated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_elapsed: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[cfg(feature = "ai-insights")]
pub fn generate_json_output(
    location: &Location,
    timezone: &Tz,
    city_name: Option<String>,
    dt: &DateTime<Tz>,
    timezone_name: &str,
    time_sync_info: &time_sync::TimeSyncInfo,
    ai_config: &ai::AiConfig,
) -> Result<String> {
    generate_json_output_impl(location, timezone, city_name, dt, timezone_name, time_sync_info, Some(ai_config))
}

#[cfg(not(feature = "ai-insights"))]
pub fn generate_json_output(
    location: &Location,
    timezone: &Tz,
    city_name: Option<String>,
    dt: &DateTime<Tz>,
    timezone_name: &str,
    time_sync_info: &time_sync::TimeSyncInfo,
) -> Result<String> {
    generate_json_output_impl(location, timezone, city_name, dt, timezone_name, time_sync_info, None)
}

#[cfg(feature = "ai-insights")]
fn generate_json_output_impl(
    location: &Location,
    timezone: &Tz,
    city_name: Option<String>,
    dt: &DateTime<Tz>,
    timezone_name: &str,
    time_sync_info: &time_sync::TimeSyncInfo,
    ai_config: Option<&ai::AiConfig>,
) -> Result<String> {
    // Calculate sun position and events
    let sun_pos = sun::solar_position(location, dt);
    let sun_events = SunEvents {
        sunrise: sun::solar_event_time(location, dt, sun::SolarEvent::Sunrise)
            .map(|t| t.format("%Y-%m-%d %H:%M:%S %Z").to_string()),
        sunset: sun::solar_event_time(location, dt, sun::SolarEvent::Sunset)
            .map(|t| t.format("%Y-%m-%d %H:%M:%S %Z").to_string()),
        solar_noon: sun::solar_event_time(location, dt, sun::SolarEvent::SolarNoon)
            .map(|t| t.format("%Y-%m-%d %H:%M:%S %Z").to_string()),
        civil_dawn: sun::solar_event_time(location, dt, sun::SolarEvent::CivilDawn)
            .map(|t| t.format("%Y-%m-%d %H:%M:%S %Z").to_string()),
        civil_dusk: sun::solar_event_time(location, dt, sun::SolarEvent::CivilDusk)
            .map(|t| t.format("%Y-%m-%d %H:%M:%S %Z").to_string()),
        nautical_dawn: sun::solar_event_time(location, dt, sun::SolarEvent::NauticalDawn)
            .map(|t| t.format("%Y-%m-%d %H:%M:%S %Z").to_string()),
        nautical_dusk: sun::solar_event_time(location, dt, sun::SolarEvent::NauticalDusk)
            .map(|t| t.format("%Y-%m-%d %H:%M:%S %Z").to_string()),
        astronomical_dawn: sun::solar_event_time(location, dt, sun::SolarEvent::AstronomicalDawn)
            .map(|t| t.format("%Y-%m-%d %H:%M:%S %Z").to_string()),
        astronomical_dusk: sun::solar_event_time(location, dt, sun::SolarEvent::AstronomicalDusk)
            .map(|t| t.format("%Y-%m-%d %H:%M:%S %Z").to_string()),
    };

    // Calculate moon position and events
    let moon_pos = moon::lunar_position(location, dt);
    let moon_events = MoonEvents {
        moonrise: moon::lunar_event_time(location, dt, moon::LunarEvent::Moonrise)
            .map(|t| t.format("%Y-%m-%d %H:%M:%S %Z").to_string()),
        moonset: moon::lunar_event_time(location, dt, moon::LunarEvent::Moonset)
            .map(|t| t.format("%Y-%m-%d %H:%M:%S %Z").to_string()),
    };

    // Lunar phases for the month
    let phases = moon::lunar_phases(dt.year(), dt.month());
    let lunar_phases: Vec<LunarPhaseData> = phases
        .iter()
        .map(|p| {
            let phase_type = match p.phase_type {
                moon::LunarPhaseType::NewMoon => "new_moon",
                moon::LunarPhaseType::FirstQuarter => "first_quarter",
                moon::LunarPhaseType::FullMoon => "full_moon",
                moon::LunarPhaseType::LastQuarter => "last_quarter",
            };
            LunarPhaseData {
                phase_type: phase_type.to_string(),
                datetime: p.datetime.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
            }
        })
        .collect();

    let city_name_ref = city_name.as_deref();
    let ai_insights = if let Some(cfg) = ai_config {
        if cfg.enabled {
            let events =
                events::collect_events_within_window(location, dt, chrono::Duration::hours(12));
            let next_idx = events.iter().position(|(time, _)| *time > *dt);
            let summaries = ai::prepare_event_summaries(&events, dt, next_idx);

            let ai_data = ai::build_ai_data(ai::AiDataContext {
                location,
                timezone,
                dt,
                city_name: city_name_ref,
                sun_pos: &sun_pos,
                moon_pos: &moon_pos,
                events: summaries,
                time_sync_info,
                lunar_phases: &phases,
            });

            let outcome = match ai::fetch_insights(cfg, &ai_data) {
                Ok(outcome) => outcome,
                Err(err) => ai::AiOutcome::from_error(&cfg.model, err),
            };

            Some(build_ai_insights(&outcome, timezone))
        } else {
            None
        }
    } else {
        None
    };

    let output = JsonOutput {
        location: LocationData {
            latitude: location.latitude.value(),
            longitude: location.longitude.value(),
            timezone: timezone_name.to_string(),
            city: city_name,
        },
        datetime: DateTimeData {
            local: dt.format("%Y-%m-%d %H:%M:%S %Z").to_string(),
            utc: dt
                .with_timezone(&chrono::Utc)
                .format("%Y-%m-%d %H:%M:%S UTC")
                .to_string(),
            timezone_offset: dt.format("%:z").to_string(),
            time_sync: build_time_sync_data(time_sync_info),
        },
        sun: SunData {
            position: PositionData {
                altitude: sun_pos.altitude,
                azimuth: sun_pos.azimuth,
                azimuth_compass: coordinates::azimuth_to_compass(sun_pos.azimuth).to_string(),
            },
            events: sun_events,
        },
        moon: MoonData {
            position: MoonPositionData {
                altitude: moon_pos.altitude,
                azimuth: moon_pos.azimuth,
                azimuth_compass: coordinates::azimuth_to_compass(moon_pos.azimuth).to_string(),
                distance_km: moon_pos.distance,
                angular_diameter_arcmin: moon_pos.angular_diameter,
            },
            events: moon_events,
            phase: PhaseData {
                name: moon::phase_name(moon_pos.phase_angle).to_string(),
                emoji: moon::phase_emoji(moon_pos.phase_angle).to_string(),
                angle_degrees: moon_pos.phase_angle,
                illumination_percent: moon_pos.illumination * 100.0,
            },
        },
        lunar_phases,
        ai_insights,
    };

    Ok(serde_json::to_string_pretty(&output)?)
}

#[cfg(not(feature = "ai-insights"))]
fn generate_json_output_impl(
    location: &Location,
    timezone: &Tz,
    city_name: Option<String>,
    dt: &DateTime<Tz>,
    timezone_name: &str,
    time_sync_info: &time_sync::TimeSyncInfo,
    _ai_config: Option<&()>,  // Placeholder parameter for type consistency
) -> Result<String> {
    // Calculate sun position and events
    let sun_pos = sun::solar_position(location, dt);
    let sun_events = SunEvents {
        sunrise: sun::solar_event_time(location, dt, sun::SolarEvent::Sunrise)
            .map(|t| t.format("%Y-%m-%d %H:%M:%S %Z").to_string()),
        sunset: sun::solar_event_time(location, dt, sun::SolarEvent::Sunset)
            .map(|t| t.format("%Y-%m-%d %H:%M:%S %Z").to_string()),
        solar_noon: sun::solar_event_time(location, dt, sun::SolarEvent::SolarNoon)
            .map(|t| t.format("%Y-%m-%d %H:%M:%S %Z").to_string()),
        civil_dawn: sun::solar_event_time(location, dt, sun::SolarEvent::CivilDawn)
            .map(|t| t.format("%Y-%m-%d %H:%M:%S %Z").to_string()),
        civil_dusk: sun::solar_event_time(location, dt, sun::SolarEvent::CivilDusk)
            .map(|t| t.format("%Y-%m-%d %H:%M:%S %Z").to_string()),
        nautical_dawn: sun::solar_event_time(location, dt, sun::SolarEvent::NauticalDawn)
            .map(|t| t.format("%Y-%m-%d %H:%M:%S %Z").to_string()),
        nautical_dusk: sun::solar_event_time(location, dt, sun::SolarEvent::NauticalDusk)
            .map(|t| t.format("%Y-%m-%d %H:%M:%S %Z").to_string()),
        astronomical_dawn: sun::solar_event_time(location, dt, sun::SolarEvent::AstronomicalDawn)
            .map(|t| t.format("%Y-%m-%d %H:%M:%S %Z").to_string()),
        astronomical_dusk: sun::solar_event_time(location, dt, sun::SolarEvent::AstronomicalDusk)
            .map(|t| t.format("%Y-%m-%d %H:%M:%S %Z").to_string()),
    };

    // Calculate moon position and events
    let moon_pos = moon::lunar_position(location, dt);
    let moon_events = MoonEvents {
        moonrise: moon::lunar_event_time(location, dt, moon::LunarEvent::Moonrise)
            .map(|t| t.format("%Y-%m-%d %H:%M:%S %Z").to_string()),
        moonset: moon::lunar_event_time(location, dt, moon::LunarEvent::Moonset)
            .map(|t| t.format("%Y-%m-%d %H:%M:%S %Z").to_string()),
    };

    // Calculate lunar phases for current month
    let phases = moon::lunar_phases(dt.year(), dt.month());
    let lunar_phases = phases
        .iter()
        .map(|p| {
            let phase_type = match p.phase_type {
                moon::LunarPhaseType::NewMoon => "new_moon",
                moon::LunarPhaseType::FirstQuarter => "first_quarter",
                moon::LunarPhaseType::FullMoon => "full_moon",
                moon::LunarPhaseType::LastQuarter => "last_quarter",
            };
            LunarPhaseData {
                phase_type: phase_type.to_string(),
                datetime: p.datetime.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
            }
        })
        .collect();

    let output = JsonOutput {
        location: LocationData {
            latitude: location.latitude.value(),
            longitude: location.longitude.value(),
            timezone: timezone_name.to_string(),
            city: city_name,
        },
        datetime: DateTimeData {
            local: dt.format("%Y-%m-%d %H:%M:%S %Z").to_string(),
            utc: dt
                .with_timezone(&chrono::Utc)
                .format("%Y-%m-%d %H:%M:%S UTC")
                .to_string(),
            timezone_offset: dt.format("%:z").to_string(),
            time_sync: build_time_sync_data(time_sync_info),
        },
        sun: SunData {
            position: PositionData {
                altitude: sun_pos.altitude,
                azimuth: sun_pos.azimuth,
                azimuth_compass: coordinates::azimuth_to_compass(sun_pos.azimuth).to_string(),
            },
            events: sun_events,
        },
        moon: MoonData {
            position: MoonPositionData {
                altitude: moon_pos.altitude,
                azimuth: moon_pos.azimuth,
                azimuth_compass: coordinates::azimuth_to_compass(moon_pos.azimuth).to_string(),
                distance_km: moon_pos.distance,
                angular_diameter_arcmin: moon_pos.angular_diameter,
            },
            events: moon_events,
            phase: PhaseData {
                name: moon::phase_name(moon_pos.phase_angle).to_string(),
                emoji: moon::phase_emoji(moon_pos.phase_angle).to_string(),
                angle_degrees: moon_pos.phase_angle,
                illumination_percent: moon_pos.illumination * 100.0,
            },
        },
        lunar_phases,
        ai_insights: None,  // AI insights not available without the feature
    };

    Ok(serde_json::to_string_pretty(&output)?)
}

fn build_time_sync_data(time_sync_info: &time_sync::TimeSyncInfo) -> TimeSyncData {
    match (time_sync_info.delta, time_sync_info.direction()) {
        (Some(delta), Some(direction)) => TimeSyncData {
            source: time_sync_info.source.to_string(),
            delta_seconds: time_sync_info.delta_seconds(),
            offset_display: Some(time_sync::format_offset(delta)),
            status: time_sync::direction_code(direction).to_string(),
            error: None,
        },
        (Some(delta), None) => TimeSyncData {
            source: time_sync_info.source.to_string(),
            delta_seconds: time_sync_info.delta_seconds(),
            offset_display: Some(time_sync::format_offset(delta)),
            status: "measurable".to_string(),
            error: None,
        },
        _ => TimeSyncData {
            source: time_sync_info.source.to_string(),
            delta_seconds: None,
            offset_display: None,
            status: if time_sync_info.error.is_some() {
                "error".to_string()
            } else {
                "unavailable".to_string()
            },
            error: time_sync_info.error.clone(),
        },
    }
}

#[cfg(feature = "ai-insights")]
fn build_ai_insights(outcome: &ai::AiOutcome, timezone: &Tz) -> AiInsightsData {
    let elapsed = Utc::now().signed_duration_since(outcome.updated_at);
    let elapsed_secs = elapsed.num_seconds().max(0);
    let minutes = elapsed_secs / 60;
    let seconds = elapsed_secs % 60;
    let elapsed_display = format!("Updated {:02}:{:02} ago", minutes, seconds);

    AiInsightsData {
        model: outcome.model.clone(),
        updated_at: outcome
            .updated_at
            .with_timezone(timezone)
            .format("%Y-%m-%d %H:%M:%S %Z")
            .to_string(),
        updated_elapsed: Some(elapsed_display),
        summary: outcome.content.clone(),
        error: outcome.error.clone(),
    }
}
