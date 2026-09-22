# CLI Reference Guide

Complete documentation of all Solunatus command-line options.

## Basic Syntax

```bash
solunatus [OPTIONS]
```

All options are optional. Without arguments, Solunatus loads your saved location from `~/.solunatus.json` if available, or exits with instructions to supply `--city` or coordinates.

## Location Options

### `--city <CITY>`
Select a city from the built-in database.

```bash
solunatus --city "New York"
solunatus --city "Tokyo"
solunatus --city "London"
```

City names must match a database entry (case-insensitive). Fuzzy search is available in the interactive settings city picker, not in `--city`. Use coordinates for an unlisted observing site.

### `--lat <LAT>`
Latitude in decimal degrees (range: -90 to +90).

```bash
solunatus --lat 40.7128  # New York
solunatus --lat -33.8688  # Sydney
```

**Required with:** `--lon` and `--tz`

### `--lon <LON>`
Longitude in decimal degrees (range: -180 to +180).

```bash
solunatus --lon -74.0060  # New York
solunatus --lon 151.2093  # Sydney
```

**Required with:** `--lat` and `--tz`

### `--tz <TIMEZONE>`
Timezone in IANA format (e.g., `America/New_York`).

```bash
solunatus --lat 40.7128 --lon -74.0060 --tz America/New_York

solunatus --lat 35.6762 --lon 139.6503 --tz Asia/Tokyo
```

**Common timezones:**
- `America/New_York`
- `Europe/London`
- `Europe/Paris`
- `Asia/Tokyo`
- `Australia/Sydney`
- `Pacific/Auckland`

