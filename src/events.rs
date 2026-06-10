//! Event collection and organization.
//!
//! Collects astronomical events (solar and lunar) within a time window
//! and organizes them chronologically for display.

use chrono::{DateTime, Duration, TimeZone};
use chrono_tz::Tz;

use crate::astro::{Location, moon, sun};

#[derive(Clone, Copy)]
enum EventSource {
    Solar(sun::SolarEvent),
    Moon(moon::LunarEvent),
}

#[derive(Clone, Copy)]
struct EventDefinition {
    label: &'static str,
    source: EventSource,
}

const EVENT_DEFINITIONS: &[EventDefinition] = &[
    EventDefinition {
        label: "☀️ Solar noon",
        source: EventSource::Solar(sun::SolarEvent::SolarNoon),
    },
    EventDefinition {
        label: "📸 Golden dusk beg",
        source: EventSource::Solar(sun::SolarEvent::GoldenDuskStart),
    },
    EventDefinition {
        label: "🌇 Sunset",
        source: EventSource::Solar(sun::SolarEvent::Sunset),
    },
    EventDefinition {
        label: "📸 Golden dusk end",
        source: EventSource::Solar(sun::SolarEvent::GoldenDuskEnd),
    },
    EventDefinition {
        label: "🌕 Moonrise",
        source: EventSource::Moon(moon::LunarEvent::Moonrise),
    },
    EventDefinition {
        label: "🌆 Civil dusk",
        source: EventSource::Solar(sun::SolarEvent::CivilDusk),
    },
    EventDefinition {
        label: "⛵ Nautical dusk",
        source: EventSource::Solar(sun::SolarEvent::NauticalDusk),
    },
    EventDefinition {
        label: "🌠 Astro dusk",
        source: EventSource::Solar(sun::SolarEvent::AstronomicalDusk),
    },
    EventDefinition {
        label: "🔭 Astro dawn",
        source: EventSource::Solar(sun::SolarEvent::AstronomicalDawn),
    },
    EventDefinition {
        label: "⚓ Nautical dawn",
        source: EventSource::Solar(sun::SolarEvent::NauticalDawn),
    },
    EventDefinition {
        label: "🏙️ Civil dawn",
        source: EventSource::Solar(sun::SolarEvent::CivilDawn),
    },
    EventDefinition {
        label: "📸 Golden dawn beg",
        source: EventSource::Solar(sun::SolarEvent::GoldenDawnStart),
    },
    EventDefinition {
        label: "🌅 Sunrise",
        source: EventSource::Solar(sun::SolarEvent::Sunrise),
    },
    EventDefinition {
        label: "📸 Golden dawn end",
        source: EventSource::Solar(sun::SolarEvent::GoldenDawnEnd),
    },
    EventDefinition {
        label: "🌑 Moonset",
        source: EventSource::Moon(moon::LunarEvent::Moonset),
    },
];

/// Collect sun and moon events that fall within a symmetrical time window around the reference.
pub fn collect_events_within_window(
    location: &Location,
    reference: &DateTime<Tz>,
    window: Duration,
) -> Vec<(DateTime<Tz>, &'static str)> {
    let max_delta = window.num_seconds().abs();
    let mut events = Vec::new();

    for offset in -1..=1 {
        let shifted = if offset == 0 {
            *reference
        } else {
            reference
                .checked_add_signed(Duration::days(offset as i64))
                .unwrap_or(*reference)
        };

        for definition in EVENT_DEFINITIONS {
            let maybe_time = match definition.source {
                EventSource::Solar(event) => sun::solar_event_time(location, &shifted, event),
                EventSource::Moon(event) => moon::lunar_event_time(location, &shifted, event),
            };

            if let Some(event_time) = maybe_time {
                let delta = event_time.signed_duration_since(reference);
                if delta.num_seconds().abs() <= max_delta {
                    events.push((event_time, definition.label));
                }
            }
        }
    }

    // Extract astro dawn and dusk times from collected events
    let astro_dawn = events
        .iter()
        .find(|(_, label)| label.contains("Astro dawn"))
        .map(|(dt, _)| *dt);
    let astro_dusk = events
        .iter()
        .find(|(_, label)| label.contains("Astro dusk"))
        .map(|(dt, _)| *dt);

    // Add dark window events using the actual astro twilight times
    let dark_windows = calculate_dark_windows(location, reference, window, astro_dawn, astro_dusk);
    for (dt, label) in dark_windows {
        events.push((dt, label));
    }

    events.sort_by_key(|(dt, _)| *dt);
    events.dedup_by(|a, b| a.0 == b.0 && a.1 == b.1);
    events
}

