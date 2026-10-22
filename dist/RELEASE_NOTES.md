# Solunatus v0.8.0 — Reliability and security

Release date: 2026-10-22

This release preserves saved dashboard settings, protects unreadable configuration, corrects lunar-age and date reporting, and updates transport and certificate-trust dependencies. The default features and Rust 1.91 minimum remain unchanged.

```bash
cargo install --locked solunatus --version 0.8.0 --force
solunatus --city "Tucson" --night
solunatus --city "Boston" --date 2026-12-25
```

## Library compatibility

0.8.0 includes public API changes from the unreleased work after 0.7.0:

- `AiConfig::merge_with_saved` is removed. Use `AiConfig::from_args_and_saved` to construct AI configuration from CLI arguments and optional saved settings; `AiConfig::from_args` remains available for CLI-only defaults.
- The public `Args::ai_server`, `Args::ai_model`, and `Args::ai_refresh_minutes` fields are now `Option` values so an omitted flag can be distinguished from an explicit override.
- New public helpers `astro::moon::lunar_age_days` and `astro::moon::lunar_phases_near` expose the lunar-age and nearby-phase calculations used by the CLI.

Library consumers should update to `solunatus = "0.8.0"` and adjust callers of the changed APIs. CLI flags remain compatible; the ineffective `--strict` flag is deprecated and warns when used.

## Correctness and configuration

- Saved AI server, model, refresh interval, and enabled state survive dashboard restarts. Explicit CLI flags still take precedence.
- Unreadable configuration is reported and preserved, including when applying TUI settings. Settings writes use private `0600` files on Unix and atomic replacement.
- Lunar age uses the preceding Meeus new moon, including months after a February with no new moon. Nearby phase lists include preceding and following phases.
- `--date` produces a one-shot text report without requiring `--json`; `--watch --date` is rejected. Dark-sky dates and displayed durations are consistent.
- The NTP cache tracks the requested server even when a fallback answers, preserving its 30-minute rate limit.
- USNO validation compares only published USNO events rather than treating golden-hour events as missing.

## Security and validation

- Updated `quinn-proto`, `rustls`, `rustls-webpki`, and `webpki-roots` address the advisories listed in the changelog, including distrusted emSign intermediate keys and the affected root trust entry.
- Workflow actions retain full commit SHA pins and live pin-drift checks. CodeQL is updated to 4.39.1.
- Public library documentation remains enforced with `deny(missing_docs)`. docs.rs builds all features; CI validates all-feature and minimal-feature documentation, default/all/minimal tests, and the declared MSRV.

## Distribution

This release consists of the crates.io package, annotated Git tag, and these GitHub release notes. No prebuilt binaries are attached. Older 0.6.1 Linux archives do not include the night planner or these fixes.

See [CHANGELOG.md](https://github.com/FunKite/solunatus/blob/v0.8.0/CHANGELOG.md) for the full release history.
