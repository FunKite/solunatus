//! NTP time synchronization with intelligent caching.
//!
//! This module provides network time protocol (NTP) synchronization to detect
//! clock drift between the system clock and authoritative time sources.
//!
//! # Features
//!
//! - **NTP Query**: Connects to time.google.com or pool.ntp.org for time sync
//! - **Smart Caching**: 30-minute cache to comply with NTP server Terms of Service
//! - **Drift Detection**: Measures system clock offset in microseconds
//! - **Direction Classification**: Identifies if system is ahead, behind, or in sync
//!
//! # Cache Behavior
//!
//! To prevent abuse of public NTP servers, this module caches time sync results
//! for 30 minutes minimum. The cache is stored at `~/.solunatus_ntp_cache.json`.
//!
//! # Examples
//!
//! ```no_run
//! use solunatus::time_sync::{check_time_sync, format_offset};
//!
//! let sync_info = check_time_sync();
//! if let Some(delta) = sync_info.delta {
//!     println!("System clock offset: {}", format_offset(delta));
//! }
//! ```

use anyhow::{anyhow, Context};
use chrono::{DateTime, Duration as ChronoDuration, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::net::UdpSocket;
use std::path::PathBuf;
use std::time::Duration as StdDuration;

const TIME_SERVERS: [(&str, &str); 2] = [
    ("time.google.com:123", "time.google.com (NTP)"),
    ("pool.ntp.org:123", "pool.ntp.org (NTP)"),
];
/// The label for the primary NTP time source (time.google.com).
pub const PRIMARY_SOURCE_LABEL: &str = TIME_SERVERS[0].1;
const SYNC_THRESHOLD_MICROS: i64 = 50_000; // 50 ms tolerance treated as in sync

// Cache settings - 30 minutes minimum between NTP queries (pool.ntp.org ToS compliance)
const CACHE_MIN_INTERVAL_SECS: i64 = 1800; // 30 minutes

/// Cached NTP time synchronization result.
///
/// Stored at `~/.solunatus_ntp_cache.json` to reduce load on public NTP servers.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TimeSyncCache {
    /// UTC timestamp when this cache entry was created
    timestamp: DateTime<Utc>,
    /// Source server that was queried
    source: String,
    /// Delta in microseconds (system time - NTP time)
    delta_micros: i64,
}

/// Returns the default NTP servers to use for time synchronization.
///
/// # Returns
///
/// A vector of (server_address, display_label) tuples containing:
/// - time.google.com:123
/// - pool.ntp.org:123
pub fn default_servers() -> Vec<(String, String)> {
    TIME_SERVERS
        .iter()
        .map(|(server, label)| (server.to_string(), label.to_string()))
        .collect()
}

/// Information about system clock synchronization status.
///
/// Contains the time source, measured drift, and any errors encountered.
#[derive(Debug, Clone)]
pub struct TimeSyncInfo {
    /// The NTP server source used (e.g., "time.google.com (NTP)")
    pub source: String,
    /// The measured clock delta (None if sync failed)
    pub delta: Option<ChronoDuration>,
    /// Error message if synchronization failed
    pub error: Option<String>,
}

/// Direction of system clock drift relative to NTP time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeSyncDirection {
    /// System clock is ahead of NTP time
    Ahead,
    /// System clock is behind NTP time
    Behind,
    /// System clock is synchronized (within ±50ms)
    InSync,
}

/// Checks system time synchronization using default NTP servers.
///
/// Uses cached results if available and fresh (< 30 minutes old).
///
/// # Returns
///
/// A [`TimeSyncInfo`] struct containing sync status and any errors.
///
/// # Examples
///
/// ```no_run
/// use solunatus::time_sync::check_time_sync;
///
/// let sync = check_time_sync();
/// match sync.delta {
///     Some(delta) => println!("System clock offset: {:?}", delta),
///     None => println!("Time sync failed: {:?}", sync.error),
/// }
/// ```
pub fn check_time_sync() -> TimeSyncInfo {
    check_time_sync_with_servers(None)
}

