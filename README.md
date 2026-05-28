# Solunatus

[![Crates.io](https://img.shields.io/crates/v/solunatus.svg)](https://crates.io/crates/solunatus)
[![Downloads](https://img.shields.io/crates/d/solunatus.svg)](https://crates.io/crates/solunatus)
[![Documentation](https://docs.rs/solunatus/badge.svg)](https://docs.rs/solunatus)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

High-precision astronomical calculations for sun and moon events, available as both:
- A Rust library (`solunatus`)
- A CLI app (`solunatus`)

Solunatus runs offline for core calculations and supports historical/future dates (from astronomical year `-0999` through `3000`).

## What You Get

- Sunrise, sunset, and solar noon
- Civil, nautical, and astronomical twilight
- Moonrise, moonset, and transit
- Moon phase, illumination, altitude/azimuth, distance
- Built-in city database (570+ cities)
- Interactive terminal UI (watch mode)
- JSON output and calendar export (HTML/JSON)
- Optional USNO validation reports
- Optional AI insights via local Ollama

## Install

Solunatus targets the latest stable Rust release for active development. The current release line supports stable Rust versions compatible with `rust-version = "1.91"`, and that floor may rise in a minor release when security, dependency compatibility, or maintainability require it. If you need an older Rust toolchain, use an older Solunatus release that still supports it.

To upgrade an existing installation from crates.io:

```bash
cargo install solunatus --force
```

### From crates.io

```bash
cargo install solunatus
```

### From source

```bash
git clone https://github.com/FunKite/solunatus.git
cd solunatus
cargo install --path .
```

## Quick Start

```bash
# Use a city from the built-in database
solunatus --city "New York"

# Or specify coordinates + timezone
solunatus --lat 40.7128 --lon -74.0060 --tz America/New_York
```

By default, Solunatus starts in interactive watch mode.  
Press `q` to quit, `s` for settings, and `r` for reports.

## Common CLI Usage

### Single snapshot (non-interactive)

```bash
solunatus --city "Tokyo" --no-prompt
```

### JSON output

```bash
solunatus --city "Tokyo" --json
```

### Specific date

```bash
solunatus --city "Lisbon" --date 2026-01-15
```

### Calendar export

```bash
solunatus --city "Lisbon" \
  --calendar \
  --calendar-start 2026-01-01 \
  --calendar-end 2026-01-31 \
  --calendar-format html \
  --calendar-output lisbon-jan-2026.html
```

### USNO validation report (feature-enabled builds)

```bash
solunatus --city "San Diego" --validate
```

### AI insights with Ollama (feature-enabled builds)

```bash
solunatus --city "Seattle" --ai-insights
```

## Command Highlights

Core flags:
- `--city <CITY>`
- `--lat <LAT> --lon <LON> --tz <TIMEZONE>`
- `--date <YYYY-MM-DD>`
- `--json`
- `--calendar --calendar-start <DATE> --calendar-end <DATE>`
- `--calendar-format <html|json>`
- `--calendar-output <PATH>`
- `--watch`
- `--no-prompt`
- `--no-save`
- `--strict`

Optional flags (feature gated):
- `--validate` (`usno-validation`)
- `--ai-insights`, `--ai-server`, `--ai-model`, `--ai-refresh-minutes` (`ai-insights`)

For full CLI docs, see [`docs/features/cli-reference.md`](docs/features/cli-reference.md).

## Optional Features

Default features:
- `usno-validation`
- `ai-insights`

Examples:

```bash
# Minimal build (no USNO validation, no AI insights)
cargo install solunatus --no-default-features

# USNO validation only
cargo install solunatus --no-default-features --features usno-validation

# AI insights only
cargo install solunatus --no-default-features --features ai-insights
```

## Library Usage

Add dependency:

```toml
[dependencies]
solunatus = "0.4.0"
chrono = "0.4"
chrono-tz = "0.10"
```

Basic example:

```rust
use chrono::Local;
use chrono_tz::America::New_York;
use solunatus::prelude::*;

fn main() {
    let location = Location::new(40.7128, -74.0060).unwrap();
    let now = Local::now().with_timezone(&New_York);

    if let Some(sunrise) = calculate_sunrise(&location, &now) {
        println!("Sunrise: {}", sunrise.format("%H:%M:%S"));
    }

    if let Some(sunset) = calculate_sunset(&location, &now) {
        println!("Sunset: {}", sunset.format("%H:%M:%S"));
    }

    let (phase_name, phase_emoji) = get_current_moon_phase(&location, &now);
    println!("Moon phase: {} {}", phase_emoji, phase_name);
}
```

More examples: [`examples/`](examples/)

## Accuracy and Scope

Solunatus uses NOAA-based solar methods and Meeus-based lunar methods, with validation tooling aligned to USNO-style conventions.

This project is intended for educational, planning, and general-purpose astronomical use.  
It is not certified for safety-critical navigation or legal timing decisions.

## Configuration

CLI settings are saved to:

```text
~/.solunatus.json
```

Use `--no-save` to avoid writing configuration.

## Releases

Crates.io is the supported distribution channel for published releases:

```bash
cargo install solunatus
```

GitHub Releases are used for tags and release notes for each published version. They should not be treated as a guaranteed source of fresh binary artifacts unless a specific release explicitly says otherwise.

## Development

```bash
# Build
cargo build

# Test
cargo test

# Safer local test runner
./scripts/safe_local_test.sh
```

Project docs index: [`docs/README.md`](docs/README.md)

## Contributing

Issue reports and feature requests are welcome:
- [GitHub Issues](https://github.com/FunKite/solunatus/issues)
- [Contributing Guide](CONTRIBUTING.md)
- [Code of Conduct](CODE_OF_CONDUCT.md)
- [Security Policy](SECURITY.md)

## License

MIT ([`LICENSE`](LICENSE))
