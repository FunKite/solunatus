# Interactive Watch Mode Guide

Master the Solunatus interactive terminal interface.

## Starting Watch Mode

```bash
# Default - starts in watch mode
solunatus --city "New York"

# Explicitly disable watch mode
solunatus --city "New York" --no-prompt
```

## Display Layout

The watch mode display shows:

```
🌅 Solunatus — Sunrise, Sunset, Moonrise, Moonset
— Location & Date —
📍 Location: New York, US
📅 Date: 2025-10-22 14:30:45 EDT  ⌚ Zone: America/New_York (UTC-5)

— Events —
06:22:15  🌅  Sunrise         8h 8m ago        (*next*)
...

— Position —
☀️  Sun:  Alt 45.3°, Az 180° S
🌕 Moon: Alt -12.5°, Az 270° W

— Moon —
🌕 Phase: Waning Gibbous (Age 20.3 days)
💡 Fraction Illum.: 87%
🔭 Apparent size: 31.2' (Average)

— Lunar Phases —
🌑 New Moon:        2025-11-01 13:47
🌓 First Quarter:   2025-11-08 18:25
🌕 Full Moon:       2025-11-15 13:28
🌗 Last Quarter:    2025-11-22 23:11
```

## Keyboard Controls

| Key | Action |
|-----|--------|
| `q` | Quit Solunatus |
| `s` | Open Settings menu (location, time sync, display sections, night mode, AI) |
| `r` | Open Reports menu (calendar generation, USNO validation, benchmarks) |
| `f` | Manually refresh AI insights (if enabled) |

### Additional Controls

- **Arrow keys** - Navigate in city picker
- **Typing** - Search in city picker
- **Enter** - Select in city picker
- **Esc** - Cancel city picker

## Features

### Real-Time Updates

The display updates automatically at fixed intervals:
- **Clock:** Every second
- **Sun/Moon positions:** Every 5 seconds
- **Moon data:** Every 10 minutes
- **Lunar phases:** Daily (at midnight local time)
- **Time sync check:** Every 30 minutes

Refresh rates are optimized to minimize CPU usage while keeping data current. These intervals are fixed and not user-adjustable.

### Night Mode

Toggle night mode from the Settings menu (press `s`):
- Text changes to red
- Preserves night vision
- Useful for outdoor astronomy
- Setting is saved in configuration

### Settings Menu (Press `s`)

The Settings menu allows you to configure:

#### Location
- **City mode:** Select from 570+ cities with fuzzy search
- **Manual mode:** Enter coordinates (latitude, longitude, timezone)

#### Time Sync
- Enable/disable system clock synchronization
- Configure NTP server (default: time.google.com)

#### Display Sections
- Toggle visibility of different panels (Location, Events, Positions, Moon, Lunar Phases)

#### Night Mode
- Enable red text for preserving night vision

#### AI Insights
- Configure Ollama integration (if installed)
- Set model and refresh interval

#### Save Configuration

Settings are automatically saved to `~/.solunatus.json`:

```json
{
  "lat": 40.7128,
  "lon": -74.0060,
  "tz": "America/New_York",
  "city": "New York"
}
```

Automatically loaded next time you run Solunatus.

### Reports Menu (Press `r`)

The Reports menu provides access to:

#### Calendar Generator
1. Select date range (start and end dates)
2. Choose format (HTML or JSON)
3. Optionally specify output file
4. Calendar generates with daily sunrise, sunset, moonrise, moonset, and lunar phases

**Output formats:**
- **HTML** - Viewable in web browser with formatted tables
- **JSON** - Machine-readable format for integration

#### USNO Validation
- Compare Solunatus calculations against U.S. Naval Observatory data
- Generates accuracy report showing differences for each event type
- Helps verify astronomical calculation accuracy

#### Performance Benchmark
- Test calculation speed across all cities in the database
- Reports timing statistics and performance metrics

## Event Indicators

### Next Event Marker

`(*next*)` indicates the upcoming astronomical event:

```
06:22:15  🌅  Sunrise         8h 8m ago        (*next*)
```

This helps you quickly identify what's coming.

### Event Types

