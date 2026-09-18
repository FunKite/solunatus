# Plan a night of stargazing or astrophotography

`--night` is available in v0.7.0. [Install or upgrade](../installation/README.md) with Cargo. The earlier `--tonight` spelling remains a compatibility alias with identical behavior, including when `--date` is supplied.

## Start with one question: when will it be dark without the Moon?

```bash
solunatus --city "Tucson" --night
```

The report includes:

- Evening golden hour and blue hour for foreground photography.
- Sunset, astronomical dusk, the following morning's astronomical dawn, and sunrise.
- The **first** moon-free dark window in the selected night, with its duration.
- Moon phase, illumination, and altitude at the labeled snapshot time.
- Planets above the horizon at that same instant, with altitude and azimuth.

The command runs offline, prints once, and exits. It does not enter watch mode or save configuration. It works without optional features, API keys, or an AI model.

## Take it to your observing site

Use an IANA timezone so the plan follows the local clock and daylight-saving rules:

```bash
solunatus --lat 36.24 --lon=-116.82 --tz America/Los_Angeles --night
```

Negative values can be supplied with `=` as shown above. City names use the built-in database; coordinates work anywhere. A city's dark interval says nothing about local light pollution.

## Plan a future night

```bash
solunatus --city "Tucson" --date 2026-10-10 --night
```

A plan always spans **local noon on the selected date to local noon the next day**. With no `--date`, it uses today's date at the observing location, including before noon. For the night that began yesterday, pass yesterday's date explicitly. DST transitions can make the interval 23 or 25 hours long.

The supported planning dates are astronomical years −0999 through 3000. This range does not imply uniform scientific accuracy; modern dates and mid-latitude locations are the strongest use case.

## Read the dark window correctly

“Moon-free” means the Sun is below −18° and the Moon is below the horizon, with a 15-minute buffer around moonrise/moonset. This uses the same darkness calculation as Solunatus's existing event timeline.

- The report shows the **first** qualifying interval, not a ranking of observing quality or a list of every interval.
- A full or bright Moon can leave no qualifying interval. At high latitudes, summer twilight can last all night.
- “No crossing” means a solar threshold does not cross during this plan; polar night can still be dark throughout it.
- Continuous darkness extending beyond the plan is clipped at the next local noon and labeled accordingly.
- Moon and planet positions are sampled at the midpoint of that first window. With no window, the snapshot uses an hour after astronomical dusk; if dusk is absent, it uses the plan's midpoint.
- “Above the horizon” describes geometry. It does not guarantee visibility; Neptune generally needs optical aid, and low objects may be obscured by terrain or haze.

Weather, seeing, transparency, terrain, and light pollution are not modeled. Compare the plan with local conditions before heading out.

## Save or script a plan

```bash
solunatus --city "Tucson" --date 2026-10-10 --night --json > night.json
```

The night-plan JSON is a separate schema from the regular `--json` snapshot. Timestamps use RFC 3339 with local UTC offsets. Missing events and absent windows are `null`; `moon_free_window_clipped` identifies a window whose natural end falls beyond the plan.

Important fields:

| Field | Meaning |
| --- | --- |
| `night_of`, `place`, `timezone` | Selected local date and observing location. |
| `interval.start`, `interval.end` | The exact local noon-to-noon boundaries. |
| `evening_golden_hour`, `evening_blue_hour` | Photography intervals, each with `start` and `end`. |
| `sunset`, `astronomical_dusk`, `astronomical_dawn`, `sunrise` | Solar crossings inside the plan. |
| `first_moon_free_window` | First qualifying interval, clipped to the plan. |
| `snapshot_at` | Time represented by the Moon and planet data. |
| `moon_illumination_percent`, `moon_altitude_degrees` | Moon lighting and position at the snapshot. |
| `planets_above_horizon` | Names, altitude, and azimuth (degrees clockwise from north). |

`--night` conflicts with `--watch`, `--calendar`, `--next`, and `--validate`. Use those modes separately.

## Keep the dashboard open at the telescope

```bash
SOLUNATUS_SKIP_TIME_SYNC=1 solunatus --city "Tucson" --no-save
```

Press `g` for the Sun/Moon altitude chart. Press `s` to enable red-text night mode or change location. Press `q` to quit. Screen brightness still matters for dark adaptation.

## Put a month on your calendar

```bash
solunatus --city "Tucson" --calendar \
  --calendar-start 2026-10-01 --calendar-end 2026-10-31 \
  --calendar-format ics --calendar-output tucson-october.ics
```

The ICS export includes daily rise/set events and quarter lunar phases. It does not export the night-plan report. The [calendar guide](calendar.md) describes the other export formats.

## README preview

The README image contains actual CLI output for a fixed date, not an illustration of predicted weather or a screenshot of another app. Recreate it with the existing Rust toolchain and Python standard library:

```bash
cargo build --locked
python3 scripts/render_night_demo.py --binary target/debug/solunatus
```

The script updates `docs/images/night-plan.svg`. Its text is XML-escaped, and it does not install or download any tools.
