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

/// Top-level JSON output structure for astronomical data.
///
/// Contains all calculated positions, events, phases, and optional AI insights.
#[derive(Serialize)]
pub struct JsonOutput {
    /// Geographic location information
    pub location: LocationData,
    /// Current date and time information
    pub datetime: DateTimeData,
    /// Solar position and events
    pub sun: SunData,
    /// Lunar position, events, and phase
    pub moon: MoonData,
    /// Lunar phases for current month
    pub lunar_phases: Vec<LunarPhaseData>,
    /// Next dark-sky window (sun below -18°, moon below horizon)
    pub dark_sky_window: Option<DarkSkyWindowData>,
    /// Major planets (Mercury through Neptune)
    pub planets: Vec<PlanetData>,
    /// Upcoming equinoxes and solstices
    pub seasons: Vec<SeasonData>,
    /// Optional AI-generated insights
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ai_insights: Option<AiInsightsData>,
}

/// Geographic location data for JSON output.
#[derive(Serialize)]
pub struct LocationData {
    /// Latitude in decimal degrees (WGS84)
    pub latitude: f64,
    /// Longitude in decimal degrees (WGS84)
    pub longitude: f64,
    /// IANA timezone identifier
    pub timezone: String,
    /// Optional city name
    pub city: Option<String>,
}

/// Date and time information for JSON output.
#[derive(Serialize)]
pub struct DateTimeData {
    /// Local time with timezone
    pub local: String,
    /// UTC time
    pub utc: String,
    /// Timezone offset (e.g., "+05:00")
    pub timezone_offset: String,
    /// NTP time synchronization status
    pub time_sync: TimeSyncData,
}

/// Solar position and events for JSON output.
#[derive(Serialize)]
pub struct SunData {
    /// Current sun position (altitude, azimuth)
    pub position: PositionData,
    /// Solar events (sunrise, sunset, twilight times)
    pub events: SunEvents,
    /// Golden hour and blue hour periods
    pub photography: PhotographyData,
}

/// Lunar position, events, and phase for JSON output.
#[derive(Serialize)]
pub struct MoonData {
    /// Current moon position (altitude, azimuth, distance, size)
    pub position: MoonPositionData,
    /// Lunar events (moonrise, moonset)
    pub events: MoonEvents,
    /// Current lunar phase details
    pub phase: PhaseData,
    /// Next lunar perigee and apogee
    pub apsides: Vec<ApsisData>,
}

/// A time period with start and end, formatted as "YYYY-MM-DD HH:MM:SS TZ".
#[derive(Serialize)]
pub struct PeriodData {
    /// Period start time
    pub start: String,
    /// Period end time
    pub end: String,
}

/// Golden hour and blue hour periods for JSON output.
///
/// Periods are `None` when the boundary altitudes are not crossed on the
/// given day (polar day/night and high-latitude edge cases).
#[derive(Serialize)]
pub struct PhotographyData {
    /// Morning blue hour (civil dawn → sun at -4°)
    pub morning_blue: Option<PeriodData>,
    /// Morning golden hour (sun -4° → +6°, rising)
    pub morning_golden: Option<PeriodData>,
    /// Evening golden hour (sun +6° → -4°, setting)
    pub evening_golden: Option<PeriodData>,
    /// Evening blue hour (sun at -4° → civil dusk)
    pub evening_blue: Option<PeriodData>,
}

/// Next dark-sky window for JSON output.
///
/// A dark-sky window has the sun below astronomical twilight (-18°) and the
/// moon below the horizon (with a 15-minute moon-glow buffer).
#[derive(Serialize)]
pub struct DarkSkyWindowData {
    /// Window start time
    pub start: String,
    /// Window end time; `None` when the window extends beyond the 36-hour scan
    pub end: Option<String>,
    /// Window duration in minutes; `None` when the end is unknown
    pub duration_minutes: Option<i64>,
}

