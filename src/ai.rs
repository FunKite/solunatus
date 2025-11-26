//! AI-powered astronomical insights via Ollama integration.
//!
//! This module provides optional integration with local Ollama LLM servers to generate
//! natural language insights about astronomical data. When enabled, it automatically
//! queries an Ollama model with structured astronomical data and displays narrative
//! analysis in the TUI.
//!
//! # Features
//!
//! - **Local Ollama Integration**: Connects to local Ollama server (default: localhost:11434)
//! - **Structured Data**: Sends JSON payloads with positions, events, phases, time sync
//! - **Auto-refresh**: Configurable refresh intervals (1-60 minutes)
//! - **Manual Refresh**: User-triggered updates via TUI hotkey
//! - **Model Selection**: Works with any Ollama model (default: llama3.2:latest)
//! - **Server Discovery**: Probes server for available models
//!
//! # Configuration
//!
//! AI insights can be configured via CLI arguments or the TUI:
//! - `--ai-insights`: Enable AI insights
//! - `--ai-server <url>`: Ollama server URL
//! - `--ai-model <model>`: Model name
//! - `--ai-refresh-minutes <n>`: Refresh interval (1-60 minutes)
//!
//! # Examples
//!
//! ```no_run
//! use solunatus::ai::{AiConfig, fetch_insights, AiDataContext, build_ai_data};
//! use solunatus::astro::Location;
//! use chrono::Utc;
//! use chrono_tz::America::New_York;
//!
//! // Configure AI
//! let config = AiConfig {
//!     enabled: true,
//!     server: "http://localhost:11434".to_string(),
//!     model: "llama3.2:latest".to_string(),
//!     refresh: std::time::Duration::from_secs(120),
//!     refresh_mode: solunatus::config::AiRefreshMode::AutoAndManual,
//! };
//!
//! // Build AI data context and fetch insights
//! // (see full example in function documentation)
//! ```
//!
//! # Security
//!
//! - Uses TLS verification for HTTPS connections
//! - Configurable timeouts prevent hanging on slow networks
//! - Error messages are truncated to prevent log spam

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration as StdDuration;

use crate::astro::moon::{LunarPhase, LunarPhaseType, LunarPosition};
use crate::astro::sun::SolarPosition;
use crate::astro::{self, coordinates};
use crate::time_sync::{self, TimeSyncInfo};

const DEFAULT_TIMEOUT_SECS: u64 = 15;
const USER_AGENT: &str = "Solunatus AI Insights";
const ERROR_SUMMARY_LIMIT: usize = 120;

/// Build a secure HTTP client with proper timeout and TLS verification
fn build_secure_http_client(timeout: StdDuration) -> Result<Client> {
    Client::builder()
        .timeout(timeout)
        .user_agent(USER_AGENT)
        .danger_accept_invalid_certs(false) // Explicitly enforce TLS verification
        .build()
        .context("Failed to build HTTP client")
}

/// Configuration for AI insights integration.
///
/// Controls how and when the application connects to an Ollama server
/// to fetch natural language insights about astronomical data.
#[derive(Debug, Clone)]
pub struct AiConfig {
    /// Whether AI insights are enabled
    pub enabled: bool,
    /// Ollama server URL (e.g., "http://localhost:11434")
    pub server: String,
    /// Ollama model name (e.g., "llama3.2:latest")
    pub model: String,
    /// Auto-refresh interval
    pub refresh: StdDuration,
    /// Refresh mode (auto or manual only)
    pub refresh_mode: crate::config::AiRefreshMode,
}

/// Summary of a single astronomical event for AI context.
///
/// Contains event name, timing, and next-event indicator.
#[derive(Debug, Clone, Serialize)]
pub struct AiEventSummary {
    /// Event name (e.g., "Sunrise", "Moonset")
    pub name: String,
    /// Local time string
    pub local_time: String,
    /// Relative time description (e.g., "in 2h 15m")
    pub relative_time: String,
    /// Whether this is the next upcoming event
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_next: Option<bool>,
}

/// Complete astronomical data payload for AI model.
///
/// This is the primary data structure sent to the Ollama API,
/// containing all astronomical information for the current moment.
#[derive(Debug, Clone, Serialize)]
pub struct AiData {
    /// Current time in local timezone
    pub timestamp_local: String,
    /// Current time in UTC
    pub timestamp_utc: String,
    /// IANA timezone identifier
    pub timezone: String,
    /// Geographic location data
    pub location: AiLocation,
    /// Current sun position
    pub sun: AiSunData,
    /// Current moon position and phase
    pub moon: AiMoonData,
    /// Upcoming astronomical events
    pub events: Vec<AiEventSummary>,
    /// NTP time sync status
    pub time_sync: AiTimeSync,
    /// Lunar phases for current month
    pub lunar_phases: Vec<AiLunarPhase>,
}

