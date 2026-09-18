# Solunatus v0.7.0 — Plan any night

Release date: 2026-09-18

Plan an observing session with `--night`: golden and blue hour, sunset, astronomical darkness, sunrise, the first moon-free dark window, and a timed Moon/planet snapshot. The report runs offline, prints once, and exits. Use `--json` for structured output.

```bash
cargo install --locked solunatus --version 0.7.0 --force
solunatus --city "Tucson" --night
solunatus --city "Tucson" --date 2026-10-10 --night --json
```

## Naming and dates

`--night` replaces the development name `--tonight`; `--tonight` remains a compatibility alias. Both cover local noon on the selected date through local noon the next day. Without `--date`, the date is today at the observing location. Daylight-saving transitions, the date line, and polar conditions are handled. Weather, terrain, and light pollution are not modeled.

## Other changes

- The dashboard title bar and `--version` report 0.7.0 from the package version.
- Minute-precision dark-window durations agree with their displayed endpoints; JSON preserves exact timestamps.
- ICS text normalizes CRLF and bare-CR line endings and preserves UTF-8 folding.
- Dependency security fixes include rustls, rustls-webpki, and lru; the changelog records the full dependency and CI maintenance since 0.6.1.
- Public library documentation is enforced with `deny(missing_docs)`. docs.rs builds all features; CI validates documentation with all features and without optional features.
- README, observing guide, CLI reference, installation instructions, and the actual-output preview now describe the published night planner.

## Distribution and compatibility

Rust 1.91 or newer is required; latest stable is recommended. Default features remain unchanged. No dependencies were added for the planner rename or this release preparation. Library users can set `solunatus = "0.7.0"`.

This release contains the crates.io package, Git tag, and GitHub release notes. No prebuilt binaries are attached; the older 0.6.1 Linux downloads do not include the night planner.

See [CHANGELOG.md](https://github.com/FunKite/solunatus/blob/v0.7.0/CHANGELOG.md) for the full release history.
