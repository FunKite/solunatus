//! Exercise the actual one-shot CLI, including feature-minimal builds.
use std::process::Command;

fn run_with_flag(flag: &str, extra: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_solunatus"))
        .args([
            "--city",
            "New York",
            "--date",
            "2026-09-15",
            "--no-save",
            flag,
        ])
        .args(extra)
        .output()
        .unwrap()
}

fn run(extra: &[&str]) -> std::process::Output {
    run_with_flag("--night", extra)
}

#[test]
fn night_outputs_a_report_and_exits_without_a_terminal() {
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
fn night_json_is_clean_and_dates_cross_midnight() {
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
fn night_rejects_conflicting_output_modes() {
    let mut conflicts = vec![
        vec!["--watch"],
        vec!["--next", "sunrise"],
        vec!["--calendar"],
    ];
    if cfg!(feature = "usno-validation") {
        conflicts.push(vec!["--validate"]);
    }
    for flag in ["--night", "--tonight"] {
        for flags in &conflicts {
            let output = run_with_flag(flag, flags);
            assert_eq!(output.status.code(), Some(2));
            assert!(String::from_utf8_lossy(&output.stderr).contains("cannot be used with"));
        }
    }
}

#[test]
fn tonight_alias_preserves_text_and_json_for_selected_dates() {
    for date in ["2020-01-15", "2026-10-10"] {
        for format in [vec![], vec!["--json"]] {
            let mut flags = vec!["--date", date];
            flags.extend(format);
            // Supply each date only once; clap deliberately rejects duplicate options.
            let invoke = |flag| {
                Command::new(env!("CARGO_BIN_EXE_solunatus"))
                    .args(["--city", "Tucson", "--no-save", flag])
                    .args(&flags)
                    .output()
                    .unwrap()
            };
            let primary = invoke("--night");
            let alias = invoke("--tonight");
            assert!(primary.status.success());
            assert!(alias.status.success());
            assert_eq!(primary.stdout, alias.stdout);
        }
    }
}

#[test]
fn generated_help_and_completions_expose_night() {
    for flags in [
        vec!["--help"],
        vec!["--completions", "bash"],
        vec!["--manpage"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_solunatus"))
            .args(flags)
            .output()
            .unwrap();
        assert!(output.status.success());
        let text = String::from_utf8(output.stdout).unwrap();
        // Man pages escape hyphens in roff.
        assert!(text.replace("\\-", "-").contains("--night"));
    }
}