/// Checks system time synchronization with optional custom server.
///
/// # Arguments
///
/// * `custom_server` - Optional NTP server address (e.g., "time.google.com:123")
///
/// # Returns
///
/// A [`TimeSyncInfo`] struct containing sync status and any errors.
pub fn check_time_sync_with_servers(custom_server: Option<&str>) -> TimeSyncInfo {
    // Determine which server we're targeting (for cache key matching)
    let target_server = custom_server
        .filter(|s| !s.trim().is_empty())
        .unwrap_or(TIME_SERVERS[0].0); // Default to primary server

    // Try to load from cache first
    if let Ok(cache) = load_cache() {
        let age = Utc::now().signed_duration_since(cache.timestamp);

        // Check if cache matches our target server and is fresh (< 30 minutes old)
        // Note: We normalize server names for comparison (strip port if present in cache)
        let cache_server_normalized = cache.source.split(':').next().unwrap_or(&cache.source);
        let target_server_normalized = target_server.split(':').next().unwrap_or(target_server);

        if cache_server_normalized == target_server_normalized
            && age.num_seconds() < CACHE_MIN_INTERVAL_SECS
        {
            let delta = ChronoDuration::microseconds(cache.delta_micros);
            return TimeSyncInfo {
                source: cache.source,
                delta: Some(delta),
                error: None,
            };
        }
    }

    // Cache is stale, missing, or for different server - perform fresh NTP query
    match fetch_delta(custom_server) {
        Ok((delta, source, server_addr)) => {
            // Save to cache for future calls
            if let Some(micros) = delta.num_microseconds() {
                let cache = TimeSyncCache {
                    timestamp: Utc::now(),
                    source: server_addr, // Store actual server address for cache matching
                    delta_micros: micros,
                };
                let _ = save_cache(&cache); // Ignore save errors
            }

            TimeSyncInfo {
                source,
                delta: Some(delta),
                error: None,
            }
        }
        Err(err) => TimeSyncInfo {
            source: target_server.to_string(),
            delta: None,
            error: Some(err.to_string()),
        },
    }
}

/// Formats a time offset as a human-readable string.
///
/// Automatically chooses appropriate units based on magnitude:
/// - Minutes/seconds for offsets ≥ 60s
/// - Seconds with 3 decimals for offsets ≥ 1s
/// - Milliseconds with 1 decimal for offsets < 1s
///
/// # Arguments
///
/// * `delta` - The time offset to format
///
/// # Returns
///
/// A formatted string like "+2.3s", "-5m30s", or "+12.5ms"
///
/// # Examples
///
/// ```
/// use chrono::Duration;
/// use solunatus::time_sync::format_offset;
///
/// let delta = Duration::milliseconds(250);
/// assert_eq!(format_offset(delta), "+250.0ms");
/// ```
pub fn format_offset(delta: ChronoDuration) -> String {
    let total_seconds = delta.num_seconds();
    let abs_seconds = total_seconds.abs();
    if abs_seconds >= 60 {
        let minutes = abs_seconds / 60;
        let seconds = abs_seconds % 60;
        let sign = if total_seconds >= 0 { "+" } else { "-" };
        return format!("{}{}m{}s", sign, minutes, seconds);
    }

    if let Some(micros) = delta.num_microseconds() {
        let seconds = micros as f64 / 1_000_000.0;
        if seconds.abs() >= 1.0 {
            format!("{:+.3}s", seconds)
        } else {
            format!("{:+.1}ms", seconds * 1000.0)
        }
    } else {
        format!("{:+}s", total_seconds)
    }
}

/// Returns a human-readable description of the sync direction.
///
/// # Arguments
///
/// * `direction` - The time sync direction
///
/// # Returns
///
/// - `"system ahead"` for [`TimeSyncDirection::Ahead`]
/// - `"system behind"` for [`TimeSyncDirection::Behind`]
/// - `"system in sync"` for [`TimeSyncDirection::InSync`]
pub fn describe_direction(direction: TimeSyncDirection) -> &'static str {
    match direction {
        TimeSyncDirection::Ahead => "system ahead",
        TimeSyncDirection::Behind => "system behind",
        TimeSyncDirection::InSync => "system in sync",
    }
}

/// Returns a machine-readable code for the sync direction.
///
/// # Arguments
///
/// * `direction` - The time sync direction
///
/// # Returns
///
/// - `"ahead"` for [`TimeSyncDirection::Ahead`]
/// - `"behind"` for [`TimeSyncDirection::Behind`]
/// - `"in_sync"` for [`TimeSyncDirection::InSync`]
pub fn direction_code(direction: TimeSyncDirection) -> &'static str {
    match direction {
        TimeSyncDirection::Ahead => "ahead",
        TimeSyncDirection::Behind => "behind",
        TimeSyncDirection::InSync => "in_sync",
    }
}

