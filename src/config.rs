//! Configuration management.
//!
//! Handles loading, saving, and managing user configuration including
//! location preferences, AI settings, and time synchronization.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

fn default_true() -> bool {
    true
}

fn default_false() -> bool {
    false
}

fn default_time_sync_server() -> String {
    String::new() // Empty means use default servers
}

fn default_ai_server() -> String {
    "http://localhost:11434".to_string()
}

fn default_ai_model() -> String {
    "llama3.2:latest".to_string()
}

/// Location input mode for determining how coordinates are obtained.
///
/// Controls whether the user specifies a city name or manual coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum LocationMode {
    /// Use city database lookup (`--city "Boston"`)
    #[default]
    City,
    /// Manual latitude/longitude entry (`--lat 42.36 --lon -71.06`)
    Manual,
}

/// Time synchronization settings for NTP sync.
///
/// Controls whether and how the application synchronizes with network time servers.
/// Disabled sync uses only the system clock without validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TimeSyncSettings {
    /// Enable NTP time synchronization (default: true)
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// NTP server URL (empty = use default servers: time.google.com, pool.ntp.org)
    #[serde(default = "default_time_sync_server")]
    pub server: String,
}

impl Default for TimeSyncSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            server: default_time_sync_server(),
        }
    }
}

/// AI insights refresh mode for controlling when insights are updated.
///
/// Determines whether AI insights refresh automatically on a timer
/// or only when manually triggered by the user.
#[cfg(feature = "ai-insights")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AiRefreshMode {
    /// Refresh automatically at intervals AND allow manual refresh (default)
    #[default]
    #[serde(rename = "auto_and_manual")]
    AutoAndManual,
    /// Only refresh when manually requested via TUI hotkey
    #[serde(rename = "manual_only")]
    ManualOnly,
}

/// AI insights settings for Ollama integration.
///
/// Configures connection to a local Ollama LLM server for generating
/// natural language insights about astronomical data.
///
/// # Example Configuration
///
/// ```json
/// {
///   "enabled": true,
///   "server": "http://localhost:11434",
///   "model": "llama3.2:latest",
///   "refresh_minutes": 2,
///   "refresh_mode": "auto_and_manual"
/// }
/// ```
#[cfg(feature = "ai-insights")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AiSettings {
    /// Enable AI insights (default: false)
    #[serde(default = "default_false")]
    pub enabled: bool,
    /// Ollama server URL (default: "http://localhost:11434")
    #[serde(default = "default_ai_server")]
    pub server: String,
    /// Ollama model name (default: "llama3.2:latest")
    #[serde(default = "default_ai_model")]
    pub model: String,
    /// Auto-refresh interval in minutes (default: 2)
    #[serde(default)]
    pub refresh_minutes: u64,
    /// Refresh mode (default: auto and manual)
    #[serde(default)]
    pub refresh_mode: AiRefreshMode,
}

#[cfg(feature = "ai-insights")]
impl Default for AiSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            server: default_ai_server(),
            model: default_ai_model(),
            refresh_minutes: 2,
            refresh_mode: AiRefreshMode::AutoAndManual,
        }
    }
}

/// User preferences for watch mode (TUI) display.
///
/// Controls which sections are visible in the terminal user interface.
/// All sections are shown by default except AI insights and night mode.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct WatchPreferences {
    /// Show location and date section (default: true)
    #[serde(default = "default_true")]
    pub show_location_date: bool,
    /// Show events section (sunrise, sunset, moonrise, etc.) (default: true)
    #[serde(default = "default_true")]
    pub show_events: bool,
    /// Show sun/moon position section (alt/az) (default: true)
    #[serde(default = "default_true")]
    pub show_positions: bool,
    /// Show moon details section (phase, illumination) (default: true)
    #[serde(default = "default_true")]
    pub show_moon: bool,
    /// Show lunar phases section (default: true)
    #[serde(default = "default_true")]
    pub show_lunar_phases: bool,
    /// Show planets section (Mercury through Neptune) (default: true)
    #[serde(default = "default_true")]
    pub show_planets: bool,
    /// Show AI insights section (default: false)
    #[cfg(feature = "ai-insights")]
    #[serde(default = "default_false")]
    pub show_ai_insights: bool,
    /// Enable night mode (red text to preserve night vision) (default: false)
    #[serde(default = "default_false")]
    pub night_mode: bool,
}

