//! Moon phases example: Calculate lunar phases for a month
//!
//! Run with:
//! ```bash
//! cargo run --example moon_phases
//! ```

use chrono::Datelike;
use chrono::Local;
use solunatus::prelude::*;

fn main() {
    println!("=== Solunatus Library - Lunar Phases ===\n");

    let now = Local::now();
    let year = now.year();
    let month = now.month();

    println!("Lunar phases for {}-{:02}\n", year, month);

    // Get all lunar phases for current month
    match get_lunar_phases_for_month(year, month) {
        Ok(phases) => {
            println!("Found {} lunar phases:\n", phases.len());

            for phase in phases {
                let emoji = match phase.phase_type {
                    LunarPhaseType::NewMoon => "🌑",
                    LunarPhaseType::FirstQuarter => "🌓",
                    LunarPhaseType::FullMoon => "🌕",
                    LunarPhaseType::LastQuarter => "🌗",
                };

                let name = match phase.phase_type {
                    LunarPhaseType::NewMoon => "New Moon",
                    LunarPhaseType::FirstQuarter => "First Quarter",
                    LunarPhaseType::FullMoon => "Full Moon",
                    LunarPhaseType::LastQuarter => "Last Quarter",
                };

                println!(
                    "  {} {:15} - {}",
                    emoji,
                    name,
                    phase.datetime.format("%Y-%m-%d %H:%M UTC")
                );
            }
        }
        Err(e) => {
            eprintln!("Error calculating lunar phases: {}", e);
        }
    }

    // Show next 3 months of full moons
    println!("\n--- Upcoming Full Moons ---\n");

    for offset in 0..3 {
        let target_month = month + offset;
        let target_year = if target_month > 12 { year + 1 } else { year };
        let target_month = ((target_month - 1) % 12) + 1;

        if let Ok(phases) = get_lunar_phases_for_month(target_year, target_month) {
            for phase in phases {
                if phase.phase_type == LunarPhaseType::FullMoon {
                    println!("  🌕 {}", phase.datetime.format("%Y-%m-%d %H:%M UTC"));
                }
            }
        }
    }

    println!("\n✓ Lunar phase calculations complete!");
}
