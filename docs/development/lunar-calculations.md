# Lunar Calculations Deep Dive

Comprehensive technical guide to lunar position and phase calculations in Solunatus.

## Overview

Solunatus implements high-precision lunar calculations based on Jean Meeus' "Astronomical Algorithms" (2nd Edition). These algorithms provide accurate geocentric and topocentric moon positions, phases, and event times.

## Accuracy

- **Geocentric positions**: Accurate to within a few arcminutes
- **Topocentric positions**: Accounts for Earth's flattening and parallax
- **Lunar phase times**: Accurate to within ±1 minute for current dates
- **Moonrise/moonset times**: Accurate to within ±3 minutes at mid-latitudes

## Algorithms Source

The lunar calculations are based on:
- Jean Meeus, "Astronomical Algorithms", 2nd Edition
- Chapter 47: Position of the Moon
- Chapter 49: Phases of the Moon

## Core Components

### 1. Lunar Position Calculation

The `LunarPosition` struct provides comprehensive information about the moon's appearance:

```rust
pub struct LunarPosition {
    pub altitude: f64,           // Degrees above horizon
    pub azimuth: f64,            // Degrees from North (0-360)
    pub distance: f64,           // Distance from Earth (km)
    pub illumination: f64,       // Fraction illuminated (0.0-1.0)
    pub phase_angle: f64,        // Phase angle in degrees
    pub angular_diameter: f64,   // Apparent size (arcminutes)
}
```

### 2. Mean Lunar Elements

The calculations begin with mean orbital elements, which are polynomial functions of time:

#### Mean Longitude (L')
```
L' = 218.3164477°
     + 481267.88123421°T
     - 0.0015786°T²
     + T³/538841
     - T⁴/65194000
```

Where T is centuries since J2000.0 (Julian Date 2451545.0).

#### Mean Elongation (D)
The angular distance between the Sun and Moon as seen from Earth:
```
D = 297.8501921°
    + 445267.1114034°T
    - 0.0018819°T²
    + T³/545868
    - T⁴/113065000
```

#### Mean Anomaly (M')
The moon's angular distance from perigee:
```
M' = 134.9633964°
     + 477198.8675055°T
     + 0.0087414°T²
     + T³/69699
     - T⁴/14712000
```

#### Sun's Mean Anomaly (M)
```
M = 357.5291092°
    + 35999.0502909°T
    - 0.0001536°T²
    + T³/24490000
```

#### Argument of Latitude (F)
```
F = 93.2720950°
    + 483202.0175233°T
    - 0.0036539°T²
    - T³/3526000
    + T⁴/863310000
```

### 3. Geocentric Coordinates

From the mean elements, the true geocentric position is calculated using a series of periodic terms. Meeus provides 60+ terms for longitude and latitude, and 40+ terms for distance.

#### Longitude Correction
The lunar longitude includes periodic terms based on:
- D (elongation)
- M (sun's anomaly)
- M' (moon's anomaly)
- F (argument of latitude)

Major terms include:
- Main perturbation by the Sun
- Evection (discovered by Ptolemy)
- Variation (discovered by Tycho Brahe)
- Annual equation
- Reduction

#### Latitude Calculation
Lunar latitude is calculated from similar periodic terms, determining how far north or south of the ecliptic the moon appears.

#### Distance from Earth
The distance varies from approximately:
- **Perigee**: ~356,400 km (closest)
- **Apogee**: ~406,700 km (farthest)
- **Mean distance**: ~384,400 km

### 4. Topocentric Correction

For observer-specific calculations, the geocentric position is corrected for:

#### Parallax
The moon is close enough that its position varies significantly based on observer location. The horizontal parallax can be up to 1°.

#### Earth's Flattening
Earth is an oblate spheroid (flattened at poles). This affects the calculation of observer position relative to Earth's center.

#### Altitude Correction
The topocentric altitude accounts for:
- Geocentric altitude
- Parallax in altitude
- Optional atmospheric refraction (~0.6° at horizon)

### 5. Phase Calculation

#### Illumination Fraction
The fraction of the moon's disk that is illuminated:
```
k = (1 + cos(phase_angle)) / 2
```

Where phase angle is the angle between the Sun and Earth as seen from the Moon:
- 0° = New Moon (0% illuminated)
- 90° = First/Last Quarter (50% illuminated)
- 180° = Full Moon (100% illuminated)

#### Phase Categories
Based on the phase angle:
- **New Moon**: 0° ± 22.5°
- **Waxing Crescent**: 22.5° - 67.5°
- **First Quarter**: 67.5° - 112.5°
- **Waxing Gibbous**: 112.5° - 157.5°
- **Full Moon**: 157.5° - 202.5°
- **Waning Gibbous**: 202.5° - 247.5°
- **Last Quarter**: 247.5° - 292.5°
- **Waning Crescent**: 292.5° - 337.5°

#### Phase Age
Days since the last new moon (0-29.5 days). The synodic month (lunar cycle) averages 29.530588 days.

### 6. Lunar Phase Times

Exact times of major phases (New, First Quarter, Full, Last Quarter) are calculated using Meeus' algorithm from Chapter 49.

The calculation involves:
1. Computing the mean phase time (JDE) for the lunation number `k`
2. Adding the complete set of periodic corrections for that phase type
   (separate coefficient tables for new moon, full moon, and the quarters),
   based on:
   - Sun's and Moon's anomalies
   - Moon's argument of latitude and longitude of ascending node
   - Eccentricity of Earth's orbit