- 🌅 Sunrise / 🌇 Sunset - Sun at horizon
- ☀️ Solar Noon - Sun at highest point
- 🏙️ Civil Twilight - 6° below horizon (visible outdoors)
- ⚓ Nautical Twilight - 12° below horizon (horizon visible)
- 🔭 Astronomical Twilight - 18° below horizon (completely dark)
- 🌑 Moonrise / Moonset - Moon at horizon

## Time Display

### Time Formats

- **Event times:** HH:MM:SS (24-hour format)
- **Durations:** "HH:MM ago" or "HH:MM from now"
- **Date:** Three-letter month abbreviation (MMM)

### Time Sync

Shows if system clock is synchronized with NTP servers:
- ✓ Synced (within ±1 second)
- ⚠️ Drift (minor offset)
- ✗ Unsynchronized (significant offset)

Auto-checks every 30 minutes. Successful results are cached for 30 minutes; failed queries are cached for one minute so repeated CLI invocations do not repeatedly wait on network timeouts. Enabling time sync or changing the server schedules a check on the next dashboard tick, honoring that cache. Saving other settings does not reset the check timer.

## Position Display

### Sun Position
```
☀️ Sun: Alt 45.3°, Az 180° S
```
- **Altitude:** Height above horizon (-90° to +90°)
- **Azimuth:** Compass direction (N, NE, E, SE, S, SW, W, NW)

### Moon Position
```
🌕 Moon: Alt -12.5°, Az 270° W
```
- **Altitude:** Can be negative (below horizon)
- **Azimuth:** Cardinal direction

## Moon Display

### Phase Information
```
🌕 Phase: Waning Gibbous (Age 20.3 days)
```
- **Phase name:** Current lunar phase
- **Age:** Days since new moon (0-29.5)
- **Emoji:** Visual representation of phase

### Illumination
```
💡 Fraction Illum.: 87%
```
- Percentage of moon's surface illuminated
- 0% = New Moon, 100% = Full Moon

### Apparent Size
```
🔭 Apparent size: 31.2' (Average)
```
- **Arcminutes:** Angular size in the sky
- **Category:** Near Perigee, Larger, Average, Smaller, Near Apogee

## Lunar Phases Calendar

Shows next lunar phase events:

```
🌑 New Moon:        2025-11-01 13:47
🌓 First Quarter:   2025-11-08 18:25
🌕 Full Moon:       2025-11-15 13:28
🌗 Last Quarter:    2025-11-22 23:11
```

Updated daily to show current month's phases.

## Terminal Sizing

Watch mode adapts to your terminal:
- Minimum recommended: 80×24 characters
- Automatically wraps controls
- Optimizes layout for available space

**Tip:** Make terminal full screen for best experience.

## Configuration During Watch Mode

All settings can be configured while running:

- **Settings menu:** Press `s` to configure location, time sync, display, night mode, and AI
- **Reports menu:** Press `r` to generate calendars, run validations, or benchmarks
- **AI refresh:** Press `f` to manually refresh AI insights (if enabled)

Changes take effect immediately and are automatically saved to configuration file.

## Tips & Tricks

1. **Quick location switch:** Press `s` → select City mode → search and select
2. **Find nearby cities:** Use Settings menu city picker with fuzzy search
3. **Save multiple setups:** Manually edit `~/.solunatus.json`
4. **Enable night mode:** Press `s` → toggle Night Mode setting
5. **Check accuracy:** Press `r` → USNO Validation to compare with Naval Observatory data

## Troubleshooting

### Display looks corrupted
- Resize terminal window
- Ensure terminal supports Unicode characters
- Try different terminal emulator

### Events show strange times
- Verify timezone is correct (press `s` to check in Settings)
- Check system clock synchronization status in display header

### Settings menu doesn't respond
- Press Esc to cancel and return to main display
- Use arrow keys or Tab to navigate between fields

## Next Steps

- **[CLI Reference](cli-reference.md)** - All command-line options
- **[Features Overview](README.md)** - All features
- **[Calendar Guide](calendar.md)** - Generate astronomical calendars
- **[AI Insights](ai-insights.md)** - Use AI for interpretations
