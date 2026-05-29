//! City database and search functionality.
//!
//! Provides an embedded database of 570+ major cities worldwide with
//! exact name matching and fuzzy search capabilities.
//!
//! # Features
//!
//! - Embedded city database (no external files needed)
//! - Exact name matching (case-insensitive)
//! - Fuzzy search with ranking
//! - Nearest city lookup by coordinates
//! - Distance and bearing calculations

use anyhow::{Context, Result};
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use serde::{Deserialize, Serialize};

/// Information about a city in the database.
///
/// Contains geographic coordinates and timezone information for a city.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct City {
    /// City name
    pub name: String,
    /// Latitude in degrees (negative = South, positive = North)
    pub lat: f64,
    /// Longitude in degrees (negative = West, positive = East)
    pub lon: f64,
    /// IANA timezone identifier (e.g., "America/New_York")
    pub tz: String,
    /// Country name
    pub country: String,
    /// State/province name (if applicable)
    pub state: Option<String>,
}

/// Database of major cities worldwide.
///
/// Contains 570+ cities with population centers and geographic data.
pub struct CityDatabase {
    cities: Vec<City>,
}

impl CityDatabase {
    /// Load the embedded city database.
    ///
    /// Loads the database from embedded JSON data (no external files required).
    ///
    /// # Errors
    ///
    /// Returns an error if the embedded data cannot be parsed.
    ///
    /// # Examples
    ///
    /// ```
    /// use solunatus::city::CityDatabase;
    ///
    /// let db = CityDatabase::load().unwrap();
    /// println!("Loaded {} cities", db.cities().len());
    /// ```
    pub fn load() -> Result<Self> {
        let data = include_str!("../data/urban_areas.json");
        let cities: Vec<City> =
            serde_json::from_str(data).context("Failed to parse city database")?;

        Ok(Self { cities })
    }

    /// Find the nearest city to given coordinates.
    ///
    /// Uses the Haversine formula to calculate great-circle distances.
    ///
    /// # Arguments
    ///
    /// * `lat` - Latitude in degrees
    /// * `lon` - Longitude in degrees
    ///
    /// # Returns
    ///
    /// `Some((city, distance_km, bearing_degrees))` or `None` if database is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use solunatus::city::CityDatabase;
    ///
    /// let db = CityDatabase::load().unwrap();
    /// if let Some((city, distance, bearing)) = db.find_nearest(40.7128, -74.0060) {
    ///     println!("Nearest city: {} ({:.1} km away)", city.name, distance);
    /// }
    /// ```
    pub fn find_nearest(&self, lat: f64, lon: f64) -> Option<(&City, f64, f64)> {
        let mut nearest: Option<(&City, f64, f64)> = None;
        let mut min_distance = f64::INFINITY;

        for city in &self.cities {
            let distance = haversine_distance(lat, lon, city.lat, city.lon);
            if distance < min_distance {
                let bearing = calculate_bearing(lat, lon, city.lat, city.lon);
                min_distance = distance;
                nearest = Some((city, distance, bearing));
            }
        }

        nearest
    }

    /// Get a reference to all cities in the database.
    ///
    /// Returns a slice of all cities (570+) in the database.
    #[allow(dead_code)]
    pub fn cities(&self) -> &[City] {
        &self.cities
    }

    /// Find a city by exact name match (case-insensitive).
    ///
    /// # Examples
    ///
    /// ```
    /// use solunatus::city::CityDatabase;
    ///
    /// let db = CityDatabase::load().unwrap();
    /// if let Some(city) = db.find_exact("Tokyo") {
    ///     println!("Found: {}, {} ({})", city.name, city.country, city.tz);
    /// }
    /// ```
    pub fn find_exact(&self, name: &str) -> Option<&City> {
        let name_lower = name.to_lowercase();
        self.cities
            .iter()
            .find(|c| c.name.to_lowercase() == name_lower)
    }