/// A lunar apsis (perigee or apogee) for JSON output.
#[derive(Serialize)]
pub struct ApsisData {
    /// "perigee" or "apogee"
    pub kind: String,
    /// Local time of the distance extremum
    pub datetime: String,
    /// Earth–moon distance in kilometers
    pub distance_km: f64,
}

/// An upcoming equinox or solstice for JSON output.
#[derive(Serialize)]
pub struct SeasonData {
    /// Event name (e.g. "March Equinox")
    pub event: String,
    /// Local time of the event
    pub datetime: String,
}

/// A bright planet's position and events for JSON output.
#[derive(Serialize)]
pub struct PlanetData {
    /// Planet name
    pub name: String,
    /// Altitude in degrees (+ above horizon, - below)
    pub altitude: f64,
    /// Azimuth in degrees (0° = North, clockwise)
    pub azimuth: f64,
    /// Compass direction (e.g., "NE", "SW")
    pub azimuth_compass: String,
    /// Geocentric distance in astronomical units
    pub distance_au: f64,
    /// Approximate visual magnitude (lower is brighter)
    pub magnitude: f64,
    /// Angular distance from the sun in degrees
    pub elongation_degrees: f64,
    /// Rise time, if the planet rises on this date
    pub rise: Option<String>,
    /// Set time, if the planet sets on this date
    pub set: Option<String>,
}

/// Celestial body position (sun/moon) for JSON output.
#[derive(Serialize)]
pub struct PositionData {
    /// Altitude in degrees (+ above horizon, - below)
    pub altitude: f64,
    /// Azimuth in degrees (0° = North, clockwise)
    pub azimuth: f64,
    /// Compass direction (e.g., "NE", "SW")
    pub azimuth_compass: String,
}

/// Lunar position with distance and angular size for JSON output.
#[derive(Serialize)]
pub struct MoonPositionData {
    /// Altitude in degrees (+ above horizon, - below)
    pub altitude: f64,
    /// Azimuth in degrees (0° = North, clockwise)
    pub azimuth: f64,
    /// Compass direction (e.g., "NE", "SW")
    pub azimuth_compass: String,
    /// Distance from Earth in kilometers
    pub distance_km: f64,
    /// Angular diameter in arcminutes
    pub angular_diameter_arcmin: f64,
}

/// Solar events for JSON output.
///
/// All times are formatted as "YYYY-MM-DD HH:MM:SS TZ".
/// Events are `None` if they don't occur on the given day (e.g., polar regions).
#[derive(Serialize)]
pub struct SunEvents {
    /// Sunrise time
    pub sunrise: Option<String>,
    /// Sunset time
    pub sunset: Option<String>,
    /// Solar noon (sun at highest altitude)
    pub solar_noon: Option<String>,
    /// Civil dawn (sun at -6° altitude)
    pub civil_dawn: Option<String>,
    /// Civil dusk (sun at -6° altitude)
    pub civil_dusk: Option<String>,
    /// Nautical dawn (sun at -12° altitude)
    pub nautical_dawn: Option<String>,
    /// Nautical dusk (sun at -12° altitude)
    pub nautical_dusk: Option<String>,
    /// Astronomical dawn (sun at -18° altitude)
    pub astronomical_dawn: Option<String>,
    /// Astronomical dusk (sun at -18° altitude)
    pub astronomical_dusk: Option<String>,
}

/// Lunar events for JSON output.
///
/// Times are formatted as "YYYY-MM-DD HH:MM:SS TZ".
/// Events are `None` if they don't occur on the given day.
#[derive(Serialize)]
pub struct MoonEvents {
    /// Moonrise time
    pub moonrise: Option<String>,
    /// Moonset time
    pub moonset: Option<String>,
}

