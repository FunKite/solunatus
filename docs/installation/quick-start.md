# Quick start

## 1. Install

With [Rust and Cargo](https://www.rust-lang.org/tools/install) installed:

```bash
cargo install --locked solunatus
```

No Rust toolchain? The [v0.6.1 release](https://github.com/FunKite/solunatus/releases/tag/v0.6.1) also provides Linux x86_64 and ARM64 archives. See the [installation guide](README.md) for verification and source builds.

## 2. Open the sky dashboard

```bash
solunatus --city "Tucson"
```

The dashboard opens in watch mode. Press `g` for the Sun/Moon altitude chart, `s` for settings and red-text night mode, `r` for reports, and `q` to quit.

Use `--no-prompt` for a single text snapshot or `--json` for structured output:

```bash
solunatus --city "Tucson" --no-prompt
solunatus --city "Tucson" --json
```

## 3. Use your observing site

```bash
solunatus --lat 36.24 --lon=-116.82 --tz America/Los_Angeles
```

City names must match an entry in the database. Use coordinates if your site is not listed. Supply the IANA timezone to get local times; without `--tz`, manual coordinates use UTC.

## 4. Find an evening event

```bash
solunatus --city "Tucson" --next golden-dusk-start --format human
solunatus --city "Tucson" --next astronomical-dusk --format iso
```

These commands print one event and exit without network access.

## 5. Try the new one-page night plan

The new `--tonight` command is on `main`, not in v0.6.1 yet:

```bash
git clone https://github.com/FunKite/solunatus.git
cd solunatus
cargo install --locked --path .
solunatus --city "Tucson" --tonight
```

It shows photography times, the first moon-free dark window, and a timed Moon/planet snapshot. [Plan a future night or export JSON →](../features/observing.md)

## Offline use and settings

Core calculations run locally. The dashboard normally checks network time; disable that with:

```bash
SOLUNATUS_SKIP_TIME_SYNC=1 solunatus --city "Tucson" --no-save
```

`--tonight` and `--next` skip the network-time check automatically. `--no-save` avoids saving settings to `~/.solunatus.json`.

## Next steps

- [Observing recipes](../features/observing.md)
- [Calendar export](../features/calendar.md)
- [CLI reference](../features/cli-reference.md)
- [Troubleshooting](troubleshooting.md)
- [Runnable Rust examples](../../examples/)
