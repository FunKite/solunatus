// Command-line argument parsing

use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum CalendarFormatArg {
    Html,
    Json,
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

    /// Calendar output format (html or json)
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
}

impl Args {
    pub fn should_watch(&self) -> bool {
        self.watch || (!self.json && !self.no_prompt)
    }
}
