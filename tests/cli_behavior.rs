//! One-shot CLI behavior: `--date` handling, deprecated flags, and saved
//! configuration safety. Each test runs with an isolated home directory.
mod common;
use common::{TestHome, temp_home};
use std::process::Output;

fn run_in(home: &TestHome, args: &[&str]) -> Output {
    home.command().args(args).output().unwrap()
}

#[test]
fn date_prints_a_one_shot_report_instead_of_opening_the_dashboard() {
    let home = temp_home("date");
    let output = run_in(
        &home,
        &["--city", "Boston", "--date", "2026-12-25", "--no-save"],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.starts_with("Solunatus "));
    assert!(text.contains("📅 Dec 25 12:00:00"));
    assert!(!text.contains('\x1b'), "must not enter terminal mode");
}

#[test]
fn watch_and_date_conflict() {
    let home = temp_home("watch-date");
    let output = run_in(
        &home,
        &["--city", "Boston", "--date", "2026-12-25", "--watch"],
    );
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("cannot be used with"));
}

#[test]
fn strict_is_hidden_and_warns_as_deprecated() {
    let home = temp_home("strict");
    let help = run_in(&home, &["--help"]);
    assert!(!String::from_utf8_lossy(&help.stdout).contains("--strict"));

    let output = run_in(
        &home,
        &["--city", "Boston", "--next", "sunrise", "--strict"],
    );
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("--strict is deprecated"));
}

#[test]
fn unreadable_config_is_reported_and_never_overwritten() {
    let home = temp_home("corrupt-config");
    let path = home.join(".solunatus.json");
    let corrupt = r#"{"lat": 42.36, "lon": -71.06, "watch": {"night_mode": true"#;
    std::fs::write(&path, corrupt).unwrap();

    // Without a location on the command line, explain why the saved one is unusable.
    let output = run_in(&home, &["--no-prompt"]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("warning: ignoring saved settings"),
        "{stderr}"
    );
    assert!(stderr.contains("could not be read"), "{stderr}");

    // A normal run that would save settings must leave the file untouched.
    let output = run_in(&home, &["--city", "Tucson", "--no-prompt"]);
    assert!(output.status.success());
    assert_eq!(std::fs::read_to_string(&path).unwrap(), corrupt);
}

#[test]
fn readable_config_is_still_saved() {
    let home = temp_home("save-config");
    let output = run_in(&home, &["--city", "Tucson", "--no-prompt"]);
    assert!(output.status.success());
    let path = home.join(".solunatus.json");
    let saved = std::fs::read_to_string(&path).unwrap();
    assert!(saved.contains("\"city\": \"Tucson\""));
}

#[cfg(unix)]
#[test]
fn saved_config_is_not_readable_by_other_users() {
    use std::os::unix::fs::PermissionsExt;

    let home = temp_home("config-permissions");
    let output = run_in(&home, &["--city", "Tucson", "--no-prompt"]);
    assert!(output.status.success());
    let path = home.join(".solunatus.json");
    let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
    assert_eq!(
        mode, 0o600,
        "config file must be readable only by its owner"
    );
}

