//! Location provenance tracking.
//!
//! This module provides types to track where geographic coordinates originated,
//! which is useful for debugging, logging, and UI display purposes.

/// Describes where latitude/longitude coordinates originated.
///
/// This enum tracks the provenance of location data to help users understand
/// whether they're using manually-entered coordinates, city database lookups,
/// or previously saved configuration.
///
/// # Examples
///
/// ```
/// use solunatus::location_source::LocationSource;
///
/// let source = LocationSource::CityDatabase;
/// assert_eq!(source.short_label(), "city");
/// ```
#[derive(Debug, Clone, Copy)]
pub enum LocationSource {
    /// Coordinates were provided manually via CLI arguments (--lat, --lon).
    ManualCli,
    /// Coordinates were looked up from the built-in city database.
    CityDatabase,
    /// Coordinates were loaded from saved configuration (~/.solunatus.json).
    SavedConfig,
}

impl LocationSource {
    /// Returns a short human-readable label describing the source.
    ///
    /// # Returns
    ///
    /// - `"manual"` for [`LocationSource::ManualCli`]
    /// - `"city"` for [`LocationSource::CityDatabase`]
    /// - `"saved"` for [`LocationSource::SavedConfig`]
    ///
    /// # Examples
    ///
    /// ```
    /// use solunatus::location_source::LocationSource;
    ///
    /// assert_eq!(LocationSource::ManualCli.short_label(), "manual");
    /// assert_eq!(LocationSource::CityDatabase.short_label(), "city");
    /// assert_eq!(LocationSource::SavedConfig.short_label(), "saved");
    /// ```
    pub fn short_label(self) -> &'static str {
        match self {
            LocationSource::ManualCli => "manual",
            LocationSource::CityDatabase => "city",
            LocationSource::SavedConfig => "saved",
        }
    }
}