/// Current lunar phase details for JSON output.
#[derive(Serialize)]
pub struct PhaseData {
    /// Phase name (e.g., "Full Moon", "Waxing Crescent")
    pub name: String,
    /// Emoji representation of phase
    pub emoji: String,
    /// Phase angle in degrees (0° = new, 180° = full)
    pub angle_degrees: f64,
    /// Illumination percentage (0-100)
    pub illumination_percent: f64,
}

/// Lunar phase event data for JSON output.
#[derive(Serialize)]
pub struct LunarPhaseData {
    /// Phase type: "new_moon", "first_quarter", "full_moon", "last_quarter"
    pub phase_type: String,
    /// UTC timestamp of phase event
    pub datetime: String,
    /// Whether this full moon is a supermoon (only present for full moons)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supermoon: Option<bool>,
}

/// NTP time synchronization status for JSON output.
#[derive(Serialize)]
pub struct TimeSyncData {
    /// NTP server source (e.g., "time.google.com (NTP)")
    pub source: String,
    /// Clock offset in seconds (system - NTP time)
    pub delta_seconds: Option<f64>,
    /// Human-readable offset (e.g., "+2.3s", "-150.0ms")
    pub offset_display: Option<String>,
    /// Status code: "ahead", "behind", "in_sync", "error", "unavailable"
    pub status: String,
    /// Error message if time sync failed
    pub error: Option<String>,
}

/// AI-generated insights for JSON output.
#[derive(Serialize)]
pub struct AiInsightsData {
    /// Ollama model used to generate insights
    pub model: String,
    /// Timestamp when insights were generated
    pub updated_at: String,
    /// Time elapsed since last update (e.g., "Updated 02:15 ago")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_elapsed: Option<String>,
    /// Generated insights text
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    /// Error message if insights generation failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Generates JSON output for astronomical data (with AI insights support).
///
/// Creates a complete JSON document containing positions, events, phases,
/// time sync status, and optional AI-generated insights.
///
/// # Arguments
///
/// * `location` - Geographic location
/// * `timezone` - Timezone for event times
/// * `city_name` - Optional city name
/// * `dt` - Current date/time
/// * `timezone_name` - Timezone display name
/// * `time_sync_info` - NTP time sync status
/// * `ai_config` - AI configuration (if ai-insights feature enabled)
///
/// # Returns
///
/// A pretty-printed JSON string containing all astronomical data.
///
/// # Errors
///
/// Returns an error if JSON serialization fails.
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
    generate_json_output_impl(
        location,
        timezone,
        city_name,
        dt,
        timezone_name,
        time_sync_info,
        Some(ai_config),
    )
}

/// Generates JSON output for astronomical data (without AI insights).
///
/// Creates a complete JSON document containing positions, events, phases,
/// and time sync status.
///
/// # Arguments
///
/// * `location` - Geographic location
/// * `timezone` - Timezone for event times
/// * `city_name` - Optional city name
/// * `dt` - Current date/time
/// * `timezone_name` - Timezone display name
/// * `time_sync_info` - NTP time sync status
///
/// # Returns
///
/// A pretty-printed JSON string containing all astronomical data.
///
/// # Errors
///
/// Returns an error if JSON serialization fails.
#[cfg(not(feature = "ai-insights"))]
pub fn generate_json_output(
    location: &Location,
    timezone: &Tz,
    city_name: Option<String>,
    dt: &DateTime<Tz>,
    timezone_name: &str,
    time_sync_info: &time_sync::TimeSyncInfo,
) -> Result<String> {
    generate_json_output_impl(
        location,
        timezone,
        city_name,
        dt,
        timezone_name,
        time_sync_info,
        None,
    )
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
    let lunar_phases = build_lunar_phase_data(&phases);

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
            photography: build_photography_data(location, dt),
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
            apsides: build_apsides_data(dt, timezone),
        },
        lunar_phases,
        dark_sky_window: build_dark_sky_data(location, dt),
        planets: build_planets_data(location, dt),
        seasons: build_seasons_data(dt, timezone),
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
    _ai_config: Option<&()>, // Placeholder parameter for type consistency
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
    let lunar_phases = build_lunar_phase_data(&phases);

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
            photography: build_photography_data(location, dt),
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
            apsides: build_apsides_data(dt, timezone),
        },
        lunar_phases,
        dark_sky_window: build_dark_sky_data(location, dt),
        planets: build_planets_data(location, dt),
        seasons: build_seasons_data(dt, timezone),
        ai_insights: None, // AI insights not available without the feature
    };

    Ok(serde_json::to_string_pretty(&output)?)
}