3. Adding the 14 planetary-argument corrections
4. Converting the result from Terrestrial Time to UTC with the ΔT estimate
   shared with the equinox/solstice calculations

The series is checked against Meeus' worked examples 49.a and 49.b and
against published USNO phase times.

### 7. Rise and Set Times

Moonrise and moonset calculations use a bisection method:

1. **Search interval**: The algorithm searches the local civil day, from one local midnight to the next (23 or 25 hours on daylight-saving transition days)
2. **Target altitude**: -0.833° (accounting for refraction and lunar radius)
3. **Bisection method**: Repeatedly narrows the time window by checking if the moon is above or below the target altitude
4. **Precision**: Converges to within seconds

#### Special Cases
- **Circumpolar**: Moon never sets (high latitudes in summer)
- **Never rises**: Moon never visible (high latitudes in winter)
- **Multiple events**: Sometimes 0, 1, or 2 rises/sets per day

### 8. Angular Diameter

The moon's apparent size varies with distance:

```
angular_diameter = 2 * arctan(MOON_RADIUS / distance)
```

- **At perigee**: ~33.5 arcminutes (largest)
- **At apogee**: ~29.5 arcminutes (smallest)
- **Mean**: ~31.1 arcminutes

This variation causes "supermoons" when full moons occur near perigee.

## Implementation Details

### Time Systems

All calculations use:
- **Input**: Local time with timezone
- **Internal**: Julian Day Numbers (JD) and Julian Centuries since J2000.0
- **Output**: Local time with timezone

### Coordinate Systems

- **Ecliptic coordinates**: Longitude and latitude relative to the ecliptic plane
- **Equatorial coordinates**: Right ascension and declination
- **Horizontal coordinates**: Altitude and azimuth (observer-specific)

### Precision Considerations

- All calculations use `f64` (double-precision floating point)
- Angles are normalized to [0°, 360°) range
- Trigonometric functions use radians internally
- Intermediate results are not rounded until final output

## Validation

Lunar calculations are validated against:
- U.S. Naval Observatory (USNO) data
- JPL Horizons ephemeris system (for comparison)
- Historical astronomical observations

## Limitations

### Known Constraints
- **Accuracy decreases** at extreme latitudes (>66° N/S) due to grazing rises/sets
- **Atmospheric refraction** uses standard model (actual varies with weather)
- **No topographic effects**: Mountains and valleys are not considered
- **Historical dates**: Accuracy decreases for dates far from present (±100 years optimal)

### Not Included
- **Libration**: The small oscillations that allow us to see slightly more than 50% of the lunar surface
- **Lunar eclipses**: Not yet implemented
- **Detailed lunar features**: Craters, seas, etc.

## Performance

Typical calculation times (on modern hardware):
- **Single position**: <0.1 ms
- **Monthly phases**: ~1 ms
- **Full day rise/set**: 1-5 ms (bisection method)

## Future Enhancements

Potential improvements under consideration:
- More precise topographic effects
- Lunar eclipse predictions
- Historical date optimization
- Libration calculations

## Code Organization

The lunar calculations are in `/src/astro/moon.rs`:

```
moon.rs
├── Mean elements functions
│   ├── moon_mean_longitude()
│   ├── moon_mean_elongation()
│   ├── moon_mean_anomaly()
│   └── sun_mean_anomaly_moon()
├── Position calculation
│   ├── geocentric_position()
│   ├── topocentric_position()
│   └── lunar_position()
├── Phase calculations
│   ├── calculate_phase_angle()
│   ├── calculate_illumination()
│   └── lunar_phases()
└── Rise/set calculations
    ├── lunar_event_time()
    └── find_lunar_events()
```

## References

### Primary Sources
1. Jean Meeus, "Astronomical Algorithms", 2nd Edition (1998)
   - Chapter 47: Position of the Moon
   - Chapter 49: Phases of the Moon

### Validation Sources
2. U.S. Naval Observatory: https://aa.usno.navy.mil/
3. JPL Horizons: https://ssd.jpl.nasa.gov/horizons/

### Additional Reading
4. Montenbruck & Pfleger, "Astronomy on the Personal Computer" (2000)
5. The Astronomical Almanac (annual publication)

## Example Usage

```rust
use solunatus::prelude::*;
use chrono::Local;
use chrono_tz::America::New_York;

// Calculate current lunar position
let location = Location::new(40.7128, -74.0060).unwrap();
let now = Local::now().with_timezone(&New_York);
let position = lunar_position(&location, &now);

println!("Moon altitude: {:.2}°", position.altitude);
println!("Moon azimuth: {:.2}°", position.azimuth);
println!("Distance: {:.0} km", position.distance);
println!("Illumination: {:.1}%", position.illumination * 100.0);

// Get moonrise/moonset times
if let Some(moonrise) = lunar_event_time(&location, &now, LunarEvent::Moonrise) {
    println!("Moonrise: {}", moonrise.format("%H:%M"));
}

// Get monthly lunar phases
let phases = lunar_phases(2025, 12);
for phase in phases {
    println!("{:?}: {}", phase.phase_type, phase.datetime.format("%Y-%m-%d %H:%M"));
}
```

## See Also

- [Solar Calculations](solar-calculations.md) - Deep dive into solar algorithms
- [Architecture Overview](architecture.md) - Overall code structure
- [Accuracy Testing](accuracy.md) - Validation methodology
- [API Documentation](https://docs.rs/solunatus) - Complete API reference