/// Check if the moon is sufficiently dark with buffer for moon glow.
///
/// Returns true if:
/// - Moon is below horizon now
/// - Moon was below horizon 15 minutes ago (glow has faded)
/// - Moon will be below horizon 15 minutes from now (glow hasn't started)
fn is_moon_sufficiently_dark(
    location: &Location,
    time: &DateTime<Tz>,
    buffer_minutes: i64,
) -> bool {
    // Check moon is below horizon now
    let moon_now = moon::lunar_position(location, time);
    if moon_now.altitude >= 0.0 {
        return false;
    }

    // Check moon was below horizon at buffer time ago (glow has faded)
    if let Some(time_past) = time.checked_sub_signed(Duration::minutes(buffer_minutes)) {
        let moon_past = moon::lunar_position(location, &time_past);
        if moon_past.altitude >= 0.0 {
            return false; // Moon set less than buffer time ago
        }
    }

    // Check moon will be below horizon at buffer time from now (glow hasn't appeared)
    if let Some(time_future) = time.checked_add_signed(Duration::minutes(buffer_minutes)) {
        let moon_future = moon::lunar_position(location, &time_future);
        if moon_future.altitude >= 0.0 {
            return false; // Moon will rise within buffer time
        }
    }

    true
}

/// Calculate dark window periods when observing conditions are optimal.
///
/// Implements the DSD (Deep Sky Darkness) standard used by astrophotographers:
/// - Sun is below astronomical twilight (-18°)
/// - Moon is below the horizon with a 15-minute buffer to account for moon glow
///
/// The 15-minute buffer after moonset and before moonrise accounts for atmospheric
/// reflection and scattering of moonlight, which can brighten the sky even when
/// the moon is technically below the horizon.
///
/// Reference: APT (Astro Photography Tool) Deep Sky Darkness Calculator
fn calculate_dark_windows(
    location: &Location,
    reference: &DateTime<Tz>,
    window: Duration,
    astro_dawn: Option<DateTime<Tz>>,
    astro_dusk: Option<DateTime<Tz>>,
) -> Vec<(DateTime<Tz>, &'static str)> {
    const MOON_GLOW_BUFFER_MINUTES: i64 = 15; // Buffer for moon glow to fade
    const SAMPLE_INTERVAL_MINUTES: i64 = 1; // 1-minute sampling for precision

    let start_time = reference.checked_sub_signed(window).unwrap_or(*reference);
    let end_time = reference.checked_add_signed(window).unwrap_or(*reference);

    let mut events = Vec::new();
    let mut in_dark_window = false;
    let mut current_time = start_time;
    let mut prev_time = start_time;
    let mut first_sample = true;

    while current_time <= end_time {
        // Check if sun is below astronomical twilight
        let sun_pos = sun::solar_position(location, &current_time);
        let sun_dark = sun_pos.altitude < -18.0;

        // Check moon conditions (DSD standard: moon below horizon with glow buffer)
        let moon_dark =
            is_moon_sufficiently_dark(location, &current_time, MOON_GLOW_BUFFER_MINUTES);

        let is_dark = sun_dark && moon_dark;

        // Detect transitions (but skip recording if it's the first sample - that's a boundary artifact)
        if is_dark && !in_dark_window {
            if !first_sample {
                // Check if moon was already suitable at prev_time
                let prev_moon_dark =
                    is_moon_sufficiently_dark(location, &prev_time, MOON_GLOW_BUFFER_MINUTES);

                // If moon conditions unchanged, use the exact astro dusk time from events
                if prev_moon_dark && moon_dark {
                    // Moon not limiting - use the provided astronomical dusk time
                    if let Some(ref dusk_time) = astro_dusk {
                        // Verify it's near our transition point
                        let time_diff = dusk_time
                            .signed_duration_since(prev_time)
                            .num_seconds()
                            .abs();
                        if time_diff <= 120 {
                            events.push((*dusk_time, "🌌 Dark win start"));
                            in_dark_window = true;
                            first_sample = false;
                            prev_time = current_time;
                            current_time = current_time
                                .checked_add_signed(Duration::minutes(SAMPLE_INTERVAL_MINUTES))
                                .unwrap_or(end_time);
                            continue;
                        }
                    }
                }

                // Otherwise refine with bisection (moon was the limiting factor)
                let refined_time =
                    refine_dark_window_transition(location, &prev_time, &current_time, true);
                events.push((refined_time, "🌌 Dark win start"));
            }
            in_dark_window = true;
        } else if !is_dark && in_dark_window {
            if !first_sample {
                // If moon conditions unchanged, use the exact astro dawn time from events
                if moon_dark {
                    // Moon still not limiting - use the provided astronomical dawn time
                    if let Some(ref dawn_time) = astro_dawn {
                        let time_diff = dawn_time
                            .signed_duration_since(prev_time)
                            .num_seconds()
                            .abs();
                        if time_diff <= 120 {
                            events.push((*dawn_time, "🌄 Dark win end"));
                            in_dark_window = false;
                            first_sample = false;
                            prev_time = current_time;
                            current_time = current_time
                                .checked_add_signed(Duration::minutes(SAMPLE_INTERVAL_MINUTES))
                                .unwrap_or(end_time);
                            continue;
                        }
                    }
                }

                // Use bisection for moon-limited transitions
                let refined_time =
                    refine_dark_window_transition(location, &prev_time, &current_time, false);
                events.push((refined_time, "🌄 Dark win end"));
            }
            in_dark_window = false;
        }

        first_sample = false;
        prev_time = current_time;
        current_time = current_time
            .checked_add_signed(Duration::minutes(SAMPLE_INTERVAL_MINUTES))
            .unwrap_or(end_time);
    }

    // Don't record boundary artifacts - only actual transitions within the window
    events
}

