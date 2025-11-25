// Application state for TUI

#[cfg(feature = "ai-insights")]
use crate::ai;
use crate::astro::*;
use crate::calendar::CalendarFormat;
use crate::calendar_optimized;
use crate::city::City;
use crate::config::{self, WatchPreferences};
use crate::events;
use crate::location_source::LocationSource;
use crate::time_sync::TimeSyncInfo;
use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Datelike, Duration as ChronoDuration, Local, NaiveDate};
use chrono_tz::Tz;
use std::{
    fs,
    path::PathBuf,
    sync::mpsc::{self, Receiver},
    thread,
    time::{Duration, Instant},
};

const STATUS_TTL: Duration = Duration::from_secs(10);
const EVENT_WINDOW_HOURS: i64 = 12;
const EVENT_REFRESH_THRESHOLD_HOURS: i64 = 6;
const POSITION_REFRESH_INTERVAL: Duration = Duration::from_secs(5);
const MOON_REFRESH_INTERVAL: Duration = Duration::from_secs(600);
const TIME_SYNC_REFRESH_INTERVAL: Duration = Duration::from_secs(1800); // 30 minutes (pool.ntp.org ToS compliance)
// Allow a small buffer below the horizon before calling the Moon "Rising" so we
// do not report rising while it is still deep below the horizon.
#[derive(Debug, Clone)]
pub struct CachedEvents {
    pub reference: DateTime<Tz>,
    pub entries: Vec<(DateTime<Tz>, &'static str)>,
}

#[derive(Debug, Clone, Copy)]
pub struct CachedPositions {
    pub timestamp: DateTime<Tz>,
    pub sun: sun::SolarPosition,
    pub moon: moon::LunarPosition,
}

impl CachedPositions {
    fn new(location: &Location, timestamp: &DateTime<Tz>) -> Self {
        Self {
            timestamp: *timestamp,
            sun: sun::solar_position(location, timestamp),
            moon: moon::lunar_position(location, timestamp),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CachedMoonDetails {
    pub timestamp: DateTime<Tz>,
    pub moon: moon::LunarPosition,
    pub altitude_trend: MoonAltitudeTrend,
}

#[derive(Debug, Clone, Copy)]
/// Simple visibility indicator for the Moon.
pub enum MoonAltitudeTrend {
    /// Moon center altitude < 0° (not visible).
    Down,
    /// Moon center altitude ≥ 0° (visible).
    Up,
}

impl CachedMoonDetails {
    fn from_positions(location: &Location, positions: &CachedPositions) -> Self {
        let altitude_trend = determine_moon_trend(location, &positions.timestamp, positions.moon);
        Self {
            timestamp: positions.timestamp,
            moon: positions.moon,
            altitude_trend,
        }
    }
}

fn determine_moon_trend(
    _location: &Location,
    _timestamp: &DateTime<Tz>,
    base: moon::LunarPosition,
) -> MoonAltitudeTrend {
    if base.altitude >= 0.0 {
        MoonAltitudeTrend::Up
    } else {
        MoonAltitudeTrend::Down
    }
}

#[derive(Debug, Clone, Copy)]
pub enum AppMode {
    Watch,
    Settings,
    CityPicker,
    LocationInput,
    AiConfig,
    Calendar,
    Reports,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportsMenuItem {
    Calendar,
    UsnoValidation,
    Benchmark,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsField {
    LocationMode,
    TimeSyncEnabled,
    TimeSyncServer,
    ShowLocationDate,
    ShowEvents,
    ShowPositions,
    ShowMoon,
    ShowLunarPhases,
    NightMode,
    AiEnabled,
    AiServer,
    AiModel,
    AiRefreshMinutes,
}

#[cfg(feature = "ai-insights")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiConfigField {
    Enabled,
    Server,
    Model,
    RefreshMinutes,
    RefreshMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocationInputField {
    Latitude,
    Longitude,
    Timezone,
}

#[derive(Debug, Clone)]
pub struct LocationInputDraft {
    pub latitude: String,
    pub longitude: String,
    pub timezone: String,
    pub field_index: usize,
    pub error: Option<String>,
}

impl Default for LocationInputDraft {
    fn default() -> Self {
        Self::new()
    }
}

impl LocationInputDraft {
    const FIELD_COUNT: usize = 3;

    pub fn new() -> Self {
        Self {
            latitude: String::new(),
            longitude: String::new(),
            timezone: "UTC".to_string(),
            field_index: 0,
            error: None,
        }
    }

    pub fn current_field(&self) -> LocationInputField {
        match self.field_index {
            0 => LocationInputField::Latitude,
            1 => LocationInputField::Longitude,
            _ => LocationInputField::Timezone,
        }
    }

    pub fn next_field(&mut self) {
        self.field_index = (self.field_index + 1) % Self::FIELD_COUNT;
        self.clear_error();
    }

    pub fn prev_field(&mut self) {
        self.field_index = (self.field_index + Self::FIELD_COUNT - 1) % Self::FIELD_COUNT;
        self.clear_error();
    }

    pub fn input_char(&mut self, c: char) {
        self.clear_error();
        match self.current_field() {
            LocationInputField::Latitude | LocationInputField::Longitude => {
                // Allow digits, minus sign, and decimal point
                if c.is_ascii_digit() || c == '-' || c == '.' {
                    let field = if self.current_field() == LocationInputField::Latitude {
                        &mut self.latitude
                    } else {
                        &mut self.longitude
                    };
                    field.push(c);
                }
            }
            LocationInputField::Timezone => {
                self.timezone.push(c);
            }
        }
    }

    pub fn backspace(&mut self) {
        self.clear_error();
        match self.current_field() {
            LocationInputField::Latitude => {
                self.latitude.pop();
            }
            LocationInputField::Longitude => {
                self.longitude.pop();
            }
            LocationInputField::Timezone => {
                self.timezone.pop();
            }
        }
    }

    pub fn clear_error(&mut self) {
        self.error = None;
    }

    pub fn set_error(&mut self, msg: String) {
        self.error = Some(msg);
    }

    pub fn validate(&self) -> Result<(f64, f64, String)> {
        // Parse latitude
        let lat = self
            .latitude
            .trim()
            .parse::<f64>()
            .map_err(|_| anyhow!("Invalid latitude"))?;

        if !(-90.0..=90.0).contains(&lat) {
            return Err(anyhow!("Latitude must be between -90 and 90"));
        }

        // Parse longitude
        let lon = self
            .longitude
            .trim()
            .parse::<f64>()
            .map_err(|_| anyhow!("Invalid longitude"))?;

        if !(-180.0..=180.0).contains(&lon) {
            return Err(anyhow!("Longitude must be between -180 and 180"));
        }

        // Validate timezone
        let tz = self.timezone.trim().to_string();
        if tz.is_empty() {
            return Err(anyhow!("Timezone cannot be empty"));
        }

        Ok((lat, lon, tz))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalendarField {
    StartDate,
    EndDate,
    Format,
    OutputPath,
}

#[derive(Debug, Clone)]
pub struct CalendarDraft {
    pub start: String,
    pub end: String,
    pub output_path: String,
    pub field_index: usize,
    pub format_index: usize,
    pub error: Option<String>,
}

impl CalendarDraft {
    const FIELD_COUNT: usize = 4;
    const FORMATS: [CalendarFormat; 2] = [CalendarFormat::Html, CalendarFormat::Json];

    pub fn new(now: DateTime<Local>) -> Self {
        let today = now.date_naive();
        let start = NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap_or(today);
        let (next_year, next_month) = if today.month() == 12 {
            (today.year() + 1, 1)
        } else {
            (today.year(), today.month() + 1)
        };
        let next_month_start = NaiveDate::from_ymd_opt(next_year, next_month, 1).unwrap_or(today);
        let end = next_month_start.pred_opt().unwrap_or(next_month_start);

        Self {
            start: start.format("%Y-%m-%d").to_string(),
            end: end.format("%Y-%m-%d").to_string(),
            output_path: Self::default_output_filename(CalendarFormat::Html, start, end),
            field_index: 0,
            format_index: 0,
            error: None,
        }
    }

    pub fn reset(&mut self, now: DateTime<Local>) {
        *self = Self::new(now);
    }

    pub fn current_field(&self) -> CalendarField {
        match self.field_index {
            0 => CalendarField::StartDate,
            1 => CalendarField::EndDate,
            2 => CalendarField::Format,
            _ => CalendarField::OutputPath,
        }
    }

    pub fn next_field(&mut self) {
        self.field_index = (self.field_index + 1) % Self::FIELD_COUNT;
        self.clear_error();
    }

    pub fn prev_field(&mut self) {
        self.field_index = (self.field_index + Self::FIELD_COUNT - 1) % Self::FIELD_COUNT;
        self.clear_error();
    }

    pub fn current_format(&self) -> CalendarFormat {
        Self::FORMATS[self.format_index]
    }

    pub fn current_format_label(&self) -> &'static str {
        match self.current_format() {
            CalendarFormat::Html => "HTML",
            CalendarFormat::Json => "JSON",
        }
    }

    pub fn cycle_format(&mut self, delta: isize) {
        let len = Self::FORMATS.len() as isize;
        let mut next = self.format_index as isize + delta;
        if next < 0 {
            next = (next % len + len) % len;
        } else {
            next %= len;
        }
        self.format_index = next as usize;
        self.sync_output_extension();
        self.clear_error();
    }

    pub fn set_format(&mut self, format: CalendarFormat) {
        if let Some(idx) = Self::FORMATS
            .iter()
            .position(|candidate| *candidate == format)
        {
            self.format_index = idx;
            self.sync_output_extension();
            self.clear_error();
        }
    }

    pub fn input_char(&mut self, c: char) {
        self.clear_error();
        match self.current_field() {
            CalendarField::StartDate => {
                if c.is_ascii_digit() || c == '-' {
                    self.start.push(c);
                }
            }
            CalendarField::EndDate => {
                if c.is_ascii_digit() || c == '-' {
                    self.end.push(c);
                }
            }
            CalendarField::Format => {}
            CalendarField::OutputPath => {
                self.output_path.push(c);
            }
        }
    }

    pub fn backspace(&mut self) {
        self.clear_error();
        match self.current_field() {
            CalendarField::StartDate => {
                self.start.pop();
            }
            CalendarField::EndDate => {
                self.end.pop();
            }
            CalendarField::Format => {}
            CalendarField::OutputPath => {
                self.output_path.pop();
            }
        }
    }

    pub fn clear_error(&mut self) {
        self.error = None;
    }

    pub fn set_error<S: Into<String>>(&mut self, msg: S) {
        self.error = Some(msg.into());
    }

    pub fn validate(&self) -> Result<(NaiveDate, NaiveDate, CalendarFormat, String)> {
        let start_str = self.start.trim();
        if start_str.is_empty() {
            return Err(anyhow!("Start date is required"));
        }
        let end_str = self.end.trim();
        if end_str.is_empty() {
            return Err(anyhow!("End date is required"));
        }

        let start = NaiveDate::parse_from_str(start_str, "%Y-%m-%d")
            .with_context(|| format!("Invalid start date '{}'", start_str))?;
        let end = NaiveDate::parse_from_str(end_str, "%Y-%m-%d")
            .with_context(|| format!("Invalid end date '{}'", end_str))?;

        if start > end {
            return Err(anyhow!("Start date must be on or before the end date"));
        }

        let format = self.current_format();

        let output_trim = self.output_path.trim();
        let output = if output_trim.is_empty() {
            Self::default_output_filename(format, start, end)
        } else {
            output_trim.to_string()
        };

        Ok((start, end, format, output))
    }

    fn sync_output_extension(&mut self) {
        let extension = Self::format_extension(self.current_format());
        if self.output_path.trim().is_empty() {
            if let (Ok(start), Ok(end)) = (
                NaiveDate::parse_from_str(self.start.trim(), "%Y-%m-%d"),
                NaiveDate::parse_from_str(self.end.trim(), "%Y-%m-%d"),
            ) {
                self.output_path = Self::default_output_filename(self.current_format(), start, end);
            }
            return;
        }

        let mut path = PathBuf::from(self.output_path.trim());
        path.set_extension(extension);
        self.output_path = path.to_string_lossy().to_string();
    }

    fn default_output_filename(format: CalendarFormat, start: NaiveDate, end: NaiveDate) -> String {
        format!(
            "solunatus-calendar-{}-{}.{}",
            start.format("%Y%m%d"),
            end.format("%Y%m%d"),
            Self::format_extension(format)
        )
    }

    fn format_extension(format: CalendarFormat) -> &'static str {
        match format {
            CalendarFormat::Html => "html",
            CalendarFormat::Json => "json",
        }
    }
}

#[cfg(feature = "ai-insights")]
#[derive(Debug, Clone)]
pub enum AiServerStatus {
    Unknown,
    Connected { server: String },
    Failed { server: String, message: String },
}

#[cfg(feature = "ai-insights")]
#[derive(Debug, Clone)]
pub struct AiConfigDraft {
    pub enabled: bool,
    pub server: String,
    pub model: String,
    pub refresh_minutes: String,
    pub refresh_mode: config::AiRefreshMode,
    pub field_index: usize,
    pub error: Option<String>,
    pub server_status: AiServerStatus,
    pub models: Vec<String>,
    pub model_index: Option<usize>,
}

#[cfg(feature = "ai-insights")]
impl AiConfigDraft {
    const FIELD_COUNT: usize = 5;

    pub fn from_config(config: &ai::AiConfig) -> Self {
        Self {
            enabled: config.enabled,
            server: config.server.clone(),
            model: config.model.clone(),
            refresh_minutes: config.refresh_minutes().to_string(),
            refresh_mode: config.refresh_mode,
            field_index: 0,
            error: None,
            server_status: AiServerStatus::Unknown,
            models: Vec::new(),
            model_index: None,
        }
    }

    pub fn sync_from(&mut self, config: &ai::AiConfig) {
        self.enabled = config.enabled;
        self.server = config.server.clone();
        self.model = config.model.clone();
        self.refresh_minutes = config.refresh_minutes().to_string();
        self.refresh_mode = config.refresh_mode;
        self.field_index = 0;
        self.error = None;
        self.reset_detection();
    }

    pub fn current_field(&self) -> AiConfigField {
        match self.field_index {
            0 => AiConfigField::Enabled,
            1 => AiConfigField::Server,
            2 => AiConfigField::Model,
            3 => AiConfigField::RefreshMinutes,
            _ => AiConfigField::RefreshMode,
        }
    }

    pub fn next_field(&mut self) {
        self.field_index = (self.field_index + 1) % Self::FIELD_COUNT;
        self.clear_error();
    }

    pub fn prev_field(&mut self) {
        self.field_index = (self.field_index + Self::FIELD_COUNT - 1) % Self::FIELD_COUNT;
        self.clear_error();
    }

    pub fn toggle_enabled(&mut self) {
        self.enabled = !self.enabled;
        self.clear_error();
    }

    pub fn input_char(&mut self, c: char) {
        self.clear_error();
        match self.current_field() {
            AiConfigField::Enabled => {}
            AiConfigField::Server => {
                self.server.push(c);
                self.mark_server_dirty();
            }
            AiConfigField::Model => {
                self.model.push(c);
                self.model_index = None;
            }
            AiConfigField::RefreshMinutes => {
                if c.is_ascii_digit() && self.refresh_minutes.len() < 2 {
                    self.refresh_minutes.push(c);
                }
            }
            AiConfigField::RefreshMode => {}
        }
    }

    pub fn backspace(&mut self) {
        self.clear_error();
        match self.current_field() {
            AiConfigField::Enabled => {}
            AiConfigField::Server => {
                self.server.pop();
                self.mark_server_dirty();
            }
            AiConfigField::Model => {
                self.model.pop();
                self.model_index = None;
            }
            AiConfigField::RefreshMinutes => {
                self.refresh_minutes.pop();
            }
            AiConfigField::RefreshMode => {}
        }
    }

    pub fn bump_refresh(&mut self, delta: i64) {
        if self.current_field() != AiConfigField::RefreshMinutes {
            return;
        }

        let mut value = self.refresh_minutes.trim().parse::<i64>().unwrap_or(2);
        value += delta;
        value = value.clamp(1, 60);
        self.refresh_minutes = value.to_string();
        self.clear_error();
    }

    pub fn toggle_refresh_mode(&mut self) {
        if self.current_field() != AiConfigField::RefreshMode {
            return;
        }

        self.refresh_mode = match self.refresh_mode {
            config::AiRefreshMode::AutoAndManual => config::AiRefreshMode::ManualOnly,
            config::AiRefreshMode::ManualOnly => config::AiRefreshMode::AutoAndManual,
        };
        self.clear_error();
    }

    pub fn clear_error(&mut self) {
        self.error = None;
    }

    pub fn set_error<S: Into<String>>(&mut self, msg: S) {
        self.error = Some(msg.into());
    }

    pub fn reset_detection(&mut self) {
        self.server_status = AiServerStatus::Unknown;
        self.models.clear();
        self.model_index = None;
    }

    pub fn mark_server_dirty(&mut self) {
        self.reset_detection();
    }

    pub fn set_detection_success(&mut self, server: String, mut models: Vec<String>) {
        self.server_status = AiServerStatus::Connected {
            server: server.clone(),
        };
        self.server = server;
        models.sort();
        models.dedup();
        self.models = models;
        self.clear_error();

        if self.models.is_empty() {
            self.model_index = None;
            return;
        }

        if let Some(idx) = self.models.iter().position(|name| name == &self.model) {
            self.model_index = Some(idx);
            self.model = self.models[idx].clone();
        } else {
            let idx = 0;
            self.model_index = Some(idx);
            self.model = self.models[idx].clone();
        }
    }

    pub fn set_detection_failure(&mut self, server: String, message: String) {
        self.server_status = AiServerStatus::Failed {
            server: server.clone(),
            message,
        };
        self.server = server;
        self.model_index = None;
        self.models.clear();
    }

    pub fn cycle_model(&mut self, delta: isize) {
        if self.models.is_empty() {
            return;
        }

        let len = self.models.len() as isize;
        let current = self.model_index.unwrap_or(0) as isize;
        let mut next = current + delta;
        if next < 0 {
            next = (next % len + len) % len;
        } else {
            next %= len;
        }

        self.model_index = Some(next as usize);
        if let Some(model) = self.models.get(next as usize) {
            self.model = model.clone();
        }
        self.clear_error();
    }
}

#[derive(Debug, Clone)]
pub struct SettingsDraft {
    pub location_mode: config::LocationMode,
    pub time_sync_enabled: bool,
    pub time_sync_server: String,
    pub show_location_date: bool,
    pub show_events: bool,
    pub show_positions: bool,
    pub show_moon: bool,
    pub show_lunar_phases: bool,
    pub night_mode: bool,
    #[cfg(feature = "ai-insights")]
    pub ai_enabled: bool,
    #[cfg(feature = "ai-insights")]
    pub ai_server: String,
    #[cfg(feature = "ai-insights")]
    pub ai_model: String,
    #[cfg(feature = "ai-insights")]
    pub ai_refresh_minutes: String,
    pub field_index: usize,
    pub error: Option<String>,
    #[cfg(feature = "ai-insights")]
    pub ai_server_status: AiServerStatus,
    #[cfg(feature = "ai-insights")]
    pub ai_models: Vec<String>,
    #[cfg(feature = "ai-insights")]
    pub ai_model_index: Option<usize>,
}

impl SettingsDraft {
    #[cfg(feature = "ai-insights")]
    const FIELD_COUNT: usize = 13;
    #[cfg(not(feature = "ai-insights"))]
    const FIELD_COUNT: usize = 9;

    pub fn from_app(app: &App) -> Self {
        Self {
            location_mode: config::LocationMode::City, // Will be loaded from config
            time_sync_enabled: !app.time_sync_disabled,
            time_sync_server: "time.google.com".to_string(), // Will be loaded from config
            show_location_date: app.show_location_date,
            show_events: app.show_events,
            show_positions: app.show_positions,
            show_moon: app.show_moon,
            show_lunar_phases: app.show_lunar_phases,
            night_mode: app.night_mode,
            #[cfg(feature = "ai-insights")]
            ai_enabled: app.ai_config.enabled,
            #[cfg(feature = "ai-insights")]
            ai_server: app.ai_config.server.clone(),
            #[cfg(feature = "ai-insights")]
            ai_model: app.ai_config.model.clone(),
            #[cfg(feature = "ai-insights")]
            ai_refresh_minutes: app.ai_config.refresh_minutes().to_string(),
            field_index: 0,
            error: None,
            #[cfg(feature = "ai-insights")]
            ai_server_status: AiServerStatus::Unknown,
            #[cfg(feature = "ai-insights")]
            ai_models: Vec::new(),
            #[cfg(feature = "ai-insights")]
            ai_model_index: None,
        }
    }

    pub fn current_field(&self) -> SettingsField {
        match self.field_index {
            0 => SettingsField::LocationMode,
            1 => SettingsField::TimeSyncEnabled,
            2 => SettingsField::TimeSyncServer,
            3 => SettingsField::ShowLocationDate,
            4 => SettingsField::ShowEvents,
            5 => SettingsField::ShowPositions,
            6 => SettingsField::ShowMoon,
            7 => SettingsField::ShowLunarPhases,
            8 => SettingsField::NightMode,
            #[cfg(feature = "ai-insights")]
            9 => SettingsField::AiEnabled,
            #[cfg(feature = "ai-insights")]
            10 => SettingsField::AiServer,
            #[cfg(feature = "ai-insights")]
            11 => SettingsField::AiModel,
            #[cfg(feature = "ai-insights")]
            _ => SettingsField::AiRefreshMinutes,
            #[cfg(not(feature = "ai-insights"))]
            _ => SettingsField::NightMode,
        }
    }

    pub fn next_field(&mut self) {
        self.field_index = (self.field_index + 1) % Self::FIELD_COUNT;
        self.clear_error();
    }

    pub fn prev_field(&mut self) {
        self.field_index = (self.field_index + Self::FIELD_COUNT - 1) % Self::FIELD_COUNT;
        self.clear_error();
    }

    pub fn toggle_current_bool(&mut self) {
        match self.current_field() {
            SettingsField::TimeSyncEnabled => self.time_sync_enabled = !self.time_sync_enabled,
            SettingsField::ShowLocationDate => self.show_location_date = !self.show_location_date,
            SettingsField::ShowEvents => self.show_events = !self.show_events,
            SettingsField::ShowPositions => self.show_positions = !self.show_positions,
            SettingsField::ShowMoon => self.show_moon = !self.show_moon,
            SettingsField::ShowLunarPhases => self.show_lunar_phases = !self.show_lunar_phases,
            SettingsField::NightMode => self.night_mode = !self.night_mode,
            #[cfg(feature = "ai-insights")]
            SettingsField::AiEnabled => self.ai_enabled = !self.ai_enabled,
            _ => {}
        }
        self.clear_error();
    }

    pub fn cycle_location_mode(&mut self) {
        self.location_mode = match self.location_mode {
            config::LocationMode::City => config::LocationMode::Manual,
            config::LocationMode::Manual => config::LocationMode::City,
        };
        self.clear_error();
    }

    pub fn input_char(&mut self, c: char) {
        self.clear_error();
        match self.current_field() {
            SettingsField::TimeSyncServer => {
                self.time_sync_server.push(c);
            }
            #[cfg(feature = "ai-insights")]
            SettingsField::AiServer => {
                self.ai_server.push(c);
            }
            #[cfg(feature = "ai-insights")]
            SettingsField::AiModel => {
                self.ai_model.push(c);
            }
            #[cfg(feature = "ai-insights")]
            SettingsField::AiRefreshMinutes => {
                if c.is_ascii_digit() && self.ai_refresh_minutes.len() < 2 {
                    self.ai_refresh_minutes.push(c);
                }
            }
            _ => {}
        }
    }

    pub fn backspace(&mut self) {
        self.clear_error();
        match self.current_field() {
            SettingsField::TimeSyncServer => {
                self.time_sync_server.pop();
            }
            #[cfg(feature = "ai-insights")]
            SettingsField::AiServer => {
                self.ai_server.pop();
            }
            #[cfg(feature = "ai-insights")]
            SettingsField::AiModel => {
                self.ai_model.pop();
            }
            #[cfg(feature = "ai-insights")]
            SettingsField::AiRefreshMinutes => {
                self.ai_refresh_minutes.pop();
            }
            _ => {}
        }
    }

    pub fn clear_error(&mut self) {
        self.error = None;
    }

    pub fn set_error<S: Into<String>>(&mut self, msg: S) {
        self.error = Some(msg.into());
    }
}

pub struct App {
    pub location: Location,
    pub timezone: Tz,
    pub city_name: Option<String>,
    pub nearest_city_info: Option<(String, f64, f64)>, // (city_name, distance_km, bearing_deg)
    pub location_source: LocationSource,
    pub current_time: DateTime<Local>,
    pub night_mode: bool,
    pub mode: AppMode,
    pub should_quit: bool,
    pub should_save: bool,
    pub city_search: String,
    pub city_results: Vec<City>,
    pub city_selected: usize,
    pub location_input_draft: LocationInputDraft,
    pub calendar_draft: CalendarDraft,
    pub settings_draft: SettingsDraft,
    pub location_mode: config::LocationMode,
    pub reports_selected_item: ReportsMenuItem,
    pub time_sync: TimeSyncInfo,
    pub time_sync_server: String,
    #[cfg(feature = "ai-insights")]
    pub ai_config: ai::AiConfig,
    #[cfg(feature = "ai-insights")]
    pub ai_outcome: Option<ai::AiOutcome>,
    #[cfg(feature = "ai-insights")]
    pub ai_last_refresh: Option<Instant>,
    #[cfg(feature = "ai-insights")]
    pub ai_config_draft: AiConfigDraft,
    pub status_message: Option<String>,
    pub status_timestamp: Option<Instant>,
    pub events_cache: CachedEvents,
    pub positions_cache: CachedPositions,
    pub positions_last_refresh: Instant,
    pub moon_overview_cache: CachedMoonDetails,
    pub moon_overview_last_refresh: Instant,
    pub lunar_phases_cache: Vec<moon::LunarPhase>,
    pub lunar_phases_generated_for: NaiveDate,
    pub show_location_date: bool,
    pub show_events: bool,
    pub show_positions: bool,
    pub show_moon: bool,
    pub show_lunar_phases: bool,
    #[cfg(feature = "ai-insights")]
    pub show_ai_insights: bool,
    pub time_sync_last_check: Instant,
    pub time_sync_disabled: bool,
    #[cfg(feature = "ai-insights")]
    ai_job_rx: Option<Receiver<Result<ai::AiOutcome, String>>>,
    #[cfg(feature = "ai-insights")]
    ai_job_prev_outcome: Option<ai::AiOutcome>,
}

/// Initial configuration for creating an App instance
pub struct AppConfig {
    pub location: Location,
    pub timezone: Tz,
    pub city_name: Option<String>,
    pub location_source: LocationSource,
    pub location_mode: config::LocationMode,
    pub time_sync: TimeSyncInfo,
    pub time_sync_disabled: bool,
    pub time_sync_server: String,
    #[cfg(feature = "ai-insights")]
    pub ai_config: ai::AiConfig,
    pub watch_prefs: Option<WatchPreferences>,
}

impl App {
    pub fn new(config: AppConfig) -> Self {
        let location = config.location;
        let timezone = config.timezone;
        let city_name = config.city_name;
        let location_source = config.location_source;
        let location_mode = config.location_mode;
        let time_sync = config.time_sync;
        let time_sync_disabled = config.time_sync_disabled;
        let time_sync_server = config.time_sync_server;
        #[cfg(feature = "ai-insights")]
        let ai_config = config.ai_config;
        let watch_prefs = config.watch_prefs;
        let now = Local::now();
        let now_tz = now.with_timezone(&timezone);
        let events_entries = events::collect_events_within_window(
            &location,
            &now_tz,
            ChronoDuration::hours(EVENT_WINDOW_HOURS),
        );
        let positions_cache = CachedPositions::new(&location, &now_tz);
        let moon_overview_cache = CachedMoonDetails::from_positions(&location, &positions_cache);
        let lunar_phases_cache = Self::collect_lunar_phases(&now_tz);
        let lunar_phases_generated_for = now_tz.date_naive();
        let prefs = watch_prefs.unwrap_or_default();

        // Calculate nearest city info (only if no city_name is set, i.e., not using city picker)
        let nearest_city_info = if city_name.is_none() {
            if let Ok(db) = crate::city::CityDatabase::load() {
                if let Some((city, distance, bearing)) =
                    db.find_nearest(location.latitude.value(), location.longitude.value())
                {
                    let city_display = if let Some(ref state) = city.state {
                        format!("{},{}", city.name, state)
                    } else {
                        city.name.clone()
                    };
                    Some((city_display, distance, bearing))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        Self {
            location,
            timezone,
            city_name,
            nearest_city_info,
            location_source,
            current_time: now,
            night_mode: prefs.night_mode,
            mode: AppMode::Watch,
            should_quit: false,
            should_save: false,
            city_search: String::new(),
            city_results: Vec::new(),
            city_selected: 0,
            location_input_draft: LocationInputDraft::new(),
            calendar_draft: CalendarDraft::new(now),
            settings_draft: SettingsDraft {
                location_mode: config::LocationMode::City,
                time_sync_enabled: !time_sync_disabled,
                time_sync_server: time_sync_server.clone(),
                show_location_date: prefs.show_location_date,
                show_events: prefs.show_events,
                show_positions: prefs.show_positions,
                show_moon: prefs.show_moon,
                show_lunar_phases: prefs.show_lunar_phases,
                night_mode: prefs.night_mode,
                #[cfg(feature = "ai-insights")]
                ai_enabled: ai_config.enabled,
                #[cfg(feature = "ai-insights")]
                ai_server: ai_config.server.clone(),
                #[cfg(feature = "ai-insights")]
                ai_model: ai_config.model.clone(),
                #[cfg(feature = "ai-insights")]
                ai_refresh_minutes: ai_config.refresh_minutes().to_string(),
                field_index: 0,
                error: None,
                #[cfg(feature = "ai-insights")]
                ai_server_status: AiServerStatus::Unknown,
                #[cfg(feature = "ai-insights")]
                ai_models: Vec::new(),
                #[cfg(feature = "ai-insights")]
                ai_model_index: None,
            },
            location_mode,
            reports_selected_item: ReportsMenuItem::Calendar,
            time_sync,
            time_sync_server,
            #[cfg(feature = "ai-insights")]
            ai_config_draft: AiConfigDraft::from_config(&ai_config),
            #[cfg(feature = "ai-insights")]
            ai_config,
            #[cfg(feature = "ai-insights")]
            ai_outcome: None,
            #[cfg(feature = "ai-insights")]
            ai_last_refresh: None,
            status_message: None,
            status_timestamp: None,
            events_cache: CachedEvents {
                reference: now_tz,
                entries: events_entries,
            },
            positions_cache,
            positions_last_refresh: Instant::now(),
            moon_overview_cache,
            moon_overview_last_refresh: Instant::now(),
            lunar_phases_cache,
            lunar_phases_generated_for,
            show_location_date: prefs.show_location_date,
            show_events: prefs.show_events,
            show_positions: prefs.show_positions,
            show_moon: prefs.show_moon,
            show_lunar_phases: prefs.show_lunar_phases,
            #[cfg(feature = "ai-insights")]
            show_ai_insights: prefs.show_ai_insights,
            time_sync_last_check: Instant::now(),
            time_sync_disabled,
            #[cfg(feature = "ai-insights")]
            ai_job_rx: None,
            #[cfg(feature = "ai-insights")]
            ai_job_prev_outcome: None,
        }
    }

    pub fn update_time(&mut self) {
        self.current_time = Local::now();
        self.expire_status_if_needed();
    }

    fn collect_lunar_phases(now_tz: &DateTime<Tz>) -> Vec<moon::LunarPhase> {
        use chrono::Datelike;

        let year = now_tz.year();
        let month = now_tz.month();

        let (prev_year, prev_month) = if month == 1 {
            (year - 1, 12)
        } else {
            (year, month - 1)
        };

        let (next_year, next_month) = if month == 12 {
            (year + 1, 1)
        } else {
            (year, month + 1)
        };

        let mut phases = Vec::new();
        phases.extend(moon::lunar_phases(prev_year, prev_month));
        phases.extend(moon::lunar_phases(year, month));
        phases.extend(moon::lunar_phases(next_year, next_month));
        phases.sort_by(|a, b| a.datetime.cmp(&b.datetime));
        phases.dedup_by(|a, b| a.datetime == b.datetime && a.phase_type == b.phase_type);
        phases
    }

    fn regenerate_events(&mut self) {
        let now_tz = self.current_time.with_timezone(&self.timezone);
        self.events_cache = CachedEvents {
            reference: now_tz,
            entries: events::collect_events_within_window(
                &self.location,
                &now_tz,
                ChronoDuration::hours(EVENT_WINDOW_HOURS),
            ),
        };
    }

    pub fn refresh_events_if_needed(&mut self) {
        let now_tz = self.current_time.with_timezone(&self.timezone);
        let reference = self.events_cache.reference;
        let threshold = ChronoDuration::hours(EVENT_REFRESH_THRESHOLD_HOURS);
        let delta = now_tz.signed_duration_since(reference);
        let need_refresh = self.events_cache.entries.is_empty()
            || delta.num_seconds().abs() >= threshold.num_seconds()
            || reference.date_naive() != now_tz.date_naive();

        if need_refresh {
            self.regenerate_events();
        }
    }

    fn recompute_positions(&mut self) {
        let now_tz = self.current_time.with_timezone(&self.timezone);
        self.positions_cache = CachedPositions::new(&self.location, &now_tz);
        self.positions_last_refresh = Instant::now();
    }

    pub fn refresh_positions_if_needed(&mut self) {
        if self.positions_last_refresh.elapsed() >= POSITION_REFRESH_INTERVAL {
            self.recompute_positions();
        }
    }

    pub fn refresh_moon_overview_if_needed(&mut self) {
        let now_tz = self.current_time.with_timezone(&self.timezone);
        let needs_update = self.moon_overview_last_refresh.elapsed() >= MOON_REFRESH_INTERVAL
            || self.moon_overview_cache.timestamp.date_naive() != now_tz.date_naive();

        if needs_update {
            if self.positions_last_refresh.elapsed() >= POSITION_REFRESH_INTERVAL {
                self.recompute_positions();
            }
            self.moon_overview_cache =
                CachedMoonDetails::from_positions(&self.location, &self.positions_cache);
            self.moon_overview_last_refresh = Instant::now();
        }
    }

    pub fn refresh_lunar_phases_if_needed(&mut self) {
        let now_tz = self.current_time.with_timezone(&self.timezone);
        if self.lunar_phases_cache.is_empty()
            || self.lunar_phases_generated_for != now_tz.date_naive()
        {
            self.lunar_phases_cache = Self::collect_lunar_phases(&now_tz);
            self.lunar_phases_generated_for = now_tz.date_naive();
        }
    }

    pub fn refresh_scheduled_data(&mut self) {
        #[cfg(feature = "ai-insights")]
        self.poll_ai_job();
        self.refresh_time_sync_if_needed();
        self.refresh_events_if_needed();
        self.refresh_positions_if_needed();
        self.refresh_moon_overview_if_needed();
        self.refresh_lunar_phases_if_needed();
    }

    pub fn reset_cached_data(&mut self) {
        self.regenerate_events();
        self.recompute_positions();
        self.moon_overview_cache =
            CachedMoonDetails::from_positions(&self.location, &self.positions_cache);
        self.moon_overview_last_refresh = Instant::now();
        let now_tz = self.current_time.with_timezone(&self.timezone);
        self.lunar_phases_cache = Self::collect_lunar_phases(&now_tz);
        self.lunar_phases_generated_for = now_tz.date_naive();
    }

    pub fn watch_preferences(&self) -> WatchPreferences {
        WatchPreferences {
            show_location_date: self.show_location_date,
            show_events: self.show_events,
            show_positions: self.show_positions,
            show_moon: self.show_moon,
            show_lunar_phases: self.show_lunar_phases,
            #[cfg(feature = "ai-insights")]
            show_ai_insights: self.show_ai_insights,
            night_mode: self.night_mode,
        }
    }

    pub fn build_config(&self) -> config::Config {
        let mut cfg = config::Config::new(
            self.location.latitude.value(),
            self.location.longitude.value(),
            self.timezone.name().to_string(),
            self.city_name.clone(),
        );
        cfg.location_mode = self.location_mode;
        cfg.watch = self.watch_preferences();
        cfg.time_sync = config::TimeSyncSettings {
            enabled: !self.time_sync_disabled,
            server: self.time_sync_server.clone(),
        };
        #[cfg(feature = "ai-insights")]
        {
            cfg.ai = config::AiSettings {
                enabled: self.ai_config.enabled,
                server: self.ai_config.server.clone(),
                model: self.ai_config.model.clone(),
                refresh_minutes: self.ai_config.refresh_minutes(),
                refresh_mode: self.ai_config.refresh_mode,
            };
        }
        cfg
    }

    fn expire_status_if_needed(&mut self) {
        if let Some(timestamp) = self.status_timestamp {
            if timestamp.elapsed() >= STATUS_TTL {
                self.status_message = None;
                self.status_timestamp = None;
            }
        }
    }

    pub fn set_status_message<S: Into<String>>(&mut self, message: S) {
        self.status_message = Some(message.into());
        self.status_timestamp = Some(Instant::now());
    }

    pub fn current_status(&self) -> Option<&str> {
        self.status_message.as_deref()
    }

    pub fn refresh_time_sync_if_needed(&mut self) {
        if self.time_sync_disabled {
            return;
        }
        if self.time_sync_last_check.elapsed() >= TIME_SYNC_REFRESH_INTERVAL {
            let server = if self.time_sync_server.trim().is_empty() {
                None
            } else {
                Some(self.time_sync_server.as_str())
            };
            self.time_sync = crate::time_sync::check_time_sync_with_servers(server);
            self.time_sync_last_check = Instant::now();
        }
    }

    pub fn time_sync_countdown(&self) -> Option<Duration> {
        if self.time_sync_disabled {
            return None;
        }
        let elapsed = self.time_sync_last_check.elapsed();
        let remaining = TIME_SYNC_REFRESH_INTERVAL
            .checked_sub(elapsed)
            .unwrap_or_else(|| Duration::from_secs(0));
        Some(remaining)
    }

    pub fn position_countdown(&self) -> Duration {
        let elapsed = self.positions_last_refresh.elapsed();
        POSITION_REFRESH_INTERVAL
            .checked_sub(elapsed)
            .unwrap_or_else(|| Duration::from_secs(0))
    }

    pub fn moon_countdown(&self) -> Duration {
        let elapsed = self.moon_overview_last_refresh.elapsed();
        MOON_REFRESH_INTERVAL
            .checked_sub(elapsed)
            .unwrap_or_else(|| Duration::from_secs(0))
    }

    pub fn toggle_night_mode(&mut self) {
        self.night_mode = !self.night_mode;
        self.should_save = true;
    }

    pub fn toggle_location_date(&mut self) {
        self.show_location_date = !self.show_location_date;
        self.should_save = true;
    }

    pub fn toggle_events(&mut self) {
        self.show_events = !self.show_events;
        self.should_save = true;
    }

    pub fn toggle_positions(&mut self) {
        self.show_positions = !self.show_positions;
        self.should_save = true;
    }

    pub fn toggle_moon(&mut self) {
        self.show_moon = !self.show_moon;
        self.should_save = true;
    }

    pub fn toggle_lunar_phases(&mut self) {
        self.show_lunar_phases = !self.show_lunar_phases;
        self.should_save = true;
    }

    pub fn open_calendar_generator(&mut self) {
        self.calendar_draft.reset(self.current_time);
        self.calendar_draft.clear_error();
        self.mode = AppMode::Calendar;
    }

    pub fn apply_calendar_generation(&mut self) -> Result<String> {
        let (start, end, format, output_path) = self.calendar_draft.validate()?;

        // Convert CalendarFormat to optimized module format
        let opt_format = match format {
            CalendarFormat::Html => calendar_optimized::CalendarFormat::Html,
            CalendarFormat::Json => calendar_optimized::CalendarFormat::Json,
        };

        // Use optimized calendar generation (70.81x faster for 75-year ranges!)
        let contents = calendar_optimized::generate_calendar_optimized(
            &self.location,
            &self.timezone,
            self.city_name.as_deref(),
            start,
            end,
            opt_format,
        )?;

        let path = PathBuf::from(&output_path);
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent).with_context(|| {
                    format!(
                        "Unable to create calendar output directory {}",
                        parent.display()
                    )
                })?;
            }
        }

        fs::write(&path, contents)
            .with_context(|| format!("Failed to write calendar output to {}", path.display()))?;

        let normalized = path.to_string_lossy().to_string();
        self.calendar_draft.output_path = normalized.clone();
        Ok(normalized)
    }

    pub fn set_location(&mut self, city: &City) {
        self.location = Location::new_unchecked(city.lat, city.lon);
        self.timezone = city.tz.parse().unwrap_or(chrono_tz::UTC);
        self.city_name = Some(city.name.clone());
        self.nearest_city_info = None; // Clear nearest city info when using city picker
        self.location_source = LocationSource::CityDatabase;
        self.should_save = true;
        self.update_time();
        self.reset_cached_data();
        #[cfg(feature = "ai-insights")]
        {
            self.ai_last_refresh = None;
            self.ai_outcome = None;
        }
    }

    pub fn update_city_search(&mut self, query: &str) {
        self.city_search = query.to_string();
        self.city_selected = 0;

        if let Ok(db) = crate::city::CityDatabase::load() {
            self.city_results = db
                .search(&self.city_search)
                .into_iter()
                .take(20)
                .map(|(city, _score)| city.clone())
                .collect();
        }
    }

    pub fn select_next_city(&mut self) {
        if !self.city_results.is_empty() && self.city_selected < self.city_results.len() - 1 {
            self.city_selected += 1;
        }
    }

    pub fn select_prev_city(&mut self) {
        if self.city_selected > 0 {
            self.city_selected -= 1;
        }
    }

    pub fn select_current_city(&mut self) {
        if !self.city_results.is_empty() && self.city_selected < self.city_results.len() {
            let city = self.city_results[self.city_selected].clone();
            self.set_location(&city);
            self.mode = AppMode::Watch;
        }
    }

    #[cfg(feature = "ai-insights")]
    pub fn should_refresh_ai(&self) -> bool {
        if !self.ai_config.enabled {
            return false;
        }

        // If manual only mode, never auto-refresh
        if self.ai_config.refresh_mode == config::AiRefreshMode::ManualOnly {
            return false;
        }

        if self.ai_job_rx.is_some() {
            return false;
        }

        match self.ai_last_refresh {
            None => true,
            Some(last) => last.elapsed() >= self.ai_config.refresh,
        }
    }

    #[cfg(feature = "ai-insights")]
    pub fn toggle_ai_enabled(&mut self) {
        let was_enabled = self.ai_config_draft.enabled;
        self.ai_config_draft.toggle_enabled();
        self.ai_config_draft.clear_error();
        if self.ai_config_draft.enabled && !was_enabled {
            self.probe_ai_server_for_draft();
        } else if !self.ai_config_draft.enabled {
            self.ai_config_draft.reset_detection();
        }
    }

    #[cfg(feature = "ai-insights")]
    pub fn advance_ai_field(&mut self) {
        let previous = self.ai_config_draft.current_field();
        self.ai_config_draft.next_field();
        self.handle_ai_field_exit(previous);
    }

    #[cfg(feature = "ai-insights")]
    pub fn retreat_ai_field(&mut self) {
        let previous = self.ai_config_draft.current_field();
        self.ai_config_draft.prev_field();
        self.handle_ai_field_exit(previous);
    }

    #[cfg(feature = "ai-insights")]
    pub fn cycle_ai_model(&mut self, delta: isize) {
        if self.ai_config_draft.current_field() == AiConfigField::Model {
            self.ai_config_draft.cycle_model(delta);
        }
    }

    #[cfg(feature = "ai-insights")]
    fn handle_ai_field_exit(&mut self, previous: AiConfigField) {
        match previous {
            AiConfigField::Enabled => {
                if self.ai_config_draft.enabled {
                    self.probe_ai_server_for_draft();
                } else {
                    self.ai_config_draft.reset_detection();
                }
            }
            AiConfigField::Server => {
                if self.ai_config_draft.enabled {
                    self.probe_ai_server_for_draft();
                }
            }
            _ => {}
        }
    }

    #[cfg(feature = "ai-insights")]
    fn probe_ai_server_for_draft(&mut self) {
        if !self.ai_config_draft.enabled {
            return;
        }

        let normalized = ai::AiConfig::normalized_server(true, &self.ai_config_draft.server);

        match ai::probe_server(&normalized) {
            Ok(models) => {
                self.ai_config_draft
                    .set_detection_success(normalized.clone(), models);
                self.ai_config_draft.clear_error();
            }
            Err(err) => {
                self.ai_config_draft
                    .set_detection_failure(normalized.clone(), err.to_string());
            }
        }
    }

    #[cfg(feature = "ai-insights")]
    pub fn refresh_ai_insights(&mut self) {
        self.start_ai_refresh_job();
    }

    #[cfg(feature = "ai-insights")]
    fn start_ai_refresh_job(&mut self) {
        if !self.ai_config.enabled {
            return;
        }

        if self.ai_job_rx.is_some() {
            return;
        }

        self.refresh_scheduled_data();
        let now_tz = self.current_time.with_timezone(&self.timezone);

        let timed_events = self.events_cache.entries.clone();
        let next_idx = timed_events.iter().position(|(time, _)| *time > now_tz);
        let event_summaries = ai::prepare_event_summaries(&timed_events, &now_tz, next_idx);

        let sun_pos = self.positions_cache.sun;
        let moon_pos = self.positions_cache.moon;

        let ai_data = ai::build_ai_data(ai::AiDataContext {
            location: &self.location,
            timezone: &self.timezone,
            dt: &now_tz,
            city_name: self.city_name.as_deref(),
            sun_pos: &sun_pos,
            moon_pos: &moon_pos,
            events: event_summaries,
            time_sync_info: &self.time_sync,
            lunar_phases: &self.lunar_phases_cache,
        });

        let config = self.ai_config.clone();
        let (tx, rx) = mpsc::channel();
        let previous_outcome = self.ai_outcome.clone();

        thread::spawn(move || {
            let result = ai::fetch_insights(&config, &ai_data).map_err(|err| err.to_string());
            let _ = tx.send(result);
        });

        self.ai_job_prev_outcome = previous_outcome;
        self.ai_job_rx = Some(rx);
    }

    #[cfg(feature = "ai-insights")]
    fn poll_ai_job(&mut self) {
        if let Some(rx) = &self.ai_job_rx {
            match rx.try_recv() {
                Ok(result) => {
                    match result {
                        Ok(outcome) => {
                            self.ai_outcome = Some(outcome);
                        }
                        Err(err_string) => {
                            if let Some(prev) = self.ai_job_prev_outcome.take() {
                                self.ai_outcome = Some(prev.with_error_message(err_string));
                            } else {
                                self.ai_outcome = Some(ai::AiOutcome::from_error(
                                    &self.ai_config.model,
                                    anyhow::anyhow!(err_string),
                                ));
                            }
                        }
                    }
                    self.ai_job_rx = None;
                    self.ai_job_prev_outcome = None;
                    self.ai_last_refresh = Some(Instant::now());
                }
                Err(mpsc::TryRecvError::Empty) => {}
                Err(mpsc::TryRecvError::Disconnected) => {
                    if let Some(prev) = self.ai_job_prev_outcome.take() {
                        self.ai_outcome =
                            Some(prev.with_error_message("AI refresh interrupted".to_string()));
                    }
                    self.ai_job_rx = None;
                }
            }
        }
    }

    #[cfg(feature = "ai-insights")]
    pub fn open_ai_config(&mut self) {
        self.ai_config_draft.sync_from(&self.ai_config);
        if self.ai_config_draft.enabled {
            self.probe_ai_server_for_draft();
        }
        self.mode = AppMode::AiConfig;
    }

    #[cfg(feature = "ai-insights")]
    pub fn apply_ai_config_changes(&mut self) -> Result<()> {
        let minutes_str = self.ai_config_draft.refresh_minutes.trim();
        if minutes_str.is_empty() {
            return Err(anyhow!("Refresh minutes cannot be empty"));
        }

        let minutes = minutes_str
            .parse::<u64>()
            .map_err(|_| anyhow!("Refresh minutes must be a number between 1 and 60"))?;
        if minutes == 0 || minutes > 60 {
            return Err(anyhow!("Refresh minutes must be between 1 and 60"));
        }

        let model = self.ai_config_draft.model.trim();
        if model.is_empty() {
            return Err(anyhow!("Model name cannot be empty"));
        }

        let normalized_server = ai::AiConfig::normalized_server(
            self.ai_config_draft.enabled,
            &self.ai_config_draft.server,
        );

        if self.ai_config_draft.enabled {
            let reuse_models = matches!(
                &self.ai_config_draft.server_status,
                AiServerStatus::Connected { server } if server == &normalized_server
            ) && !self.ai_config_draft.models.is_empty();

            let models = if reuse_models {
                self.ai_config_draft.models.clone()
            } else {
                ai::probe_server(&normalized_server).map_err(|err| {
                    anyhow!(
                        "Unable to reach Ollama server at {} ({})",
                        normalized_server,
                        err
                    )
                })?
            };

            self.ai_config_draft
                .set_detection_success(normalized_server.clone(), models);
            if self.ai_config_draft.model.trim().is_empty() {
                return Err(anyhow!("Select a model to continue"));
            }
        } else {
            self.ai_config_draft.reset_detection();
        }

        let final_model = self.ai_config_draft.model.trim();
        if final_model.is_empty() {
            return Err(anyhow!("Model name cannot be empty"));
        }

        self.ai_config.enabled = self.ai_config_draft.enabled;
        self.ai_config.server = normalized_server;
        self.ai_config.model = final_model.to_string();
        self.ai_config.refresh = Duration::from_secs(minutes * 60);
        self.ai_config.refresh_mode = self.ai_config_draft.refresh_mode;

        self.ai_config_draft.sync_from(&self.ai_config);
        self.ai_outcome = None;
        self.ai_last_refresh = None;

        if self.ai_config.enabled {
            self.start_ai_refresh_job();
        }

        Ok(())
    }

    pub fn open_settings(&mut self) {
        // Sync current app state to settings draft
        self.settings_draft = SettingsDraft {
            location_mode: self.location_mode,
            time_sync_enabled: !self.time_sync_disabled,
            time_sync_server: self.time_sync_server.clone(),
            show_location_date: self.show_location_date,
            show_events: self.show_events,
            show_positions: self.show_positions,
            show_moon: self.show_moon,
            show_lunar_phases: self.show_lunar_phases,
            night_mode: self.night_mode,
            #[cfg(feature = "ai-insights")]
            ai_enabled: self.ai_config.enabled,
            #[cfg(feature = "ai-insights")]
            ai_server: self.ai_config.server.clone(),
            #[cfg(feature = "ai-insights")]
            ai_model: self.ai_config.model.clone(),
            #[cfg(feature = "ai-insights")]
            ai_refresh_minutes: self.ai_config.refresh_minutes().to_string(),
            field_index: 0,
            error: None,
            #[cfg(feature = "ai-insights")]
            ai_server_status: AiServerStatus::Unknown,
            #[cfg(feature = "ai-insights")]
            ai_models: Vec::new(),
            #[cfg(feature = "ai-insights")]
            ai_model_index: None,
        };
        // Probe AI server if AI is enabled
        #[cfg(feature = "ai-insights")]
        if self.ai_config.enabled {
            self.probe_ai_server_for_settings();
        }
        self.mode = AppMode::Settings;
    }

    pub fn apply_settings_changes(&mut self) -> Result<()> {
        // Validate and apply changes

        // Validate AI refresh minutes
        #[cfg(feature = "ai-insights")]
        if self.settings_draft.ai_enabled {
            let minutes_str = self.settings_draft.ai_refresh_minutes.trim();
            if minutes_str.is_empty() {
                return Err(anyhow!("AI refresh minutes cannot be empty"));
            }

            let minutes = minutes_str
                .parse::<u64>()
                .map_err(|_| anyhow!("AI refresh minutes must be a number between 1 and 60"))?;
            if minutes == 0 || minutes > 60 {
                return Err(anyhow!("AI refresh minutes must be between 1 and 60"));
            }

            self.ai_config.refresh = Duration::from_secs(minutes * 60);
        }

        // Apply location mode
        self.location_mode = self.settings_draft.location_mode;

        // Apply time sync settings
        self.time_sync_disabled = !self.settings_draft.time_sync_enabled;
        self.time_sync_server = self.settings_draft.time_sync_server.clone();

        // Apply panel visibility
        self.show_location_date = self.settings_draft.show_location_date;
        self.show_events = self.settings_draft.show_events;
        self.show_positions = self.settings_draft.show_positions;
        self.show_moon = self.settings_draft.show_moon;
        self.show_lunar_phases = self.settings_draft.show_lunar_phases;

        // Apply night mode
        self.night_mode = self.settings_draft.night_mode;

        // Apply AI settings
        #[cfg(feature = "ai-insights")]
        {
            self.ai_config.enabled = self.settings_draft.ai_enabled;
            self.ai_config.server = self.settings_draft.ai_server.clone();
            self.ai_config.model = self.settings_draft.ai_model.clone();

            // Reset AI refresh if settings changed
            if self.ai_config.enabled {
                self.ai_outcome = None;
                self.ai_last_refresh = None;
                self.start_ai_refresh_job();
            }
        }

        self.should_save = true;
        Ok(())
    }

    pub fn reset_settings_to_defaults(&mut self) {
        self.settings_draft = SettingsDraft {
            location_mode: config::LocationMode::City,
            time_sync_enabled: true,
            time_sync_server: "time.google.com".to_string(),
            show_location_date: true,
            show_events: true,
            show_positions: true,
            show_moon: true,
            show_lunar_phases: true,
            night_mode: false,
            #[cfg(feature = "ai-insights")]
            ai_enabled: false,
            #[cfg(feature = "ai-insights")]
            ai_server: "http://localhost:11434".to_string(),
            #[cfg(feature = "ai-insights")]
            ai_model: "llama3.2:latest".to_string(),
            #[cfg(feature = "ai-insights")]
            ai_refresh_minutes: "2".to_string(),
            field_index: 0,
            error: None,
            #[cfg(feature = "ai-insights")]
            ai_server_status: AiServerStatus::Unknown,
            #[cfg(feature = "ai-insights")]
            ai_models: Vec::new(),
            #[cfg(feature = "ai-insights")]
            ai_model_index: None,
        };
    }

    #[cfg(feature = "ai-insights")]
    pub fn probe_ai_server_for_settings(&mut self) {
        if !self.settings_draft.ai_enabled {
            return;
        }

        let normalized = ai::AiConfig::normalized_server(true, &self.settings_draft.ai_server);

        match ai::probe_server(&normalized) {
            Ok(mut models) => {
                self.settings_draft.ai_server_status = AiServerStatus::Connected {
                    server: normalized.clone(),
                };
                self.settings_draft.ai_server = normalized;
                models.sort();
                models.dedup();
                self.settings_draft.ai_models = models.clone();
                self.settings_draft.error = None;

                if models.is_empty() {
                    self.settings_draft.ai_model_index = None;
                    return;
                }

                if let Some(idx) = models.iter().position(|name| name == &self.settings_draft.ai_model) {
                    self.settings_draft.ai_model_index = Some(idx);
                    self.settings_draft.ai_model = models[idx].clone();
                } else {
                    let idx = 0;
                    self.settings_draft.ai_model_index = Some(idx);
                    self.settings_draft.ai_model = models[idx].clone();
                }
            }
            Err(err) => {
                self.settings_draft.ai_server_status = AiServerStatus::Failed {
                    server: normalized.clone(),
                    message: err.to_string(),
                };
                self.settings_draft.ai_server = normalized;
                self.settings_draft.ai_model_index = None;
                self.settings_draft.ai_models.clear();
            }
        }
    }

    #[cfg(feature = "ai-insights")]
    pub fn cycle_ai_model_in_settings(&mut self, delta: isize) {
        if self.settings_draft.ai_models.is_empty() {
            return;
        }

        let len = self.settings_draft.ai_models.len() as isize;
        let current = self.settings_draft.ai_model_index.unwrap_or(0) as isize;
        let mut next = current + delta;
        if next < 0 {
            next = (next % len + len) % len;
        } else {
            next %= len;
        }

        self.settings_draft.ai_model_index = Some(next as usize);
        if let Some(model) = self.settings_draft.ai_models.get(next as usize) {
            self.settings_draft.ai_model = model.clone();
        }
        self.settings_draft.error = None;
    }
}
