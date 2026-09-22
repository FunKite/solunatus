// Command-line argument parsing

use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum CalendarFormatArg {
    Html,
    Json,
    Ics,
}

/// Events queryable via `--next` (kebab-case on the command line,
/// e.g. `--next solar-noon`).
#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum NextEventArg {
    Sunrise,
    Sunset,
    SolarNoon,
    CivilDawn,
    CivilDusk,
    NauticalDawn,
    NauticalDusk,
    AstronomicalDawn,
    AstronomicalDusk,
    GoldenDawnStart,
    GoldenDawnEnd,
    GoldenDuskStart,
    GoldenDuskEnd,
    Moonrise,
    Moonset,
}

/// Output format for `--next`.
#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum TimeFormatArg {
    /// RFC 3339 with local offset (e.g. 2026-06-09T20:19:44-04:00)
    Iso,
    /// Unix epoch seconds
    Unix,
    /// Local time without offset (e.g. 2026-06-09 20:19:44)
    Local,
    /// Local time plus a countdown (e.g. ... (08:19:44 from now))
    Human,
}

#[derive(Parser, Debug, Clone)]
#[command(name = "solunatus")]
#[command(version)]
#[command(about = "High-precision astronomical CLI for sun and moon calculations", long_about = None)]
pub struct Args {
    /// Latitude in decimal degrees (positive North, negative South)
    #[arg(long)]
    pub lat: Option<f64>,

    /// Longitude in decimal degrees (positive East, negative West)
    #[arg(long)]
    pub lon: Option<f64>,

    /// Timezone (IANA timezone name, e.g. America/New_York)
    #[arg(long)]
    pub tz: Option<String>,

    /// Date in YYYY-MM-DD format (defaults to today). Prints a one-shot
    /// report for that date; the live dashboard always shows the current time.
    #[arg(long)]
    pub date: Option<String>,

    /// Select a city from the built-in database
    #[arg(long)]
    pub city: Option<String>,

    /// Output in JSON format
    #[arg(long)]
    pub json: bool,

    /// Plan a night: photography times, moon-free darkness, and planets.
    /// Uses local noon on --date (default: today at the location) through the next noon; runs offline.
    #[arg(long, visible_alias = "tonight", conflicts_with_all = ["calendar", "next", "watch"])]
    pub night: bool,

    /// Generate a calendar for the specified date range
    #[arg(long)]
    pub calendar: bool,

    /// Calendar output format (html, json, or ics)
    #[arg(long, default_value = "html", value_enum)]
    pub calendar_format: CalendarFormatArg,

    /// Calendar range start date (YYYY-MM-DD, supports negative years like -0999)
    #[arg(long, requires = "calendar")]
    pub calendar_start: Option<String>,

    /// Calendar range end date (YYYY-MM-DD, supports negative years like -0999)
    #[arg(long, requires = "calendar")]
    pub calendar_end: Option<String>,

    /// Path to write the generated calendar (stdout when omitted)
    #[arg(long, requires = "calendar")]
    pub calendar_output: Option<PathBuf>,

    /// Print the next occurrence of an event and exit (for scripting).
    /// Searches forward from now, or from noon of --date when given.
    #[arg(long, value_enum)]
    pub next: Option<NextEventArg>,

    /// Output format for --next
    #[arg(long, value_enum, default_value = "iso", requires = "next")]
    pub format: TimeFormatArg,

    /// Force watch mode (live updates)
    #[arg(long, conflicts_with = "date")]
    pub watch: bool,

    /// Disable all interactive prompts
    #[arg(long)]
    pub no_prompt: bool,

    /// Disable saving settings to config file
    #[arg(long)]
    pub no_save: bool,

    /// Deprecated no-op, hidden from help; slated for removal.
    /// (`--next` already exits with an error when an event never occurs.)
    #[arg(long, hide = true)]
    pub strict: bool,

    /// Enable AI insights via a local Ollama server
    #[cfg(feature = "ai-insights")]
    #[arg(long)]
    pub ai_insights: bool,

    /// Ollama server base URL or host:port (default: saved setting, else http://localhost:11434)
    #[cfg(feature = "ai-insights")]
    #[arg(long)]
    pub ai_server: Option<String>,

    /// Ollama model to query for insights (default: saved setting, else llama3.2:latest)
    #[cfg(feature = "ai-insights")]
    #[arg(long)]
    pub ai_model: Option<String>,

    /// Minutes between AI insight refreshes in watch mode (1-60; default: saved setting, else 2)
    #[cfg(feature = "ai-insights")]
    #[arg(long, value_parser = clap::value_parser!(u64).range(1..=60))]
    pub ai_refresh_minutes: Option<u64>,

    /// Generate USNO validation report comparing calculations with Naval Observatory data
    #[cfg(feature = "usno-validation")]
    #[arg(long, conflicts_with = "night")]
    pub validate: bool,

    /// Generate shell completions to stdout and exit
    #[arg(long, value_enum, value_name = "SHELL")]
    pub completions: Option<clap_complete::Shell>,

    /// Generate a man page (roff format) to stdout and exit
    #[arg(long)]
    pub manpage: bool,
}

impl Args {
    /// Whether to open the interactive dashboard.
    ///
    /// The dashboard tracks the current time, so a `--date` request gets the
    /// one-shot text report instead of being silently ignored.
    pub fn should_watch(&self) -> bool {
        self.watch || (!self.json && !self.no_prompt && self.date.is_none())
    }
}