/// Find the next dark-sky window at or after `reference`.
///
/// A dark-sky window is a period where the sun is below astronomical twilight
/// (-18°) and the moon is below the horizon, using the same 15-minute moon-glow
/// buffer as the events timeline (the DSD standard used by astrophotographers).
///
/// Scans forward up to 36 hours. If `reference` is already inside a dark window,
/// the returned window starts at `reference`. The end time is `None` when the
/// window extends beyond the scan horizon (possible at polar latitudes).
pub fn next_dark_window(
    location: &Location,
    reference: &DateTime<Tz>,
) -> Option<(DateTime<Tz>, Option<DateTime<Tz>>)> {
    const MOON_GLOW_BUFFER_MINUTES: i64 = 15;
    const STEP_MINUTES: i64 = 5;
    const HORIZON_HOURS: i64 = 36;

    let is_dark = |time: &DateTime<Tz>| {
        sun::solar_position(location, time).altitude < -18.0
            && is_moon_sufficiently_dark(location, time, MOON_GLOW_BUFFER_MINUTES)
    };

    let scan_end = reference.checked_add_signed(Duration::hours(HORIZON_HOURS))?;

    let mut prev = *reference;
    let mut prev_dark = is_dark(&prev);
    let mut window_start: Option<DateTime<Tz>> = if prev_dark { Some(prev) } else { None };

    let mut current = prev;
    while current < scan_end {
        current = current.checked_add_signed(Duration::minutes(STEP_MINUTES))?;
        let dark = is_dark(&current);

        if dark && !prev_dark && window_start.is_none() {
            window_start = Some(refine_dark_window_transition(
                location, &prev, &current, true,
            ));
        } else if !dark
            && prev_dark
            && let Some(start) = window_start
        {
            let end = refine_dark_window_transition(location, &prev, &current, false);
            return Some((start, Some(end)));
        }

        prev = current;
        prev_dark = dark;
    }

    window_start.map(|start| (start, None))
}

/// Refine dark window transition time using bisection
fn refine_dark_window_transition(
    location: &Location,
    start: &DateTime<Tz>,
    end: &DateTime<Tz>,
    looking_for_start: bool,
) -> DateTime<Tz> {
    const MOON_GLOW_BUFFER_MINUTES: i64 = 15; // Must match buffer in calculate_dark_windows
    const TOLERANCE_SECONDS: i64 = 5; // 5-second precision

    let mut left = *start;
    let mut right = *end;

    while right.signed_duration_since(left).num_seconds() > TOLERANCE_SECONDS {
        let mid_seconds = (left.timestamp() + right.timestamp()) / 2;
        let mid = left.timezone().timestamp_opt(mid_seconds, 0).unwrap();

        let sun_pos = sun::solar_position(location, &mid);
        let sun_dark = sun_pos.altitude < -18.0;

        let moon_dark = is_moon_sufficiently_dark(location, &mid, MOON_GLOW_BUFFER_MINUTES);

        let is_dark = sun_dark && moon_dark;

        if looking_for_start {
            // Looking for when it becomes dark
            if is_dark {
                right = mid; // Transition is before mid
            } else {
                left = mid; // Transition is after mid
            }
        } else {
            // Looking for when it stops being dark
            if is_dark {
                left = mid; // Transition is after mid
            } else {
                right = mid; // Transition is before mid
            }
        }
    }

    // Return the midpoint of the final interval
    let mid_seconds = (left.timestamp() + right.timestamp()) / 2;
    left.timezone().timestamp_opt(mid_seconds, 0).unwrap()
}
