# Solunatus v0.8.1 — Accuracy and reliability fixes

Release date: 2026-11-10

This release corrects lunar phase and seasonal times, fixes sun events near the international date line and rise/set searches on daylight-saving days, and makes the CLI and dashboard more forgiving. The public library API only gains additions, so it is a drop-in update for 0.8.0. The default features and Rust 1.91 minimum remain unchanged.

```bash
cargo install --locked solunatus --version 0.8.1 --force
solunatus --city "Tucson" --night
solunatus --city "Boston" --date 2026-12-25
```

## Library compatibility

0.8.1 is compatible with 0.8.0. New public helpers: `CityDatabase::find_qualified` selects a city with a state/country code or IANA timezone qualifier, and `tui::drafts` settings drafts report whether the current field takes text. Results change where 0.8.0 was wrong: lunar phase and season times, sun events in zones beside the date line, and rise/set events on 23- and 25-hour days. Library consumers can update to `solunatus = "0.8.1"` without code changes.

## Accuracy

- Lunar phase times use the complete Meeus chapter 49 series and are converted from Terrestrial Time to UTC. They were up to about six minutes off and now match Meeus' worked examples and published USNO times to the minute.
- Lunar phase and seasonal-event times use the published TT−UTC leap-second offsets from 1972 through June 2027. 2025 results were about five seconds early.
- Solar events belong to the requested civil date in places that keep a calendar a day away from their longitude's solar day, such as Apia, Nuku'alofa and Kiritimati.
- Moon and planet rise/set searches cover the local civil day, so events in the extra hour of a 25-hour day are no longer missed.

## CLI and dashboard

- `--next` no longer skips an occurrence still ahead today, including near daylight-saving changes.
- Negative coordinates and years work in the ordinary form (`--lon -74.0060`).
- `--lat` without `--lon` is an error; an ignored `--tz`, or coordinates beside `--city`, print a warning.
- Ambiguous city names warn and list alternatives; `--city` accepts qualifiers such as `"Portland, ME"` or `"Portland, America/New_York"`.
- Typing `d` in a Settings text field enters the letter instead of resetting every setting. Manually entered locations are saved, and Windows terminals no longer act twice on one key press.
- Failed NTP checks are cached for a minute instead of delaying every run, and `--calendar` no longer queries NTP.
- `SOLUNATUS_CONFIG_DIR` selects a separate settings directory.

## Security and validation

- Tokio is updated for RUSTSEC-2026-0386 (informational; Solunatus does not call the affected constructors).
- CodeQL action pins are refreshed to v4.39.3. Windows CI now covers default, all-feature and minimal builds.
- The published crate no longer includes GitHub workflows or workflow tests.

## Distribution

This release consists of the crates.io package, annotated Git tag, and these GitHub release notes. No prebuilt binaries are attached. Older 0.6.1 Linux archives do not include the night planner or these fixes.

See [CHANGELOG.md](https://github.com/FunKite/solunatus/blob/v0.8.1/CHANGELOG.md) for the full release history.