/// Geographic location data for AI payload.
#[derive(Debug, Clone, Serialize)]
pub struct AiLocation {
    /// Latitude in decimal degrees
    pub latitude_deg: f64,
    /// Longitude in decimal degrees
    pub longitude_deg: f64,
    /// Optional city name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,
}

/// Solar position data for AI payload.
#[derive(Debug, Clone, Serialize)]
pub struct AiSunData {
    /// Sun altitude in degrees
    pub altitude_deg: f64,
    /// Sun azimuth in degrees (0° = North)
    pub azimuth_deg: f64,
    /// Compass direction (e.g., "SE")
    pub azimuth_compass: String,
}

/// Lunar position and phase data for AI payload.
#[derive(Debug, Clone, Serialize)]
pub struct AiMoonData {
    /// Moon altitude in degrees
    pub altitude_deg: f64,
    /// Moon azimuth in degrees (0° = North)
    pub azimuth_deg: f64,
    /// Compass direction (e.g., "WSW")
    pub azimuth_compass: String,
    /// Illumination percentage (0-100)
    pub illumination_percent: f64,
    /// Phase name (e.g., "Waxing Gibbous")
    pub phase_name: String,
    /// Phase angle in degrees (0° = new, 180° = full)
    pub phase_angle_deg: f64,
    /// Distance from Earth in kilometers
    pub distance_km: f64,
    /// Angular diameter in arcminutes
    pub angular_diameter_arcmin: f64,
}

/// NTP time synchronization status for AI payload.
#[derive(Debug, Clone, Serialize)]
pub struct AiTimeSync {
    /// NTP server source
    pub source: String,
    /// Clock offset in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delta_seconds: Option<f64>,
    /// Human-readable offset display
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset_display: Option<String>,
    /// Direction code ("ahead", "behind", "in_sync")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction_code: Option<String>,
    /// Direction description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction_description: Option<String>,
    /// Error message if sync failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Lunar phase event data for AI payload.
#[derive(Debug, Clone, Serialize)]
pub struct AiLunarPhase {
    /// Phase name (e.g., "Full Moon")
    pub name: String,
    /// Phase emoji
    pub emoji: String,
    /// Phase type (e.g., "FullMoon")
    pub phase_type: String,
    /// Local time string
    pub datetime_local: String,
    /// UTC time string
    pub datetime_utc: String,
}

/// Result of an AI insights request.
///
/// Contains either the generated insights text or an error message,
/// along with timing metadata.
#[derive(Debug, Clone)]
pub struct AiOutcome {
    /// The Ollama model that generated the response
    pub model: String,
    /// Generated insights text (None if request failed)
    pub content: Option<String>,
    /// Error message (None if request succeeded)
    pub error: Option<String>,
    /// UTC timestamp when this result was obtained
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
struct OllamaResponse {
    response: String,
}

#[derive(Debug, Deserialize)]
struct OllamaModelEntry {
    name: String,
}

#[derive(Debug, Deserialize)]
struct OllamaTagsResponse {
    models: Vec<OllamaModelEntry>,
}

#[derive(Serialize)]
struct OllamaRequest<'a> {
    model: &'a str,
    prompt: &'a str,
    stream: bool,
}

impl AiConfig {
    pub fn from_args(args: &crate::cli::Args) -> Result<Self> {
        let enabled = args.ai_insights;
        let refresh_minutes = args.ai_refresh_minutes;
        if !(1..=60).contains(&refresh_minutes) {
            return Err(anyhow!(
                "AI refresh minutes must be between 1 and 60 (got {})",
                refresh_minutes
            ));
        }

        Ok(Self {
            enabled,
            server: Self::normalized_server(enabled, &args.ai_server),
            model: args.ai_model.trim().to_string(),
            refresh: StdDuration::from_secs(refresh_minutes * 60),
            refresh_mode: crate::config::AiRefreshMode::AutoAndManual,
        })
    }

    pub fn merge_with_saved(mut self, saved_settings: &crate::config::AiSettings) -> Self {
        self.refresh_mode = saved_settings.refresh_mode;
        self
    }

    pub fn endpoint(&self) -> String {
        format!("{}/api/generate", self.server)
    }

