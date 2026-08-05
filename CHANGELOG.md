# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Security
- **CI**: Refreshed the `dtolnay/rust-toolchain` SHA pin (used in `rust.yml`, `security.yml`, `usno-validation.yml`, and `planet-validation.yml`) from `631a55b` to the current `stable` branch tip `4cda84d`. The old pin had never been updated since it was first added and had gone stale: `dtolnay/rust-toolchain` ships no tags, only branches (`stable`, `beta`, `nightly`, version branches) that are force-pushed to a new commit on every Rust release, so an unrefreshed SHA pin eventually points at a commit unreachable from any branch tip. This was silently breaking the weekly `github-actions` Dependabot update job (`error: no such commit 631a55b...`, recurring since at least 2026-07-14) and risked a hard CI failure once GitHub garbage-collected the orphaned commit
- **CI**: Extended the `Pin Drift Check` workflow job to also track `dtolnay/rust-toolchain` against its `stable` branch (previously it only covered `actions/checkout` and `github/codeql-action`, both of which were confirmed still current), so a future rebase of the `stable` branch is caught automatically instead of failing silently in Dependabot
- **CI**: Advanced the `github/codeql-action/init` and `.../analyze` pinned SHA from `7188fc3` (v4.37.1) through `e4fba86` (v4.37.3) and `f205ea1` (v4.37.4) to `5595cca` (v4.37.6) in `codeql.yml` via the workflow-dependencies group (Dependabot PRs #92 and #96) and maintenance PR #94 to clear the Pin Drift gate; no user-facing CodeQL changes upstream

### Changed
- **Dependencies**: Bumped `clap` from 4.6.2 to 4.6.4 and the transitive `clap_derive` from 4.6.1 to 4.6.4 via the production-dependencies group (Dependabot PR #93); pulls in `clap_derive`'s move to `syn` 3.0. Lockfile-only; no `Cargo.toml` constraints changed and `cargo audit` reports no known advisories

## [0.6.1] - 2026-07-20

### Security
- **Dependencies**: Bumped the transitive `quinn-proto` from 0.11.14 to 0.11.15 to remediate **RUSTSEC-2026-0185** (CVSS 7.5, high) — a remote memory-exhaustion flaw from unbounded out-of-order QUIC stream reassembly. `quinn-proto` reaches the tree only through the optional reqwest HTTP stack (behind the `usno-validation` / `ai-insights` features); lockfile-only, no `Cargo.toml` constraints changed, and `cargo audit` is clean again
- **CI**: Advanced the `actions/checkout` pinned SHA from `df4cb1c` (v6.0.3) to `9c091bb` (v7.0.0) across all workflows and retargeted the Pin Drift Check to the `v7` tag (Dependabot PR #76). v7.0.0 hardens supply-chain safety by blocking checkout of fork pull requests under the `pull_request_target` and `workflow_run` triggers; the SHA pin and `Security Workflow Audit` / `Pin Drift Check` gates remain green
- **CI**: Advanced the `github/codeql-action/init` and `.../analyze` pinned SHA from `8aad20d` (v4.36.2) to `54f647b` (v4.36.3) in `codeql.yml` to clear the workflow Pin Drift gate (Dependabot PR #81); no user-facing CodeQL changes upstream
- **CI**: Advanced the `github/codeql-action/init` and `.../analyze` pinned SHA from `54f647b` (v4.36.3) to `99df26d` (v4.37.0) in `codeql.yml` to clear the workflow Pin Drift gate (Dependabot PR #85); upstream bumps the default CodeQL bundle to 2.26.0, no user-facing CodeQL changes for this repo
- **Dependencies**: Bumped the transitive `crossbeam-epoch` from 0.9.18 to 0.9.20 to remediate **RUSTSEC-2026-0204** — an invalid pointer dereference in the `fmt::Pointer` impl for `Atomic`/`Shared` when the underlying pointer is invalid. `crossbeam-epoch` reaches the tree only through the optional `parallel` (rayon-backed) feature; lockfile-only, no `Cargo.toml` constraints changed, and `cargo audit` is clean again
- **CI**: Advanced the `actions/checkout` pinned SHA from `9c091bb` (v7.0.0) to `3d3c42e` (v7.0.1) across all workflows to clear the workflow Pin Drift gate; no user-facing checkout behavior changes for this repo

### Changed
- **Dependencies**: Bumped `ratatui` from 0.30.1 to 0.30.2 via the production-dependencies group (Dependabot PR #77); pulls in the refreshed `ratatui-core`/`-crossterm`/`-macros`/`-widgets` 0.1.2/0.7.2/0.3.2 stack and the new optional `ratatui-termina` backend (unused here). Lockfile-only; no `Cargo.toml` constraints changed and `cargo audit` reports no known advisories
- **Dependencies**: Bumped `anyhow` from 1.0.102 to 1.0.103 via the production-dependencies group (Dependabot PR #79); upstream fixes a Stacked Borrows violation (undefined behavior surfaced under Miri) in `Error::downcast_mut`. Lockfile-only; no `Cargo.toml` constraints changed and `cargo audit` reports no known advisories
- **Dependencies**: Bumped `clap_complete` from 4.6.5 to 4.6.7 via the production-dependencies group (Dependabot PR #82); adds `pwsh` detection to shell-completion generation. Lockfile-only; no `Cargo.toml` constraints changed and `cargo audit` reports no known advisories
- **Dependencies**: Bumped `clap` 4.6.1 → 4.6.2, `serde` 1.0.228 → 1.0.229, `serde_json` 1.0.150 → 1.0.151, and `anyhow` 1.0.103 → 1.0.104 via the production-dependencies group (Dependabot PR #88). Lockfile-only; no `Cargo.toml` constraints changed and `cargo audit` reports no known advisories

## [0.6.0] - 2026-06-10

### Added
- **Planet accuracy validation**: Planet positions are validated against the JPL Horizons ephemeris (altitude/azimuth within 0.06° across 1990–2049, ≈ a few seconds of rise/set time). New offline regression tests (`tests/planet_accuracy.rs`) pin Horizons reference values at three epochs and run on every build; a new scheduled CI workflow (`planet-validation.yml`, via `scripts/planet_drift_check.py`) re-checks live Horizons data weekly with thresholds of 0.25° position, 0.8 magnitude, and 0.5% distance
- **Planets**: New `astro::planets` module computing apparent positions (altitude, azimuth, distance, approximate visual magnitude, solar elongation) and rise/set times for the seven major planets (Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune) from Keplerian mean elements with the major Jupiter–Saturn and Uranus perturbation terms; surfaced as a "— Planets —" text section and a `planets` array in JSON output, with a regression test anchored to the December 2020 Jupiter–Saturn great conjunction
- **Seasons**: New `astro::seasons` module computing equinoxes and solstices (Meeus ch. 27, validated against the book's worked example) with ΔT correction to UTC; the next two seasonal events appear in text and JSON output
- **Golden/blue hour**: Four new `SolarEvent` variants at the -4° and +6° photography thresholds (`GoldenDawnStart/End`, `GoldenDuskStart/End`), a `photo_periods` helper returning the morning/evening golden hour and blue hour ranges, golden hour entries in the events timeline, a "— Photography —" text section, and a `sun.photography` JSON block
- **Dark-sky window**: New `events::next_dark_window` scans 36 hours ahead for the next period with the sun below -18° and the moon down (same DSD criteria as the events timeline) and reports it in the Photography section and as `dark_sky_window` in JSON
- **Lunar apsides**: New `next_lunar_apsides` finds the next perigee and apogee (ternary-search refinement of the Meeus distance series); shown in the Moon section and as `moon.apsides` in JSON. Full moons within 360,000 km are flagged as supermoons (`is_supermoon`) in the Lunar Phases section, JSON, and ICS export
- **iCalendar export**: `--calendar-format ics` generates an RFC 5545 calendar (sunrise/sunset/moonrise/moonset per day plus quarter lunar phases, UTC times, folded lines, deterministic UIDs) for import into calendar applications; also selectable in the TUI calendar generator
- **Scripting query mode**: `--next <event>` prints the next occurrence of any solar, golden hour, or lunar event and exits; `--format iso|unix|local|human` controls the output. Skips the NTP check so cron/automation calls stay fast and offline
- **Shell completions & man page**: `--completions <shell>` (bash, zsh, fish, elvish, powershell) and `--manpage` generate to stdout via `clap_complete`/`clap_mangen`
- **TUI altitude chart**: New chart view (`g` key in watch mode) plotting sun and moon altitude across the local day with a horizon line and a "now" marker; curves are cached and respect night mode
- **TUI planets panel**: The watch view gains a "— Planets —" section with a 60-second refresh countdown showing altitude, azimuth with compass direction, visual magnitude, and rise/set times for all seven major planets; toggleable via a new "Planets" entry in the settings Panel Visibility section, persisted as `watch.show_planets` in the config file

### Fixed
- **Feature-gated builds**: Builds with `--no-default-features` produced no output at all — a misplaced `#[cfg(feature = "usno-validation")]` attribute compiled out the entire output-mode dispatch (JSON, watch, and text modes), so the binary silently exited. The gate now applies only to the `--validate` branch
- **TUI events alignment**: Widened the events label column (16→17 normal, 14→15 night mode) so the 17-character golden hour labels no longer overflow it and push their countdown durations one column out of alignment with the other events
- **ICS phase boundary**: The iCalendar export now scans one UTC month past each end of the requested range when collecting quarter lunar phases, so a phase whose UTC timestamp falls in the neighboring month but whose local date is inside the range (e.g. the 2026-12-01 06:14 UTC last quarter, which is Nov 30 in America/Los_Angeles) is no longer dropped

### Changed
- **Documentation**: API documentation coverage raised from ~86% to 100% — all public items (the `astro::units` type-safe wrappers, `ai` configuration methods, `BatchResult`/`LibraryInfo` fields, and the `coordinates`/`time_utils` modules) now carry doc comments, and `#![warn(missing_docs)]` keeps it that way; refreshed the README for the 0.6.0 feature set
- **Dependencies**: Updated `ratatui` to 0.30.1 and `chrono` to 0.4.45 via the production-dependencies group (supersedes Dependabot PR #72); `chrono` 0.4.45 rejects a TZ offset hour of 24 to avoid a `FixedOffset` overflow, and the `ratatui` bump pulls in refreshed transitive crates (`lru` 0.18.0, `strum` 0.28.0, `bitflags` 2.13.0, and the new `palette` color stack). Lockfile-only; no `Cargo.toml` constraints changed and `cargo audit` reports no known advisories

### Security
- **CI**: Advanced the `github/codeql-action` pinned SHA from `7211b7c` (v4.36.0) to `8aad20d` (v4.36.2), and the `actions/checkout` pinned SHA from `de0fac2` (v6.0.2) to `df4cb1c` (v6.0.3), to clear the workflow Pin Drift gate and keep the SHA pins aligned with their tracked `v4` / `v6` tags
- **Dependencies**: Refreshed `Cargo.lock` to the latest Rust 1.91–compatible versions as supply-chain hygiene; notable movement in the transitive TLS/HTTP stack (behind the optional `usno-validation` / `ai-insights` reqwest features): `rustls` 0.23.34 → 0.23.40, `rustls-pki-types` 1.12.0 → 1.14.1, `hyper` 1.7.0 → 1.10.1, `hyper-rustls` 0.27.7 → 0.27.9, `tokio` 1.48.0 → 1.52.3, and `webpki-roots` 1.0.4 → 1.0.7, alongside other transitive crates. Lockfile-only; no `Cargo.toml` constraints changed. `cargo audit` reports no known advisories across the dependency tree

## [0.5.0] - 2026-05-29

### Added
- **Performance**: New optional `parallel` Cargo feature parallelizes multi-day calendar generation across CPU cores via rayon, using the canonical per-day algorithm so results are identical to the single-threaded path
- **Tests**: Added a regression test that locks calendar moonrise/moonset output to the canonical `lunar_event_time` algorithm

### Changed
- **Toolchain**: Migrated the crate to the Rust 2024 edition (`edition = "2024"`); the `rust-version = "1.91"` floor already satisfies the 2024 edition requirement (Rust 1.85+)
- **Code Quality**: Collapsed nested `if let` blocks into 2024-edition let-chains across the TUI and USNO validation modules, clearing the clippy warnings surfaced by the edition bump
- **Dependencies**: Bumped `clap` to 4.6.1 and `rayon` to 1.12.0 via the production-dependencies group (Dependabot PR #58)
- **Dependencies**: Updated `serde_json` to 1.0.150 via the production-dependencies group (Dependabot PR #64)
- **CI**: Refreshed `github/codeql-action` pinned SHA progression 4.35.2 → 4.35.3 → 4.35.4 → 4.35.5 → 4.36.0 (Dependabot PRs #59, #60, #61, #63) to stay aligned with the tracked `v4` tag

### Fixed
- **Calendar**: TUI calendar export now uses the same canonical lunar algorithm as the CLI, so both report identical moonrise/moonset times (the TUI previously used a separate "optimized" path that could return a different horizon crossing)
- **CLI**: `--calendar` is now a terminal one-shot and no longer falls through into interactive watch mode after writing the file
- **TUI**: Watch mode installs a panic hook and an RAII guard so the terminal (raw mode, alternate screen, cursor) is always restored on panics and early error returns, not just on the normal exit path
- **Config**: Saved-config and city-database coordinates are validated on load; a corrupt `~/.solunatus.json` now fails fast with an actionable message instead of feeding invalid values into the calculations
- **Stability**: Error-summary truncation in AI insights and time sync now respects UTF-8 char boundaries, fixing a potential panic on multi-byte characters

### Removed
- **Code Quality**: Removed unused/mislabeled optimization modules (`simd_math`, `m1_optimizations`, `moon_batch_optimized`, `calendar_optimized`) and the excluded `src/bin/*` benchmark binaries (~3,400 lines)
- **Build**: Dropped the no-op `cpu-*` and `benchmarks` Cargo features

### Security
- **Time Sync**: Hardened the NTP client to connect the socket (source filtering), set and verify the originate timestamp to reject off-path/stale replies, and use the standard round-trip-corrected offset formula
- **Validation**: HTML-escaped externally-sourced fields (city name, USNO API values) in the USNO validation report to prevent markup injection
- **AI Insights**: Capped the per-request Ollama timeout so a long refresh interval can no longer make a single request block for many minutes
- **Dependencies**: Updated transitive `rand` 0.8.x usage to 0.8.6 to remediate GHSA-cq8v-f236-94qc / RUSTSEC-2026-0097 (supersedes Dependabot PR #55)
- **Dependencies**: Updated `rustls-webpki` to 0.103.13 (Dependabot PR #56)


## [0.4.0] - 2026-04-18

### Changed
- **CI**: Refreshed `github/codeql-action` pinned SHAs to the current `v4` tag and kept the drift gate aligned
- **CI**: Tightened Dependabot GitHub Actions updates to run daily with grouped maintainer-reviewed workflow pin PRs
- **CI**: Moved the Reykjavik USNO drift validation case off a seasonal boundary date, extracted USNO drift cases into `scripts/usno_drift_cases.sh` for easier maintenance, and hardened the workflow to fail closed if the case list cannot be loaded
- **Toolchain**: Replaced the 6-month Rust support window with an explicit `rust-version` contract: latest stable remains the active development target, the current release line supports stable Rust `1.91+`, and that floor may rise in a minor release when security, dependency compatibility, or maintainability require it
- **CLI**: Non-watch runs now honor saved time-sync settings and persist explicit `--city` / `--lat` / `--lon` location changes back to the user config
- **Validation**: Bounded USNO retry budgets so validation reuses the primary day fetch and surrounding-day probes fail fast during API outages
- **Dependencies**: Updated `clap` to 4.5.60 and `anyhow` to 1.0.102 (Dependabot PR #35)
- **Dependencies**: Updated `chrono` to 0.4.44 (supersedes Dependabot PR #36)

### Security
- **Dependencies**: Updated `quinn-proto` to 0.11.14 to remediate RustSec advisory `RUSTSEC-2026-0037`
- **Dependencies**: Updated `rustls-webpki` to 0.103.10 to remediate GHSA-pwjx-qhcg-rvj4 (supersedes Dependabot PR #43)
- **Dependencies**: Updated `rand` to 0.9.3 and `rustls-webpki` to 0.103.12 to remediate GitHub alerts `GHSA-cq8v-f236-94qc`, `GHSA-965h-392x-2mh5`, and `GHSA-xgp8-3hg3-c2mh`

## [0.3.3] - 2026-02-17

### Changed
- **Release**: Bumped crate version to `0.3.3`, which updates CLI/TUI application banners via `CARGO_PKG_VERSION`
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
- **CI**: Added scheduled/manual USNO drift validation workflow that runs fixed-city `--validate` checks with caution/fail/missing thresholds

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
