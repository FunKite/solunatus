//! One-shot CLI behavior: `--date` handling, deprecated flags, and saved
//! configuration safety. Each test runs with an isolated home directory.
use std::path::PathBuf;
use std::process::{Command, Output};

/// A fresh, empty home directory unique to one test.
fn temp_home(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "solunatus-cli-test-{}-{}",
        name,
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn run_in(home: &PathBuf, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_solunatus"))
        .args(args)
        .env("HOME", home)
        .env("SOLUNATUS_SKIP_TIME_SYNC", "1")
        .output()
        .unwrap()
}

#[cfg(unix)]
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

#[cfg(unix)]
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

#[cfg(unix)]
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

#[cfg(unix)]
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

#[cfg(unix)]
#[test]
fn readable_config_is_still_saved() {
    let home = temp_home("save-config");
    let output = run_in(&home, &["--city", "Tucson", "--no-prompt"]);
    assert!(output.status.success());
    let saved = std::fs::read_to_string(home.join(".solunatus.json")).unwrap();
    assert!(saved.contains("\"city\": \"Tucson\""));
}
