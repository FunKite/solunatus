//! Cached data types for the TUI.
//!
//! These types cache expensive astronomical calculations to avoid
//! recomputation on every frame render.

use crate::astro::moon::LunarPosition;
use crate::astro::planets::{Planet, PlanetEvent, PlanetPosition};
use crate::astro::sun::SolarPosition;
use crate::astro::{Location, moon, planets, sun};
use chrono::DateTime;
use chrono_tz::Tz;

// ============================================================================
// Cached Events
// ============================================================================

/// Cached astronomical events for the event window.
#[derive(Debug, Clone)]
pub struct CachedEvents {
    /// Reference time when this cache was generated
    pub reference: DateTime<Tz>,
    /// List of (timestamp, event_name) tuples
    pub entries: Vec<(DateTime<Tz>, &'static str)>,
}

// ============================================================================
// Cached Positions
// ============================================================================

/// Cached sun and moon positions.
#[derive(Debug, Clone, Copy)]
pub struct CachedPositions {
    /// Timestamp when positions were calculated
    pub timestamp: DateTime<Tz>,
    /// Sun position data
    pub sun: SolarPosition,
    /// Moon position data
    pub moon: LunarPosition,
}

impl CachedPositions {
    /// Create new cached positions for the given location and time.
    pub fn new(location: &Location, timestamp: &DateTime<Tz>) -> Self {
        Self {
            timestamp: *timestamp,
            sun: sun::solar_position(location, timestamp),
            moon: moon::lunar_position(location, timestamp),
        }
    }
}

// ============================================================================
// Cached Planets
// ============================================================================

/// Position and rise/set times for one planet.
#[derive(Debug, Clone, Copy)]
pub struct CachedPlanet {
    /// Which planet this entry describes
    pub planet: Planet,
    /// Apparent position and brightness
    pub position: PlanetPosition,
    /// Rise time on the local day, if any
    pub rise: Option<DateTime<Tz>>,
    /// Set time on the local day, if any
    pub set: Option<DateTime<Tz>>,
}

/// Cached planet positions and rise/set times for the planets panel.
#[derive(Debug, Clone)]
pub struct CachedPlanets {
    /// Timestamp when the entries were calculated
    pub timestamp: DateTime<Tz>,
    /// One entry per supported planet, in distance order from the sun
    pub entries: Vec<CachedPlanet>,
}

impl CachedPlanets {
    /// Compute positions and rise/set times for all supported planets.
    pub fn new(location: &Location, timestamp: &DateTime<Tz>) -> Self {
        let entries = Planet::ALL
            .iter()
            .map(|&planet| CachedPlanet {
                planet,
                position: planets::planet_position(planet, location, timestamp),
                rise: planets::planet_event_time(planet, location, timestamp, PlanetEvent::Rise),
                set: planets::planet_event_time(planet, location, timestamp, PlanetEvent::Set),
            })
            .collect();
        Self {
            timestamp: *timestamp,
            entries,
        }
    }
}

// ============================================================================
// Moon Altitude Trend
// ============================================================================

/// Simple visibility indicator for the Moon.
#[derive(Debug, Clone, Copy)]
pub enum MoonAltitudeTrend {
    /// Moon center altitude < 0° (not visible).
    Down,
    /// Moon center altitude ≥ 0° (visible).
    Up,
}

// ============================================================================
// Cached Moon Details
// ============================================================================

/// Cached moon details including trend information.
#[derive(Debug, Clone, Copy)]
pub struct CachedMoonDetails {
    /// Timestamp when details were calculated
    pub timestamp: DateTime<Tz>,
    /// Moon position data
    pub moon: LunarPosition,
    /// Whether moon is above or below horizon
    pub altitude_trend: MoonAltitudeTrend,
}

impl CachedMoonDetails {
    /// Create moon details from cached positions.
    pub fn from_positions(location: &Location, positions: &CachedPositions) -> Self {
        let altitude_trend = determine_moon_trend(location, &positions.timestamp, positions.moon);
        Self {
            timestamp: positions.timestamp,
            moon: positions.moon,
            altitude_trend,
        }
    }
}

/// Determine if moon is above or below the horizon.
fn determine_moon_trend(
    _location: &Location,
    _timestamp: &DateTime<Tz>,
    base: LunarPosition,
) -> MoonAltitudeTrend {
    if base.altitude >= 0.0 {
        MoonAltitudeTrend::Up
    } else {
        MoonAltitudeTrend::Down
    }
}