    pub fn refresh_minutes(&self) -> u64 {
        let mins = self.refresh.as_secs() / 60;
        if mins == 0 {
            1
        } else if mins > 60 {
            60
        } else {
            mins
        }
    }

    pub fn refresh_mode_label(&self) -> &'static str {
        match self.refresh_mode {
            crate::config::AiRefreshMode::AutoAndManual => "Auto & Manual",
            crate::config::AiRefreshMode::ManualOnly => "Manual Only",
        }
    }

    pub fn normalized_server(enabled: bool, server: &str) -> String {
        let mut value = server.trim().to_string();
        if value.is_empty() {
            value = "http://localhost:11434".to_string();
        }

        if enabled && !(value.starts_with("http://") || value.starts_with("https://")) {
            value = format!("http://{}", value);
        }

        value.trim_end_matches('/').to_string()
    }
}

impl AiOutcome {
    pub fn success(model: &str, content: String) -> Self {
        Self {
            model: model.to_string(),
            content: Some(content),
            error: None,
            updated_at: Utc::now(),
        }
    }

    pub fn from_error(model: &str, err: anyhow::Error) -> Self {
        Self {
            model: model.to_string(),
            content: None,
            error: Some(summarize_error(&err.to_string())),
            updated_at: Utc::now(),
        }
    }

    pub fn with_error_message(mut self, message: String) -> Self {
        self.error = Some(summarize_error(&message));
        self
    }
}

