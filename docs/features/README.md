# Solunatus Features

Comprehensive guide to all features and capabilities of Solunatus.

## Core Astronomical Data

### Solar Events
- **Sunrise & Sunset** - Time when the sun's upper limb crosses the horizon
- **Solar Noon** - Highest point of the sun in the sky for the day
- **Twilight Times**:
  - Civil Twilight (6° below horizon) - Best for outdoor visibility
  - Nautical Twilight (12° below horizon) - Horizon visible to mariners
  - Astronomical Twilight (18° below horizon) - When sky becomes completely dark

### Solar Position (Real-time)
- **Altitude** - How high the sun is above the horizon (in degrees)
- **Azimuth** - Direction from north (N, NE, E, SE, S, SW, W, NW)
- Updates continuously in watch mode

### Moon Events
- **Moonrise & Moonset** - Times when moon crosses the horizon
- **Moon Transit** - Highest point of the moon in the sky
- Accurate to within ±3 minutes at mid-latitudes

### Moon Details
- **Phase** - Current lunar phase with name, emoji, and illumination percentage
- **Phase Age** - Days since new moon (0-29.5 day lunar cycle)
- **Distance** - Current distance from Earth in kilometers
- **Angular Size** - Apparent size of the moon in arcminutes
- **Size Category** - Classification from "Near Perigee" to "Near Apogee"
- **Maximum Altitude** - Highest point the moon reaches today

### Lunar Phases Calendar
Monthly calendar showing exact times of:
- New Moon
- First Quarter
- Full Moon
- Last Quarter

## Output Modes

### Night Plan (current source)

`solunatus --city "Tucson" --night` prints photography times, the first moon-free dark window, and a timed Moon/planet snapshot. Add `--date` to plan ahead or `--json` to export it. Available since v0.7.0, with `--tonight` retained as an alias. See the [observing guide](observing.md) for interpretation.

### Interactive Watch Mode (Default)
Live-updating terminal display with:
- Real-time clock
- Current astronomical data
- Upcoming events for the next 12 hours
- Interactive keyboard controls
- Configurable refresh rates

**Keyboard Controls:**
- `q` - Quit
- `s` - Open Settings menu (location, time sync, display, night mode, AI)
- `r` - Open Reports menu (calendar, USNO validation, benchmarks)
- `f` - Manually refresh AI insights (if enabled)

See [Interactive Mode Guide](interactive-mode.md) for details.

### Text Output
Single snapshot of astronomical data with all events and positions.

```bash
solunatus --city "New York" --no-prompt
```

### JSON Output
Machine-readable format for integration with other tools and scripts.

```bash
solunatus --city "New York" --json
```

See [JSON Output Guide](json-output.md) for schema details.

### Calendar Generation
Generate astronomical data tables for date ranges.

```bash
solunatus --city "Paris" \
  --calendar \
  --calendar-start 2025-12-01 \
  --calendar-end 2025-12-31 \
  --calendar-format html
```

Supported formats:
- **HTML** - Interactive calendar for viewing in a browser
- **JSON** - Machine-readable daily data

See [Calendar Guide](calendar.md) for examples.

## Location Handling

### City Database
- **570+ cities** worldwide
- Fuzzy search by city name
- Pre-configured timezone for each city
- Updated city picker with search in watch mode

```bash
solunatus --city "New York"
solunatus --city "Los Angeles"
solunatus --city "Tokyo"
```

### Manual Coordinates
Specify latitude, longitude, and timezone explicitly:

```bash
solunatus --lat 40.7128 --lon -74.0060 --tz America/New_York
```

### Configuration Persistence
Save your preferred location:

```bash
# In watch mode, press 's' to open settings, then save from there
# Or use
solunatus --city "New York" --save
```

Settings saved to `~/.solunatus.json`:

```json
{
  "lat": 40.7128,
  "lon": -74.0060,
  "tz": "America/New_York",
  "city": "New York"
}
```

## Advanced Features

### AI-Powered Insights (Optional)
Get AI-generated narrative summaries of astronomical events using a local Ollama instance.

```bash
solunatus --ai-insights --ai-model "llama2"
```

**Features:**
- Narrative descriptions of current sky conditions
- Interpretive summaries of interesting events
- Configurable refresh interval (1-60 minutes)
- Works in watch mode and text output

See [AI Insights Guide](ai-insights.md) for setup and configuration.

### System Clock Synchronization
Verify your system clock accuracy against NTP servers:

```bash
# Automatic on startup
solunatus --city "New York"
```

- Checks against Google's NTP server
- Warns if clock is significantly off
- Updates every 15 minutes in watch mode
- Helpful for accurate astronomical calculations

### Date-Specific Calculations
Calculate events for any date:

```bash
# Future date
solunatus --city "Boston" --date 2025-12-25

# Past date
solunatus --city "Sydney" --date 2024-07-20
```

## Command-Line Interface (CLI)

### Basic Syntax

```bash
solunatus [OPTIONS]
```

### Complete Options Reference

See [CLI Reference Guide](cli-reference.md) for complete documentation of all command-line flags and options.

## Calculation Standards

### Accuracy Specifications

All calculations follow U.S. Naval Observatory (USNO) standards for:
- Standardized, reproducible results
- Alignment with celestial navigation conventions
- Consistency with maritime and aviation almanacs

**Typical Accuracy:**
- Solar events (sunrise/sunset/twilight): ±1-3 minutes
- Lunar events (moonrise/moonset): ±3 minutes at mid-latitudes
- Lunar phase times: ±2 minutes

Differences from other sources are usually due to different atmospheric assumptions or calculation methods.

### Calculation Methods

- **Solar Position:** NOAA solar calculation algorithms
- **Lunar Position:** Topocentric model accounting for Earth's flattening and parallax
- **Lunar Phases:** Meeus "Phases of the Moon" algorithm from "Astronomical Algorithms"
- **Rise/Set Times:** Bisection root-finding with atmospheric refraction corrections

## Feature Comparison

| Feature | CLI | Library | TUI |
|---------|-----|---------|-----|
| Sunrise/Sunset | ✓ | ✓ | ✓ |
| Twilight Times | ✓ | ✓ | ✓ |
| Current Position | ✓ | ✓ | ✓ |
| Moonrise/Moonset | ✓ | ✓ | ✓ |
| Moon Phase | ✓ | ✓ | ✓ |
| Calendar Generation | ✓ | ✓ | ✓ |
| JSON Output | ✓ | ✓ | - |
| Watch Mode | - | - | ✓ |
| AI Insights | ✓ | - | ✓ |
| City Database | ✓ | ✓ | ✓ |

## Next Steps

- **[CLI Reference](cli-reference.md)** - Complete command options
- **[Interactive Mode Guide](interactive-mode.md)** - Master the TUI
- **[JSON Output Guide](json-output.md)** - Format and examples
- **[Calendar Guide](calendar.md)** - Date range calculations
- **[AI Insights Guide](ai-insights.md)** - Setup and configuration
- **[City Database Guide](city-database.md)** - Available locations
- **[Examples](../../examples/)** - Code samples