#[cfg(unix)]
#[test]
fn saving_legacy_config_does_not_expose_new_settings_through_old_descriptor() {
    use std::io::Read;
    use std::os::unix::fs::PermissionsExt;

    let home = temp_home("legacy-config-permissions");
    let output = run_in(&home, &["--city", "Boston", "--no-prompt"]);
    assert!(output.status.success());
    let path = home.join(".solunatus.json");
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
    let previous = std::fs::read_to_string(&path).unwrap();
    let mut old_descriptor = std::fs::File::open(&path).unwrap();

    let output = run_in(&home, &["--city", "Tucson", "--no-prompt"]);
    assert!(output.status.success());
    let mut visible_through_old_descriptor = String::new();
    old_descriptor
        .read_to_string(&mut visible_through_old_descriptor)
        .unwrap();
    assert_eq!(visible_through_old_descriptor, previous);
    assert!(
        std::fs::read_to_string(&path)
            .unwrap()
            .contains("\"city\": \"Tucson\"")
    );
    assert_eq!(
        std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
        0o600
    );
    assert_eq!(
        std::fs::read_dir(&home).unwrap().count(),
        1,
        "temporary file must be removed"
    );
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

/// Half the world has a negative longitude or latitude; the ordinary
/// space-separated form must work, as the documentation shows it.
#[test]
fn negative_coordinates_work_without_an_equals_sign() {
    let home = temp_home("negative-coordinates");
    let output = run_in(
        &home,
        &[
            "--lat",
            "-33.4489",
            "--lon",
            "-70.6693",
            "--tz",
            "America/Santiago",
            "--date",
            "2026-06-15",
            "--no-save",
        ],
    );
    assert!(output.status.success(), "{}", stderr(&output));
    assert!(stdout(&output).contains("Lat,Lon~-33.449,-70.669"));
}

#[test]
fn latitude_without_longitude_is_rejected_instead_of_ignored() {
    let home = temp_home("lat-only");
    // A saved location must not be used silently in place of the partial one.
    let saved = run_in(&home, &["--city", "Tucson", "--no-prompt"]);
    assert!(saved.status.success());

    let output = run_in(&home, &["--lat", "40.7", "--no-prompt", "--no-save"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(stderr(&output).contains("--lon"), "{}", stderr(&output));
    assert!(stdout(&output).is_empty());
}

#[test]
fn ignored_timezone_flag_is_reported() {
    let home = temp_home("tz-ignored");
    let output = run_in(
        &home,
        &[
            "--city",
            "Boston",
            "--tz",
            "Asia/Tokyo",
            "--date",
            "2026-06-15",
            "--no-save",
        ],
    );
    assert!(output.status.success());
    assert!(
        stderr(&output).contains("warning: --tz is ignored"),
        "{}",
        stderr(&output)
    );
    assert!(stdout(&output).contains("America/New_York"));
}

/// Apia keeps UTC+13 at 171.8°W, a calendar day away from its solar day.
/// Reports for a date must describe that date, and `--next` must not skip
/// the event that is still ahead today.
#[test]
fn date_line_city_reports_events_for_the_requested_date() {
    let home = temp_home("date-line");
    let base = ["--city", "Apia", "--date", "2026-11-09", "--no-save"];

    let output = run_in(&home, &[&base[..], &["--json"]].concat());
    assert!(output.status.success(), "{}", stderr(&output));
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let events = &report["sun"]["events"];
    for event in ["sunrise", "solar_noon", "sunset", "astronomical_dusk"] {
        let time = events[event].as_str().unwrap();
        assert!(time.starts_with("2026-11-09 "), "{event}: {time}");
    }

    // Searching from local noon, sunset is about six and a half hours away.
    let output = run_in(
        &home,
        &[&base[..], &["--next", "sunset", "--format", "local"]].concat(),
    );
    assert!(output.status.success(), "{}", stderr(&output));
    assert!(
        stdout(&output).starts_with("2026-11-09 18:31"),
        "{}",
        stdout(&output)
    );

    let output = run_in(
        &home,
        &[&base[..], &["--next", "sunrise", "--format", "local"]].concat(),
    );
    assert!(
        stdout(&output).starts_with("2026-11-10 05:5"),
        "{}",
        stdout(&output)
    );
}

/// `--help` advertises negative (astronomical) years for calendar ranges.
#[test]
fn calendar_accepts_negative_years() {
    let home = temp_home("negative-years");
    let output = run_in(
        &home,
        &[
            "--city",
            "Boston",
            "--calendar",
            "--calendar-format",
            "json",
            "--calendar-start",
            "-0999-01-01",
            "--calendar-end",
            "-0999-01-02",
        ],
    );
    assert!(output.status.success(), "{}", stderr(&output));
    let calendar: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(calendar["days"].as_array().unwrap().len(), 2);
    assert_eq!(calendar["days"][0]["date"], "-0999-01-01");
    assert!(
        !home.join(".solunatus.json").exists(),
        "calendar generation must not save settings"
    );
}

#[test]
fn invalid_config_directory_does_not_fall_back_to_home() {
    let home = temp_home("invalid-override");
    for directory in ["", "relative-config"] {
        let output = home
            .command()
            .env("SOLUNATUS_CONFIG_DIR", directory)
            .args(["--city", "Tucson", "--no-prompt"])
            .output()
            .unwrap();
        assert!(output.status.success(), "{}", stderr(&output));
        assert!(stderr(&output).contains("SOLUNATUS_CONFIG_DIR must be an absolute"));
        assert!(!home.join(".solunatus.json").exists());
    }
}

#[test]
fn explicit_config_directory_takes_precedence_over_home() {
    let config = temp_home("selected-config");
    let home = temp_home("decoy-home");
    let sentinel = "{ personal settings must not be read or overwritten";
    std::fs::write(home.join(".solunatus.json"), sentinel).unwrap();
    let output = config
        .command()
        .env("HOME", home.as_ref())
        .args(["--city", "Tucson", "--no-prompt"])
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", stderr(&output));
    assert!(!stderr(&output).contains("ignoring saved settings"));
    assert!(config.join(".solunatus.json").exists());
    assert_eq!(
        std::fs::read_to_string(home.join(".solunatus.json")).unwrap(),
        sentinel
    );
}

#[test]
fn ambiguous_city_warns_without_changing_legacy_choice_or_json() {
    let home = temp_home("ambiguous-city");
    let output = run_in(&home, &["--city", "Portland", "--json", "--no-save"]);
    assert!(output.status.success(), "{}", stderr(&output));
    assert!(stderr(&output).contains("city 'Portland' is ambiguous"));
    assert!(stderr(&output).contains("Portland, ME, US"));
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["location"]["timezone"], "America/Los_Angeles");

    let output = run_in(
        &home,
        &["--city", " portland, me, us ", "--json", "--no-save"],
    );
    assert!(output.status.success(), "{}", stderr(&output));
    assert!(!stderr(&output).contains("ambiguous"));
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["location"]["timezone"], "America/New_York");
}

#[test]
fn ambiguous_or_unknown_qualified_city_fails_without_saving() {
    let home = temp_home("bad-qualified-city");
    for query in [
        "Columbus, US",
        "Columbus, America/New_York",
        "Portland, ZZ",
        "Portland,",
    ] {
        let output = run_in(&home, &["--city", query, "--no-prompt"]);
        assert!(!output.status.success(), "{query}");
        assert!(!home.join(".solunatus.json").exists());
    }
}

#[test]
fn coordinate_utc_default_warns_and_explicit_timezone_does_not() {
    let home = temp_home("utc-default-warning");
    // A saved timezone must not silently leak into new manual coordinates.
    let output = run_in(&home, &["--city", "Boston", "--no-prompt"]);
    assert!(output.status.success(), "{}", stderr(&output));
    let base = ["--lat", "45.5", "--lon", "-122.6", "--no-prompt"];
    let output = run_in(&home, &base);
    assert!(output.status.success(), "{}", stderr(&output));
    assert!(stderr(&output).contains("without --tz uses UTC"));
    let saved: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(home.join(".solunatus.json")).unwrap())
            .unwrap();
    assert_eq!(saved["tz"], "UTC");
    for timezone in ["UTC", "America/Los_Angeles"] {
        let output = run_in(&home, &[&base[..], &["--tz", timezone]].concat());
        assert!(output.status.success(), "{}", stderr(&output));
        assert!(!stderr(&output).contains("without --tz"));
    }
    let output = run_in(&home, &[&base[..], &["--city", "Boston"]].concat());
    assert!(output.status.success(), "{}", stderr(&output));
    assert!(!stderr(&output).contains("without --tz"));
}

#[cfg(feature = "ai-insights")]
#[test]
fn invalid_saved_ai_refresh_does_not_block_offline_outputs_or_modify_preferences() {
    let home = temp_home("invalid-ai-refresh");
    let config = r#"{"lat":42.36,"lon":-71.06,"tz":"America/New_York","ai":{"enabled":true,"refresh_minutes":999}}"#;
    std::fs::write(home.join(".solunatus.json"), config).unwrap();
    for mode in [
        &["--night"][..],
        &["--next", "sunrise"],
        &["--json"],
        &["--no-prompt"],
        &[
            "--calendar",
            "--calendar-start",
            "2026-11-10",
            "--calendar-end",
            "2026-11-10",
        ],
    ] {
        let mut args = vec!["--city", "Boston", "--no-save"];
        args.extend_from_slice(mode);
        let output = run_in(&home, &args);
        assert!(
            output.status.success(),
            "{mode:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            std::fs::read_to_string(home.join(".solunatus.json")).unwrap(),
            config
        );
    }
}