fn format_local_time(dt: &DateTime<Tz>) -> String {
    dt.format("%Y-%m-%d %H:%M:%S %Z").to_string()
}

fn build_photography_data(location: &Location, dt: &DateTime<Tz>) -> PhotographyData {
    let to_period = |period: Option<(DateTime<Tz>, DateTime<Tz>)>| {
        period.map(|(start, end)| PeriodData {
            start: format_local_time(&start),
            end: format_local_time(&end),
        })
    };

    let periods = sun::photo_periods(location, dt);
    PhotographyData {
        morning_blue: to_period(periods.morning_blue),
        morning_golden: to_period(periods.morning_golden),
        evening_golden: to_period(periods.evening_golden),
        evening_blue: to_period(periods.evening_blue),
    }
}

fn build_dark_sky_data(location: &Location, dt: &DateTime<Tz>) -> Option<DarkSkyWindowData> {
    events::next_dark_window(location, dt).map(|(start, end)| DarkSkyWindowData {
        start: format_local_time(&start),
        end: end.as_ref().map(format_local_time),
        duration_minutes: end.map(|e| e.signed_duration_since(start).num_minutes()),
    })
}

fn build_apsides_data(dt: &DateTime<Tz>, timezone: &Tz) -> Vec<ApsisData> {
    moon::next_lunar_apsides(dt)
        .into_iter()
        .map(|apsis| ApsisData {
            kind: match apsis.kind {
                moon::LunarApsisKind::Perigee => "perigee".to_string(),
                moon::LunarApsisKind::Apogee => "apogee".to_string(),
            },
            datetime: format_local_time(&apsis.datetime.with_timezone(timezone)),
            distance_km: apsis.distance_km,
        })
        .collect()
}

fn build_seasons_data(dt: &DateTime<Tz>, timezone: &Tz) -> Vec<SeasonData> {
    seasons::next_seasonal_events(dt, 2)
        .into_iter()
        .map(|event| SeasonData {
            event: event.kind.name().to_string(),
            datetime: format_local_time(&event.datetime.with_timezone(timezone)),
        })
        .collect()
}

fn build_planets_data(location: &Location, dt: &DateTime<Tz>) -> Vec<PlanetData> {
    planets::Planet::ALL
        .into_iter()
        .map(|planet| {
            let pos = planets::planet_position(planet, location, dt);
            PlanetData {
                name: planet.name().to_string(),
                altitude: pos.altitude,
                azimuth: pos.azimuth,
                azimuth_compass: coordinates::azimuth_to_compass(pos.azimuth).to_string(),
                distance_au: pos.distance_au,
                magnitude: pos.magnitude,
                elongation_degrees: pos.elongation,
                rise: planets::planet_event_time(planet, location, dt, planets::PlanetEvent::Rise)
                    .map(|t| format_local_time(&t)),
                set: planets::planet_event_time(planet, location, dt, planets::PlanetEvent::Set)
                    .map(|t| format_local_time(&t)),
            }
        })
        .collect()
}

fn build_lunar_phase_data(phases: &[moon::LunarPhase]) -> Vec<LunarPhaseData> {
    phases
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
                supermoon: (p.phase_type == moon::LunarPhaseType::FullMoon)
                    .then(|| moon::is_supermoon(p)),
            }
        })
        .collect()
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