See [IANA Timezone Database](https://en.wikipedia.org/wiki/List_of_tz_database_time_zones) for complete list.

### `--date <DATE>`
Calculate for a specific date in `YYYY-MM-DD` format.

```bash
solunatus --city "Boston" --date 2025-12-25  # Christmas 2025

solunatus --lat 40.7128 --lon -74.0060 --tz America/New_York --date 1969-07-20  # Apollo 11 landing
```

Default: Today's date

A dated request prints a one-shot text report (or JSON with `--json`) rather than opening the live dashboard, which always follows the current time. `--date` conflicts with `--watch`.

**Night-plan/calendar range:** Astronomical years −0999 through 3000. Accuracy depends on the date and location.

## Output Options

### `--night` (since v0.7.0; alias `--tonight`)

Print an offline observing plan for local noon today (or `--date`) through noon tomorrow. Includes photography times, the first moon-free dark window, and a timed Moon/planet snapshot. Add `--json` for the night-plan schema.

```bash
solunatus --city "Tucson" --night
solunatus --city "Tucson" --date 2026-10-10 --night --json
```

Exits after printing and does not save settings. Conflicts with `--watch`, `--calendar`, `--next`, and `--validate`. See the [observing guide](observing.md) for date boundaries, polar cases, and interpreting the result.

### `--json`
Output in JSON format instead of text.

```bash
solunatus --city "Paris" --json

# Pipe to file
solunatus --city "Tokyo" --json > tokyo.json

# Parse with jq
solunatus --city "Sydney" --json | jq '.sun.events'
```

Useful for:
- Integration with other tools
- Scripting and automation
- Programmatic parsing

See [JSON Output Guide](json-output.md) for schema details.

### `--no-prompt`
Suppress interactive mode and output single snapshot.

```bash
solunatus --city "New York" --no-prompt
```

Useful for:
- Scripting
- Cron jobs
- Pipe to other programs

### `--no-save`
Disable saving settings to `~/.solunatus.json`. There is no `--save` flag; settings normally save through the existing configuration flow.

```bash
solunatus --city "Paris" --no-save
```

## Calendar Options

Generate astronomical calendars for date ranges.

### `--calendar`
Enable calendar generation mode.

```bash
solunatus --city "London" --calendar --calendar-start 2025-12-01 --calendar-end 2025-12-31
```

**Required with:** `--calendar-start` and `--calendar-end`

### `--calendar-start <DATE>`
Start date for calendar in `YYYY-MM-DD` format.

```bash
solunatus --city "Paris" --calendar --calendar-start 2025-01-01 --calendar-end 2025-01-31
```

### `--calendar-end <DATE>`
End date for calendar in `YYYY-MM-DD` format.

```bash
solunatus --city "Tokyo" --calendar --calendar-start 2025-06-01 --calendar-end 2025-06-30
```

### `--calendar-format <FORMAT>`
Output format for calendar: `html` or `json`.

```bash
# HTML calendar (viewable in browser)
solunatus --city "Boston" --calendar \
  --calendar-start 2025-12-01 \
  --calendar-end 2025-12-31 \
  --calendar-format html

# JSON calendar (machine-readable)
solunatus --city "Sydney" --calendar \
  --calendar-start 2025-03-01 \
  --calendar-end 2025-03-31 \
  --calendar-format json
```

Default: `html`

### `--calendar-output <PATH>`
Save calendar to file. If not specified, prints to stdout.

```bash
solunatus --city "Paris" --calendar \
  --calendar-start 2025-12-01 \
  --calendar-end 2025-12-31 \
  --calendar-format html \
  --calendar-output december.html

# View in browser
open december.html  # macOS
xdg-open december.html  # Linux
```

## AI Insights Options

Integrate with local Ollama for narrative summaries (optional).

### `--ai-insights`
Enable AI insights. Requires Ollama to be running.

```bash
solunatus --city "New York" --ai-insights
```

**Setup required:** See [AI Insights Guide](ai-insights.md)

Saved AI settings (server, model, refresh interval) from the dashboard's AI panel (`a`) apply automatically; each flag below overrides its saved value for one run. A saved "enabled" setting turns insights on in the interactive dashboard only; one-shot text and JSON output need `--ai-insights`.

### `--ai-server <URL>`
Ollama server address (default: saved setting, else `http://localhost:11434`).

```bash
solunatus --city "Boston" --ai-insights --ai-server "http://192.168.1.100:11434"
```

### `--ai-model <MODEL>`
LLM model to use for insights (default: saved setting, else `llama3.2:latest`).

```bash
solunatus --city "Tokyo" --ai-insights --ai-model "llama2"
```

**Common models:**
- `llama2` - Meta's Llama 2 (recommended)
- `llama3` - Meta's Llama 3
- `neural-chat` - Intel's Neural Chat
- Others: Check `ollama list`

### `--ai-refresh-minutes <MINUTES>`
How often to refresh AI insights (1-60 minutes).

```bash
solunatus --city "Paris" --ai-insights --ai-refresh-minutes 5
```

Default: saved setting, else 2 minutes

## General Options

### `--help`
Show help message.

```bash
solunatus --help
solunatus -h
```

### `--version`
Show version information.

```bash
solunatus --version
solunatus -V
```

## Example Commands

### Get sunrise/sunset for today (saved location)
```bash
solunatus
```

### Specific city
```bash
solunatus --city "San Francisco"
```

### Manual coordinates
```bash
solunatus --lat 51.5074 --lon -0.1278 --tz Europe/London
```

### Specific date
```bash
solunatus --city "Sydney" --date 2025-12-25
```

### JSON for scripting
```bash
solunatus --city "Tokyo" --json | jq '.sun.events'
```

### Calendar for December
```bash
solunatus --city "New York" --calendar \
  --calendar-start 2025-12-01 \
  --calendar-end 2025-12-31 \
  --calendar-format html \
  --calendar-output december.html
```

### With AI insights
```bash
solunatus --city "Paris" --ai-insights --ai-model "llama2"
```

### Silent mode (no prompt, just data)
```bash
solunatus --city "Boston" --no-prompt --json > boston_today.json
```

## Environment Variables

### `ASTROTIMES_SKIP_TIME_SYNC`
Skip system clock synchronization check at startup.

```bash
export ASTROTIMES_SKIP_TIME_SYNC=1
solunatus --city "New York"
```

Useful for:
- Testing without internet
- Faster startup
- Offline operation

## Configuration File

Settings are saved to `~/.solunatus.json`:

```json
{
  "lat": 40.7128,
  "lon": -74.0060,
  "tz": "America/New_York",
  "city": "New York"
}
```

Automatically loaded if no arguments specified.

## Common Workflows

### Cron Job: Daily sunrise reminder
```bash
#!/bin/bash
SUNRISE=$(solunatus --city "Boston" --no-prompt --json | jq -r '.events[0].time')
echo "Sunrise is at $SUNRISE"
```

### Generate astronomical yearbook
```bash
for month in {01..12}; do
  solunatus --city "London" --calendar \
    --calendar-start "2025-$month-01" \
    --calendar-end "2025-$month-31" \
    --calendar-format html \
    --calendar-output "2025_$month.html"
done
```

### Watch mode with custom refresh
```bash
solunatus --city "Tokyo"
# Then press [ to slow down or ] to speed up
```

### Get all astronomical data as JSON
```bash
solunatus --city "Sydney" --json > full_data.json
cat full_data.json | jq .
```

## Tips & Tricks

- Use `--no-prompt` for non-interactive scripts
- Use `--json` for piping to other tools
- Press `s` in watch mode to open settings menu
- Use city picker (press `c`) for fastest city selection
- Choose a frequently used location in watch-mode settings (`s`)
- Check timezone spelling: Use IANA format, not abbreviations

## Need Help?

- [Features Overview](README.md)
- [Interactive Mode Guide](interactive-mode.md)
- [Quick Start](../installation/quick-start.md)
- [Troubleshooting](../installation/troubleshooting.md)
