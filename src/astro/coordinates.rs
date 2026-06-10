//! Coordinate system transformation utilities.
//!
//! Helpers for converting between astronomical coordinate representations,
//! such as azimuth angles to compass directions.

/// Convert altitude-azimuth to compass bearing
pub fn azimuth_to_compass(azimuth: f64) -> &'static str {
    let idx = ((azimuth + 11.25) / 22.5) as usize % 16;
    const COMPASS: [&str; 16] = [
        "N", "NNE", "NE", "ENE", "E", "ESE", "SE", "SSE", "S", "SSW", "SW", "WSW", "W", "WNW",
        "NW", "NNW",
    ];
    COMPASS[idx]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_azimuth_to_compass() {
        assert_eq!(azimuth_to_compass(0.0), "N");
        assert_eq!(azimuth_to_compass(45.0), "NE");
        assert_eq!(azimuth_to_compass(90.0), "E");
        assert_eq!(azimuth_to_compass(180.0), "S");
        assert_eq!(azimuth_to_compass(270.0), "W");
    }
}
