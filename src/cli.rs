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

    /// Date in YYYY-MM-DD format (defaults to today)
    #[arg(long)]
    pub date: Option<String>,

    /// Select a city from the built-in database
    #[arg(long)]
    pub city: Option<String>,

    /// Output in JSON format
    #[arg(long)]
    pub json: bool,

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
    #[arg(long)]
    pub watch: bool,

    /// Disable all interactive prompts
    #[arg(long)]
    pub no_prompt: bool,

    /// Disable saving settings to config file
    #[arg(long)]
    pub no_save: bool,

    /// Strict mode: exit with error if events don't occur (polar regions)
    #[arg(long)]
    pub strict: bool,

    /// Enable AI insights via a local Ollama server
    #[cfg(feature = "ai-insights")]
    #[arg(long)]
    pub ai_insights: bool,

    /// Ollama server base URL or host:port (defaults to http://localhost:11434)
    #[cfg(feature = "ai-insights")]
    #[arg(long, default_value = "http://localhost:11434")]
    pub ai_server: String,

    /// Ollama model to query for insights
    #[cfg(feature = "ai-insights")]
    #[arg(long, default_value = "llama3")]
    pub ai_model: String,

    /// Minutes between AI insight refreshes in watch mode (1-60, default 2)
    #[cfg(feature = "ai-insights")]
    #[arg(long, default_value_t = 2, value_parser = clap::value_parser!(u64).range(1..=60))]
    pub ai_refresh_minutes: u64,

    /// Generate USNO validation report comparing calculations with Naval Observatory data
    #[cfg(feature = "usno-validation")]
    #[arg(long)]
    pub validate: bool,

    /// Generate shell completions to stdout and exit
    #[arg(long, value_enum, value_name = "SHELL")]
    pub completions: Option<clap_complete::Shell>,

    /// Generate a man page (roff format) to stdout and exit
    #[arg(long)]
    pub manpage: bool,
}

impl Args {
    pub fn should_watch(&self) -> bool {
        self.watch || (!self.json && !self.no_prompt)
    }
}
