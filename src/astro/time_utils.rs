// Time utilities for astronomical calculations

use chrono::{DateTime, Duration, TimeZone};

/// Format duration with seconds as detailed string
pub fn format_duration_detailed(duration: Duration) -> String {
    let total_secs = duration.num_seconds().abs();
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;

    if duration.num_seconds() < 0 {
        format!("{:02}:{:02}:{:02} ago", hours, minutes, seconds)
    } else {
        format!("{:02}:{:02}:{:02} from now", hours, minutes, seconds)
    }
}

/// Calculate time until an event
pub fn time_until<T: TimeZone + Clone>(from: &DateTime<T>, to: &DateTime<T>) -> Duration {
    to.clone().signed_duration_since(from.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_format_duration_detailed_future() {
        let dur = Duration::hours(2) + Duration::minutes(30) + Duration::seconds(15);
        assert_eq!(format_duration_detailed(dur), "02:30:15 from now");
    }

    #[test]
    fn test_format_duration_detailed_past() {
        let dur = Duration::hours(-1) - Duration::minutes(5) - Duration::seconds(9);
        assert_eq!(format_duration_detailed(dur), "01:05:09 ago");
    }

    #[test]
    fn test_time_until() {
        let start = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2024, 1, 1, 1, 0, 0).unwrap();
        let diff = time_until(&start, &end);
        assert_eq!(diff, Duration::hours(1));
    }
}