/// Returns (delta, label, server_address)
fn fetch_delta(custom_server: Option<&str>) -> anyhow::Result<(ChronoDuration, String, String)> {
    let mut last_err: Option<anyhow::Error> = None;

    // If custom server is specified, try it first
    if let Some(server_str) = custom_server {
        let server_trimmed = server_str.trim();
        if !server_trimmed.is_empty() {
            let server_with_port = if server_trimmed.contains(':') {
                server_trimmed.to_string()
            } else {
                format!("{}:123", server_trimmed)
            };

            match query_ntp(&server_with_port) {
                Ok(server_time) => {
                    let system_time = Utc::now();
                    let delta = system_time.signed_duration_since(server_time);
                    // Return server address and label for cache tracking/display.
                    return Ok((
                        delta,
                        server_trimmed.to_string(),
                        server_trimmed.to_string(),
                    ));
                }
                Err(err) => {
                    last_err = Some(anyhow!("{} query failed: {}", server_trimmed, err));
                }
            }
        }
    }

    // Fall back to default servers
    for (server, label) in TIME_SERVERS.iter() {
        match query_ntp(server) {
            Ok(server_time) => {
                let system_time = Utc::now();
                let delta = system_time.signed_duration_since(server_time);
                // Extract server address without port for cache tracking
                let server_addr = server.split(':').next().unwrap_or(server).to_string();
                return Ok((delta, (*label).to_string(), server_addr));
            }
            Err(err) => {
                last_err = Some(anyhow!("{} query failed: {}", label, err));
            }
        }
    }

    Err(last_err.unwrap_or_else(|| anyhow!("all time sources failed")))
}

const NTP_UNIX_OFFSET: i64 = 2_208_988_800; // Seconds between 1900-01-01 and 1970-01-01

/// Encode a UTC instant as a 64-bit NTP timestamp (32-bit seconds + 32-bit fraction).
fn ntp_timestamp_bytes(instant: DateTime<Utc>) -> [u8; 8] {
    let seconds = (instant.timestamp() + NTP_UNIX_OFFSET) as u32;
    let fraction =
        (((instant.timestamp_subsec_nanos() as u64) << 32) / 1_000_000_000u64) as u32;
    let mut bytes = [0u8; 8];
    bytes[..4].copy_from_slice(&seconds.to_be_bytes());
    bytes[4..].copy_from_slice(&fraction.to_be_bytes());
    bytes
}

/// Decode a 64-bit NTP timestamp from a packet field into a UTC instant.
fn ntp_timestamp_to_utc(field: &[u8]) -> Option<DateTime<Utc>> {
    let seconds = u32::from_be_bytes([field[0], field[1], field[2], field[3]]);
    let fraction = u32::from_be_bytes([field[4], field[5], field[6], field[7]]);
    let unix_seconds = seconds as i64 - NTP_UNIX_OFFSET;
    if unix_seconds < 0 {
        return None;
    }
    let nanos = ((fraction as u128) * 1_000_000_000u128 / (1u128 << 32)) as u32;
    Utc.timestamp_opt(unix_seconds, nanos).single()
}

