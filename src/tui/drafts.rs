//! Input draft types for the TUI settings and forms.
//!
//! These types handle user input state for various forms in the TUI,
//! including location input, calendar generation, and AI configuration.

use crate::calendar::CalendarFormat;
use crate::config;
use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Datelike, Local, NaiveDate};
use std::path::PathBuf;

#[cfg(feature = "ai-insights")]
use crate::ai;

// ============================================================================
// Location Input Draft
// ============================================================================

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

// ============================================================================
// Calendar Draft
// ============================================================================

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

// ============================================================================
// AI Config Draft (feature-gated)
// ============================================================================

#[cfg(feature = "ai-insights")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiConfigField {
    Enabled,
    Server,
    Model,
    RefreshMinutes,
    RefreshMode,
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

// ============================================================================
// Settings Draft
// ============================================================================

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
    #[cfg(feature = "ai-insights")]
    AiEnabled,
    #[cfg(feature = "ai-insights")]
    AiServer,
    #[cfg(feature = "ai-insights")]
    AiModel,
    #[cfg(feature = "ai-insights")]
    AiRefreshMinutes,
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
            SettingsField::AiRefreshMinutes
                if c.is_ascii_digit() && self.ai_refresh_minutes.len() < 2 =>
            {
                self.ai_refresh_minutes.push(c);
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