impl Default for WatchPreferences {
    fn default() -> Self {
        Self {
            show_location_date: true,
            show_events: true,
            show_positions: true,
            show_moon: true,
            show_lunar_phases: true,
            show_planets: true,
            #[cfg(feature = "ai-insights")]
            show_ai_insights: false,
            night_mode: false,
        }
    }
}

/// Main application configuration.
///
/// Stored in `~/.solunatus.json` and automatically loaded on startup.
/// Contains location, display preferences, time sync settings, and optional AI configuration.
///
/// # Example Configuration File
///
/// ```json
/// {
///   "lat": 42.3834,
///   "lon": -71.4162,
///   "tz": "America/New_York",
///   "city": "Waltham",
///   "location_mode": "City",
///   "watch": {
///     "show_location_date": true,
///     "show_events": true,
///     "show_positions": true,
///     "show_moon": true,
///     "show_lunar_phases": true,
///     "night_mode": false
///   },
///   "time_sync": {
///     "enabled": true,
///     "server": ""
///   }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Latitude in decimal degrees (WGS84)
    pub lat: f64,
    /// Longitude in decimal degrees (WGS84)
    pub lon: f64,
    /// IANA timezone identifier (e.g., "America/New_York")
    pub tz: String,
    /// Optional city name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,
    /// Location input mode (city vs manual coordinates)
    #[serde(default)]
    pub location_mode: LocationMode,
    /// Watch mode display preferences
    #[serde(default)]
    pub watch: WatchPreferences,
    /// NTP time synchronization settings
    #[serde(default)]
    pub time_sync: TimeSyncSettings,
    /// AI insights settings (if feature enabled)
    #[cfg(feature = "ai-insights")]
    #[serde(default)]
    pub ai: AiSettings,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            lat: 0.0,
            lon: 0.0,
            tz: "UTC".to_string(),
            city: None,
            location_mode: LocationMode::City,
            watch: WatchPreferences::default(),
            time_sync: TimeSyncSettings::default(),
            #[cfg(feature = "ai-insights")]
            ai: AiSettings::default(),
        }
    }
}

impl Config {
    /// Creates a new configuration with the specified location.
    ///
    /// # Arguments
    ///
    /// * `lat` - Latitude in decimal degrees
    /// * `lon` - Longitude in decimal degrees
    /// * `tz` - IANA timezone identifier
    /// * `city` - Optional city name
    ///
    /// # Returns
    ///
    /// A new [`Config`] with default preferences and the specified location.
    pub fn new(lat: f64, lon: f64, tz: String, city: Option<String>) -> Self {
        Self {
            lat,
            lon,
            tz,
            city,
            location_mode: LocationMode::City,
            watch: WatchPreferences::default(),
            time_sync: TimeSyncSettings::default(),
            #[cfg(feature = "ai-insights")]
            ai: AiSettings::default(),
        }
    }

    /// Returns the path to the configuration file (`~/.solunatus.json`).
    ///
    /// # Errors
    ///
    /// Returns an error if the home directory cannot be determined.
    pub fn config_path() -> Result<PathBuf> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
        Ok(home_dir.join(".solunatus.json"))
    }

    /// Loads configuration from the config file.
    ///
    /// # Returns
    ///
    /// - `Ok(Some(config))` if file exists and is valid
    /// - `Ok(None)` if file doesn't exist
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Home directory cannot be determined
    /// - File exists but cannot be read
    /// - File contents are invalid JSON
    pub fn load() -> Result<Option<Self>> {
        let path = Self::config_path()?;

        if !path.exists() {
            return Ok(None);
        }

        let contents = fs::read_to_string(&path).context("Failed to read config file")?;

        let config: Self =
            serde_json::from_str(&contents).context("Failed to parse config file")?;

        Ok(Some(config))
    }

    /// Saves configuration to the config file.
    ///
    /// Creates or overwrites `~/.solunatus.json` with the current configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Home directory cannot be determined
    /// - File cannot be written (permissions, disk full, etc.)
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;

        let contents = serde_json::to_string_pretty(self).context("Failed to serialize config")?;

        fs::write(&path, contents).context("Failed to write config file")?;

        Ok(())
    }
}