    /// Search for cities using fuzzy matching.
    ///
    /// Searches city names, states, and countries using fuzzy string matching.
    /// Results are sorted by match score (highest first).
    ///
    /// # Arguments
    ///
    /// * `query` - Search query string
    ///
    /// # Returns
    ///
    /// A vector of `(city, score)` tuples sorted by score (descending).
    ///
    /// # Examples
    ///
    /// ```
    /// use solunatus::city::CityDatabase;
    ///
    /// let db = CityDatabase::load().unwrap();
    /// let results = db.search("new york");
    /// for (city, score) in results.iter().take(5) {
    ///     println!("{}, {} (score: {})", city.name, city.country, score);
    /// }
    /// ```
    pub fn search(&self, query: &str) -> Vec<(&City, i64)> {
        let matcher = SkimMatcherV2::default();
        // Pre-allocate with approximate capacity
        let mut results = Vec::with_capacity(64);

        for city in &self.cities {
            // Use stack-allocated buffer to avoid repeated allocations
            let match_score = if let Some(state) = &city.state {
                // Try to match against "Name, State, Country" format
                let mut search_buf =
                    String::with_capacity(city.name.len() + state.len() + city.country.len() + 4);
                search_buf.push_str(&city.name);
                search_buf.push_str(", ");
                search_buf.push_str(state);
                search_buf.push_str(", ");
                search_buf.push_str(&city.country);
                matcher.fuzzy_match(&search_buf, query)
            } else {
                // Try to match against "Name, Country" format
                let mut search_buf =
                    String::with_capacity(city.name.len() + city.country.len() + 2);
                search_buf.push_str(&city.name);
                search_buf.push_str(", ");
                search_buf.push_str(&city.country);
                matcher.fuzzy_match(&search_buf, query)
            };

            if let Some(score) = match_score {
                results.push((city, score));
            }
        }

        // Sort by score descending (highest scores first)
        results.sort_unstable_by_key(|entry| std::cmp::Reverse(entry.1));
        results
    }
}

/// Calculate the great-circle distance between two points using the Haversine formula
/// Returns distance in kilometers
fn haversine_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    const EARTH_RADIUS_KM: f64 = 6371.0;

    let lat1_rad = lat1.to_radians();
    let lat2_rad = lat2.to_radians();
    let delta_lat = (lat2 - lat1).to_radians();
    let delta_lon = (lon2 - lon1).to_radians();

    let a = (delta_lat / 2.0).sin().powi(2)
        + lat1_rad.cos() * lat2_rad.cos() * (delta_lon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    EARTH_RADIUS_KM * c
}

/// Calculate the initial bearing from point 1 to point 2
/// Returns bearing in degrees (0-360, where 0 is North)
fn calculate_bearing(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let lat1_rad = lat1.to_radians();
    let lat2_rad = lat2.to_radians();
    let delta_lon = (lon2 - lon1).to_radians();

    let y = delta_lon.sin() * lat2_rad.cos();
    let x = lat1_rad.cos() * lat2_rad.sin() - lat1_rad.sin() * lat2_rad.cos() * delta_lon.cos();

    let bearing_rad = y.atan2(x);
    let bearing_deg = bearing_rad.to_degrees();

    // Normalize to 0-360
    (bearing_deg + 360.0) % 360.0
}

/// Convert bearing in degrees to compass direction
pub fn bearing_to_compass(bearing: f64) -> &'static str {
    let normalized = ((bearing % 360.0) + 360.0) % 360.0;
    match normalized {
        b if b < 11.25 => "N",
        b if b < 33.75 => "NNE",
        b if b < 56.25 => "NE",
        b if b < 78.75 => "ENE",
        b if b < 101.25 => "E",
        b if b < 123.75 => "ESE",
        b if b < 146.25 => "SE",
        b if b < 168.75 => "SSE",
        b if b < 191.25 => "S",
        b if b < 213.75 => "SSW",
        b if b < 236.25 => "SW",
        b if b < 258.75 => "WSW",
        b if b < 281.25 => "W",
        b if b < 303.75 => "WNW",
        b if b < 326.25 => "NW",
        b if b < 348.75 => "NNW",
        _ => "N",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_database() {
        let db = CityDatabase::load().unwrap();
        assert!(!db.cities().is_empty());
    }

    #[test]
    fn test_find_exact() {
        let db = CityDatabase::load().unwrap();
        let city = db.find_exact("New York");
        assert!(city.is_some());
        assert_eq!(city.unwrap().country, "US");
    }

    #[test]
    fn test_search() {
        let db = CityDatabase::load().unwrap();
        let results = db.search("san");
        assert!(!results.is_empty());
    }
}
