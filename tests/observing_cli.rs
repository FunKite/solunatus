//! Exercise the actual one-shot CLI, including feature-minimal builds.
use std::process::Command;

fn run(extra: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_solunatus"))
        .args([
            "--city",
            "New York",
            "--date",
            "2026-09-15",
            "--no-save",
            "--tonight",
        ])
        .args(extra)
        .output()
        .unwrap()
}

#[test]
fn tonight_outputs_a_report_and_exits_without_a_terminal() {
    let output = run(&[]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.starts_with("SOLUNATUS / NIGHT PLAN\n"));
    assert!(text.contains("FIRST MOON-FREE DARK WINDOW"));
    assert!(text.contains("Night of 2026-09-15"));
    assert!(
        !text.contains('\x1b'),
        "one-shot output must not enter terminal mode"
    );
}

#[test]
fn tonight_json_is_clean_and_dates_cross_midnight() {
    let output = run(&["--json"]);
    assert!(output.status.success());
    let plan: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(plan["night_of"], "2026-09-15");
    assert!(
        plan["astronomical_dawn"]
            .as_str()
            .unwrap()
            .starts_with("2026-09-16")
    );
    assert!(plan["first_moon_free_window"].is_object());
}

#[test]
fn tonight_rejects_conflicting_output_modes() {
    for flags in [&["--watch"][..], &["--next", "sunrise"], &["--calendar"]] {
        let output = run(flags);
        assert_eq!(output.status.code(), Some(2));
        assert!(String::from_utf8_lossy(&output.stderr).contains("cannot be used with"));
    }
}