/// Prepares event summaries for AI payload.
///
/// Converts a list of astronomical events into AI-friendly summaries with
/// formatted times and relative time descriptions.
///
/// # Arguments
///
/// * `events` - Slice of (timestamp, event_name) tuples
/// * `reference` - Reference time for calculating relative times
/// * `next_index` - Optional index of the next upcoming event
///
/// # Returns
///
/// A vector of [`AiEventSummary`] structs ready for AI model input.
pub fn prepare_event_summaries(
    events: &[(DateTime<Tz>, &'static str)],
    reference: &DateTime<Tz>,
    next_index: Option<usize>,
) -> Vec<AiEventSummary> {
    events
        .iter()
        .enumerate()
        .map(|(idx, (time, name))| AiEventSummary {
            name: (*name).to_string(),
            local_time: time.format("%Y-%m-%d %H:%M:%S %Z").to_string(),
            relative_time: astro::time_utils::format_duration_detailed(
                astro::time_utils::time_until(reference, time),
            ),
            is_next: next_index.map(|n| n == idx),
        })
        .collect()
}

/// Context for building AI data payload.
///
/// Groups all necessary astronomical data for constructing an [`AiData`] payload.
pub struct AiDataContext<'a> {
    /// Geographic location
    pub location: &'a astro::Location,
    /// Timezone for time formatting
    pub timezone: &'a Tz,
    /// Current date/time
    pub dt: &'a DateTime<Tz>,
    /// Optional city name
    pub city_name: Option<&'a str>,
    /// Current solar position
    pub sun_pos: &'a SolarPosition,
    /// Current lunar position
    pub moon_pos: &'a LunarPosition,
    /// Upcoming events
    pub events: Vec<AiEventSummary>,
    /// Time sync status
    pub time_sync_info: &'a TimeSyncInfo,
    /// Lunar phases for current month
    pub lunar_phases: &'a [LunarPhase],
}

/// Builds the AI data payload from astronomical context.
///
/// Converts raw astronomical data into a structured JSON-serializable
/// format suitable for sending to an LLM.
///
/// # Arguments
///
/// * `ctx` - Context containing all astronomical data
///
/// # Returns
///
/// An [`AiData`] struct ready for JSON serialization and AI model input.
pub fn build_ai_data(ctx: AiDataContext) -> AiData {
    let location = ctx.location;
    let timezone = ctx.timezone;
    let dt = ctx.dt;
    let city_name = ctx.city_name;
    let sun_pos = ctx.sun_pos;
    let moon_pos = ctx.moon_pos;
    let events = ctx.events;
    let time_sync_info = ctx.time_sync_info;
    let lunar_phases = ctx.lunar_phases;
    let direction = time_sync_info.direction();

    AiData {
        timestamp_local: dt.format("%Y-%m-%d %H:%M:%S %Z").to_string(),
        timestamp_utc: dt
            .with_timezone(&Utc)
            .format("%Y-%m-%d %H:%M:%S UTC")
            .to_string(),
        timezone: timezone.name().to_string(),
        location: AiLocation {
            latitude_deg: location.latitude.value(),
            longitude_deg: location.longitude.value(),
            city: city_name.map(|c| c.to_string()),
        },
        sun: AiSunData {
            altitude_deg: sun_pos.altitude,
            azimuth_deg: sun_pos.azimuth,
            azimuth_compass: coordinates::azimuth_to_compass(sun_pos.azimuth).to_string(),
        },
        moon: AiMoonData {
            altitude_deg: moon_pos.altitude,
            azimuth_deg: moon_pos.azimuth,
            azimuth_compass: coordinates::azimuth_to_compass(moon_pos.azimuth).to_string(),
            illumination_percent: moon_pos.illumination * 100.0,
            phase_name: astro::moon::phase_name(moon_pos.phase_angle).to_string(),
            phase_angle_deg: moon_pos.phase_angle,
            distance_km: moon_pos.distance,
            angular_diameter_arcmin: moon_pos.angular_diameter,
        },
        events,
        time_sync: AiTimeSync {
            source: time_sync_info.source.to_string(),
            delta_seconds: time_sync_info.delta_seconds(),
            offset_display: time_sync_info
                .delta
                .map(time_sync::format_offset),
            direction_code: direction.map(|dir| time_sync::direction_code(dir).to_string()),
            direction_description: direction
                .map(|dir| time_sync::describe_direction(dir).to_string()),
            error: time_sync_info.error.clone(),
        },
        lunar_phases: lunar_phases
            .iter()
            .map(|phase| {
                let (name, emoji) = match phase.phase_type {
                    LunarPhaseType::NewMoon => ("New Moon", "🌑"),
                    LunarPhaseType::FirstQuarter => ("First Quarter", "🌓"),
                    LunarPhaseType::FullMoon => ("Full Moon", "🌕"),
                    LunarPhaseType::LastQuarter => ("Last Quarter", "🌗"),
                };

                AiLunarPhase {
                    name: name.to_string(),
                    emoji: emoji.to_string(),
                    phase_type: format!("{:?}", phase.phase_type),
                    datetime_local: phase
                        .datetime
                        .with_timezone(timezone)
                        .format("%Y-%m-%d %H:%M:%S %Z")
                        .to_string(),
                    datetime_utc: phase
                        .datetime
                        .with_timezone(&Utc)
                        .format("%Y-%m-%d %H:%M:%S UTC")
                        .to_string(),
                }
            })
            .collect(),
    }
}

/// Fetches AI insights from an Ollama server.
///
/// Sends astronomical data to the configured Ollama model and retrieves
/// natural language insights. Automatically adjusts timeout based on
/// the configured refresh interval.
///
/// # Arguments
///
/// * `config` - AI configuration (server, model, timeout)
/// * `data` - Astronomical data to send to the model
///
/// # Returns
///
/// An [`AiOutcome`] containing either the generated insights or an error.
///
/// # Errors
///
/// Returns an error if:
/// - AI insights are disabled in config
/// - Network connection fails
/// - Ollama server is unreachable
/// - Model request times out
/// - Server returns non-success status
///
/// # Examples
///
/// ```no_run
/// # use solunatus::ai::*;
/// # use std::time::Duration;
/// let config = AiConfig {
///     enabled: true,
///     server: "http://localhost:11434".to_string(),
///     model: "llama3.2:latest".to_string(),
///     refresh: Duration::from_secs(120),
///     refresh_mode: solunatus::config::AiRefreshMode::AutoAndManual,
/// };
///
/// // Assume `data` is built from AiDataContext
/// # let data = AiData {
/// #     timestamp_local: String::new(),
/// #     timestamp_utc: String::new(),
/// #     timezone: String::new(),
/// #     location: AiLocation { latitude_deg: 0.0, longitude_deg: 0.0, city: None },
/// #     sun: AiSunData { altitude_deg: 0.0, azimuth_deg: 0.0, azimuth_compass: String::new() },
/// #     moon: AiMoonData {
/// #         altitude_deg: 0.0, azimuth_deg: 0.0, azimuth_compass: String::new(),
/// #         illumination_percent: 0.0, phase_name: String::new(), phase_angle_deg: 0.0,
/// #         distance_km: 0.0, angular_diameter_arcmin: 0.0,
/// #     },
/// #     events: vec![],
/// #     time_sync: AiTimeSync {
/// #         source: String::new(), delta_seconds: None, offset_display: None,
/// #         direction_code: None, direction_description: None, error: None,
/// #     },
/// #     lunar_phases: vec![],
/// # };
///
/// match fetch_insights(&config, &data) {
///     Ok(outcome) => {
///         if let Some(text) = outcome.content {
///             println!("Insights: {}", text);
///         }
///     }
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// ```
pub fn fetch_insights(config: &AiConfig, data: &AiData) -> Result<AiOutcome> {
    if !config.enabled {
        return Err(anyhow!("AI insights are disabled"));
    }

    let prompt = build_prompt(data)?;
    let desired_timeout = if config.refresh > StdDuration::from_secs(1) {
        config.refresh - StdDuration::from_secs(1)
    } else {
        StdDuration::from_secs(DEFAULT_TIMEOUT_SECS)
    };
    let timeout = if desired_timeout >= StdDuration::from_secs(DEFAULT_TIMEOUT_SECS) {
        desired_timeout
    } else {
        StdDuration::from_secs(DEFAULT_TIMEOUT_SECS)
    };

    let client = build_secure_http_client(timeout)
        .context("failed to construct HTTP client for Ollama")?;

    let body = OllamaRequest {
        model: &config.model,
        prompt: &prompt,
        stream: false,
    };

    let response = client
        .post(config.endpoint())
        .json(&body)
        .send()
        .with_context(|| format!("failed to reach Ollama server at {}", config.server))?;

    if !response.status().is_success() {
        return Err(anyhow!(
            "Ollama server returned status {}",
            response.status()
        ));
    }

    let payload: OllamaResponse = response
        .json()
        .context("failed to parse Ollama response payload")?;

    let content = payload.response.trim().to_string();
    if content.is_empty() {
        Ok(AiOutcome {
            model: config.model.clone(),
            content: Some("No insights returned by model.".to_string()),
            error: None,
            updated_at: Utc::now(),
        })
    } else {
        Ok(AiOutcome::success(&config.model, content))
    }
}

fn build_prompt(data: &AiData) -> Result<String> {
    let data_json =
        serde_json::to_string_pretty(data).context("failed to serialize AI data payload")?;

    Ok(format!(
        "You are an astronomy specialist generating concise insights.\n\
         Requirements:\n\
         - Provide a single short paragraph of narrative analysis highlighting notable solar and lunar observations.\n\
         - Do not repeat raw numbers or tables that the user can already see; focus on interpretation and context.\n\
         - No bullet points, formatting, or questions. One response only with no follow-ups.\n\
         Data:\n{}\n\nInsights:",
        data_json
    ))
}

fn summarize_error(message: &str) -> String {
    if message.len() <= ERROR_SUMMARY_LIMIT {
        message.to_string()
    } else {
        let mut truncated = message[..ERROR_SUMMARY_LIMIT].to_string();
        truncated.push('…');
        truncated
    }
}

/// Probes an Ollama server to discover available models.
///
/// Queries the `/api/tags` endpoint to retrieve a list of installed models.
/// Useful for validating server connectivity and displaying model choices to users.
///
/// # Arguments
///
/// * `server` - Ollama server URL (e.g., "http://localhost:11434")
///
/// # Returns
///
/// A sorted, deduplicated vector of model names on success.
///
/// # Errors
///
/// Returns an error if:
/// - Server is unreachable
/// - Server returns non-success status
/// - Response cannot be parsed
/// - Server reports no installed models
///
/// # Examples
///
/// ```no_run
/// use solunatus::ai::probe_server;
///
/// match probe_server("http://localhost:11434") {
///     Ok(models) => {
///         println!("Available models:");
///         for model in models {
///             println!("  - {}", model);
///         }
///     }
///     Err(e) => eprintln!("Failed to probe server: {}", e),
/// }
/// ```
pub fn probe_server(server: &str) -> Result<Vec<String>> {
    let client = build_secure_http_client(StdDuration::from_secs(DEFAULT_TIMEOUT_SECS))
        .context("failed to construct HTTP client for Ollama")?;

    let endpoint = format!("{}/api/tags", server.trim_end_matches('/'));
    let response = client
        .get(&endpoint)
        .send()
        .with_context(|| format!("failed to reach Ollama server at {}", server))?;

    if !response.status().is_success() {
        return Err(anyhow!(
            "Ollama server returned status {} while listing models",
            response.status()
        ));
    }

    let tags: OllamaTagsResponse = response
        .json()
        .context("failed to parse Ollama model list")?;

    let mut models: Vec<String> = tags.models.into_iter().map(|entry| entry.name).collect();
    models.sort();
    models.dedup();

    if models.is_empty() {
        return Err(anyhow!("Ollama server reported no installed models"));
    }

    Ok(models)
}
