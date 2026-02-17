# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed
- **Dependencies**: Updated `clap` to 4.5.56 and `chrono` to 0.4.43 (Dependabot PR #24)
- **Dependencies**: Updated `serde_json` to 1.0.149 and `reqwest` to 0.12.28
- **Dependencies**: Updated `ratatui` to 0.30.0 (brings in `lru` 0.16.3)
- **Dependencies**: Updated `bytes` to 1.11.1 (PR #25)
- **Dependencies**: Updated `time` to 0.3.47 (Dependabot PR #26)
- **Dependencies**: Updated `clap` to 4.5.57 and `anyhow` to 1.0.101 (Dependabot PR #27)
- **Dependencies**: Updated `clap` to 4.5.58 (Dependabot PR #29)
- **CI**: Added Rust CI and security workflows with SHA-pinned actions and required checks (`Build`, `Clippy`, `Rustfmt`, `Feature Tests`, `Documentation`, `Rust Security Audit`, `Security Workflow Audit`)
- **Security**: Hardened GitHub Actions policy to selected actions with SHA pinning required
- **CLI**: Improved manual timezone handling by surfacing invalid timezone values as errors instead of silently falling back to UTC
- **Time Sync**: Improved source reporting to preserve the queried server identity in sync results
- **Documentation**: Simplified and modernized `README.md` quick start, feature, and usage sections
- **Algorithm**: Refined moon batch rise/set sweep to scan the full day with contiguous crossing brackets
- **CI**: Fixed `clippy` violations in moon batch optimization code to restore Rust CI green status (PR #31)

### Added
- **Developer Safety**: Added `scripts/safe_local_test.sh` for safer local testing (offline by default, credential scrubbing, isolated target directory)
- **Security**: Added workflow pin drift check to verify pinned action SHAs against tracked major tags

### Security
- **Dependencies**: Updated `lru` to 0.16.3 to address GHSA-rhfx-m35p-ff5j (low severity)

## [0.3.2] - 2025-12-04

### Changed
- **Code Quality**: Refactored `print_text_output` in main.rs, reducing code duplication by ~60%
  - Created helper functions: `print_header()`, `print_location_section()`, `print_events_section()`, `print_position_section()`, `moon_size_class()`, `print_moon_section()`, `print_lunar_phases_section()`, `print_ai_section()`
- **Code Quality**: Split `tui/app.rs` into smaller, focused modules
  - New `src/tui/drafts.rs` (~600 lines): `LocationInputDraft`, `CalendarDraft`, `AiConfigDraft`, `SettingsDraft` and their field enums
  - New `src/tui/cache.rs` (~90 lines): `CachedEvents`, `CachedPositions`, `CachedMoonDetails`, `MoonAltitudeTrend`
  - Reduced `app.rs` by ~400 lines for better maintainability
- **Code Quality**: Added `#[must_use]` attributes to pure functions across astronomical modules
  - `src/astro/mod.rs`: `julian_day()`, `julian_century()`, `normalize_degrees()`, `normalize_degrees_signed()`
  - `src/astro/sun.rs`: `equation_of_time()`, `solar_noon()`, `solar_position()`
  - `src/astro/moon.rs`: `lunar_position()`, `lunar_phases()`, `phase_name()`, `phase_emoji()`
- **Accuracy**: Enhanced lunar distance calculation with additional Meeus periodic terms in `moon.rs`

### Fixed
- **Safety**: Changed `jd_to_datetime` return type from `DateTime<Utc>` to `Option<DateTime<Utc>>` to prevent potential panics
  - Uses `.single()` instead of `.unwrap()` for safe date construction
  - Updated `lunar_phases()` to handle `None` with let-else pattern

### Documentation
- **Comprehensive Module Documentation**: Significantly improved crates.io documentation coverage (from 37.5% baseline)
  - **Module-level docs**: Added detailed module documentation for `ai`, `time_sync`, `benchmark`, `location_source`, and `usno_validation`
  - **Struct documentation**: Documented all public structs in `output` (14 structs), `config` (4 structs), `ai` (9 structs), and `usno_validation` (3 structs) with field-level descriptions
  - **Function documentation**: Added comprehensive docs for:
    - `ai`: `fetch_insights`, `probe_server`, `build_ai_data`, `prepare_event_summaries`
    - `time_sync`: `check_time_sync`, `check_time_sync_with_servers`, `format_offset`, `describe_direction`, `direction_code`
    - `calendar`: `generate_calendar`
    - `output`: `generate_json_output` (both AI and non-AI variants)
    - `benchmark`: `run_benchmark`, `generate_html_report`
  - **Enum documentation**: Documented `ValidationStatus`, `TimeSyncDirection`, `LocationMode`, `AiRefreshMode`, `LocationSource`, `CalendarFormat`
  - **Usage examples**: Added practical examples for all major public functions showing real-world usage patterns
  - **Error documentation**: Documented error cases and return values for all fallible functions
  - **Configuration examples**: Added JSON configuration examples for `Config` and `AiSettings`
  - Fixed broken intra-doc links for better documentation navigation
  - Documentation now builds without warnings (0 warnings, 33 tests passing)

## [0.3.1] - 2025-11-25

### Changed
- **Binary Size Optimization**: Made `rayon` dependency optional, reducing minimal build size by 59% (#17)
  - New `parallel` feature flag for optional Rayon parallelization in calendar generation
  - Minimal build (`--no-default-features`): 2.4 MB vs 5.8 MB full build
  - `parallel` feature is disabled by default to minimize binary size
  - Calendar generation works with or without the `parallel` feature

### Documentation
- Added comprehensive library usage documentation in README
- Documented all feature flags (`cpu-portable`, `usno-validation`, `ai-insights`, `parallel`)
- Added feature flag examples for different use cases (minimal, core + parallel, full)
- Clarified binary size and compilation benefits of disabling optional features
- Updated Dependencies section to distinguish core vs optional dependencies

### Fixed
- Resolved cross-compilation issues by making `rayon` optional (addresses #17)

## [0.3.0] - 2025-11-25

### Added
- **Optional Features**: `reqwest` dependency is now optional, allowing minimal builds without OpenSSL
  - New feature flags: `usno-validation` and `ai-insights` (both enabled by default)
  - Build without reqwest: `cargo install solunatus --no-default-features`
  - Solves cross-compilation issues (#17)

### Changed
- **Security**: Switched from OpenSSL to `rustls-tls` backend for HTTP requests (pure Rust TLS)
- **Events Display**: Filter events to ±12h sliding window from current time for cleaner output
- Updated `clap` from 4.5.51 to 4.5.53 (#15)
- Updated `actions/checkout` from 5 to 6 in CI workflow (#16)

## [0.2.3] - 2025-11-14

### Changed
- **Security**: Improved HTTP client security with explicit TLS verification enforcement
- **Security**: Replaced `unsafe` code in M1 optimizations with safe explicit initialization
- **Robustness**: Replaced `.unwrap()` call in date parsing with proper error handling
- **Robustness**: Improved configuration path detection using `dirs` crate instead of manual environment variable checks
- **Code Quality**: Removed hardcoded version strings - now uses `CARGO_PKG_VERSION` throughout
- **Code Quality**: Removed hardcoded city count - now dynamically loaded from database
- **Validation**: Added CLI-level input validation for AI refresh interval (enforces 1-60 minute range)
- Updated `clap` from 4.5.50 to 4.5.51

### Documentation
- Comprehensive README improvements with expanded feature descriptions and usage examples
- Added detailed solar and lunar calculation documentation in `docs/development/`
- Enhanced troubleshooting guide with additional common issues and solutions
- Improved AI insights documentation
- Better crates.io documentation coverage
- Updated Code of Conduct

### Fixed
- Better error messages when date/time parsing fails
- More reliable home directory detection across platforms

## [0.2.2] - 2025-10-25

### Fixed
- **Actually fixed benchmark binaries being listed for installation** - removed benchmark `[[bin]]` entries from Cargo.toml
- Now only the `solunatus` binary is installed via `cargo install` (benchmarks can still be built locally with `cargo build --bin <name>`)

## [0.2.1] - 2025-10-25

### Fixed
- Fixed README.md title displaying "Astrotimes" instead of "Solunatus" on crates.io
- Fixed broken screenshot link in README.md

## [0.2.0] - 2025-10-25

### Changed
- **BREAKING**: Renamed project from "astrotimes" to "solunatus" to avoid naming conflict with existing crate
- Updated crate name, binary name, and all documentation to reflect new name "Solunatus"
- Configuration file path changed from `~/.astro_times.json` to `~/.solunatus.json`
- NTP cache file path changed from `~/.astrotimes_ntp_cache.json` to `~/.solunatus_ntp_cache.json`
- Environment variable changed from `ASTROTIMES_SKIP_TIME_SYNC` to `SOLUNATUS_SKIP_TIME_SYNC`
- Benchmark binaries now excluded from `cargo install` (only main `solunatus` binary installed)
- Updated `clap` from 4.5.48 to 4.5.50
- Updated `reqwest` from 0.12.23 to 0.12.24
- Updated `ratatui` from 0.28.1 to 0.29.0
- Updated `crossterm` from 0.28.1 to 0.29.0
- Updated `chrono-tz` from 0.9.0 to 0.10.4

### Migration Notes
- Users migrating from astrotimes 0.1.x will need to reconfigure their location settings
- Previous astrotimes versions have been yanked from crates.io
- Install with: `cargo install solunatus`

## [0.1.1] - 2025-10-24 (yanked)

### Fixed
- Fixed doctest example in `batch_calculate` function missing `TimeZone` import

### Changed
- First public release to crates.io
- Repository made public on GitHub

## [0.1.0] - 2025-10-22 (yanked)

### Added
- Initial release of AstroTimes as a Rust library and CLI application
- NOAA solar position and event calculations (sunrise, sunset, twilight times)
- Meeus lunar position and phase calculations
- Interactive terminal UI (watch mode) with keyboard controls
- City database with 570+ worldwide locations
- JSON output mode for programmatic access
- HTML calendar generation for date ranges
- AI-powered insights via local Ollama integration
- System clock synchronization verification
- Configuration persistence (~/.astro_times.json)
- Library API for integration into Rust projects

### Technical Highlights
- Pure Rust implementation with no external astronomical calculation dependencies
- Accuracy within 1-3 minutes of U.S. Naval Observatory reference data
- Cross-platform support (macOS, Linux, Windows)
- Single self-contained binary with embedded city database
- Offline-first design with optional online features

## Roadmap 
 - The roadmap is subject to change based on various factors.
   
### Planned Features
- [ ] Planetary positions (Mercury, Venus, Mars, Jupiter, Saturn)
- [ ] Eclipse predictions (solar and lunar)

### Future Enhancements
- Performance optimization for batch processing
- Additional city database expansion