/// Returns the server time corrected for round-trip delay, evaluated at the moment
/// the client received the reply (so the caller can compare against `Utc::now()`).
fn query_ntp(server: &str) -> anyhow::Result<chrono::DateTime<Utc>> {
    let socket =
        UdpSocket::bind("0.0.0.0:0").context("failed to bind local UDP socket for time sync")?;
    socket
        .set_read_timeout(Some(StdDuration::from_secs(3)))
        .context("failed to set read timeout")?;
    socket
        .set_write_timeout(Some(StdDuration::from_secs(3)))
        .context("failed to set write timeout")?;
    // Connect so the kernel only delivers datagrams from this server, rejecting
    // off-path injected replies.
    socket
        .connect(server)
        .with_context(|| format!("failed to connect to {}", server))?;

    let mut packet = [0u8; 48];
    packet[0] = 0b00_100_011; // LI = 0, VN = 4, Mode = 3 (client)

    // Stamp our transmit time as the originate timestamp; the server echoes it
    // back, letting us reject stale or spoofed replies and measure round-trip time.
    let t1 = Utc::now();
    let originate = ntp_timestamp_bytes(t1);
    packet[40..48].copy_from_slice(&originate);

    socket
        .send(&packet)
        .with_context(|| format!("failed to send request to {}", server))?;

    let mut response = [0u8; 48];
    let len = socket
        .recv(&mut response)
        .with_context(|| format!("failed to receive response from {}", server))?;
    let t4 = Utc::now();

    if len < 48 {
        return Err(anyhow!("incomplete NTP response ({} bytes)", len));
    }

    // Reject non-server replies and Kiss-o'-Death packets (stratum 0).
    let mode = response[0] & 0b0000_0111;
    if mode != 4 || response[1] == 0 {
        return Err(anyhow!("unexpected NTP reply (mode {}, stratum {})", mode, response[1]));
    }

    // The server must echo our originate timestamp in bytes 24..32.
    if response[24..32] != originate {
        return Err(anyhow!("NTP reply originate timestamp mismatch (possible spoof)"));
    }

    let t2 = ntp_timestamp_to_utc(&response[32..40])
        .ok_or_else(|| anyhow!("invalid receive timestamp from {}", server))?;
    let t3 = ntp_timestamp_to_utc(&response[40..48])
        .ok_or_else(|| anyhow!("invalid transmit timestamp from {}", server))?;

    // Clock offset (server - client) = ((T2 - T1) + (T3 - T4)) / 2.
    let d1 = t2
        .signed_duration_since(t1)
        .num_nanoseconds()
        .ok_or_else(|| anyhow!("NTP offset overflow"))?;
    let d2 = t3
        .signed_duration_since(t4)
        .num_nanoseconds()
        .ok_or_else(|| anyhow!("NTP offset overflow"))?;
    let offset = ChronoDuration::nanoseconds((d1 + d2) / 2);

    Ok(t4 + offset)
}

impl TimeSyncInfo {
    /// Classifies the direction of time drift.
    ///
    /// # Returns
    ///
    /// - `Some(`[`TimeSyncDirection`]`)` if delta is available
    /// - `None` if time sync failed or delta is out of range
    pub fn direction(&self) -> Option<TimeSyncDirection> {
        self.delta.and_then(classify_direction)
    }

    /// Converts the time delta to seconds as a floating point value.
    ///
    /// # Returns
    ///
    /// - `Some(f64)` representing seconds (e.g., 0.123 for 123ms)
    /// - `None` if time sync failed
    pub fn delta_seconds(&self) -> Option<f64> {
        self.delta.and_then(|delta| {
            delta
                .num_microseconds()
                .map(|micros| micros as f64 / 1_000_000.0)
        })
    }

    /// Returns a shortened version of the error message if present.
    ///
    /// Truncates error messages to 60 characters for display purposes.
    ///
    /// # Returns
    ///
    /// - `Some(String)` with truncated error message
    /// - `None` if no error occurred
    pub fn error_summary(&self) -> Option<String> {
        self.error.as_ref().map(|err| summarize_error(err))
    }
}

fn summarize_error(err: &str) -> String {
    const MAX_LEN: usize = 60;
    if err.chars().count() <= MAX_LEN {
        err.to_string()
    } else {
        let truncated: String = err.chars().take(MAX_LEN).collect();
        format!("{}…", truncated.trim_end())
    }
}

fn classify_direction(delta: ChronoDuration) -> Option<TimeSyncDirection> {
    if let Some(micros) = delta.num_microseconds() {
        if micros.abs() <= SYNC_THRESHOLD_MICROS {
            Some(TimeSyncDirection::InSync)
        } else if micros > 0 {
            Some(TimeSyncDirection::Ahead)
        } else {
            Some(TimeSyncDirection::Behind)
        }
    } else {
        None
    }
}

/// Get the cache file path
fn cache_file_path() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".solunatus_ntp_cache.json"))
}

/// Load cached time sync result
fn load_cache() -> anyhow::Result<TimeSyncCache> {
    let path = cache_file_path().ok_or_else(|| anyhow!("cannot determine home directory"))?;

    if !path.exists() {
        return Err(anyhow!("cache file does not exist"));
    }

    let contents = fs::read_to_string(&path)
        .with_context(|| format!("failed to read cache file: {}", path.display()))?;

    let cache: TimeSyncCache =
        serde_json::from_str(&contents).context("failed to parse cache file")?;

    Ok(cache)
}

/// Save time sync result to cache
fn save_cache(cache: &TimeSyncCache) -> anyhow::Result<()> {
    let path = cache_file_path().ok_or_else(|| anyhow!("cannot determine home directory"))?;

    let json = serde_json::to_string_pretty(cache).context("failed to serialize cache")?;

    fs::write(&path, json)
        .with_context(|| format!("failed to write cache file: {}", path.display()))?;

    Ok(())
}
