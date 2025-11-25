# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.1] - TBD

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
