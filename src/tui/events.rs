// Event handling for TUI

#[cfg(feature = "ai-insights")]
use super::app::AiConfigField;
use super::app::{App, AppMode, CalendarField};
use crate::location_source::LocationSource;
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use std::time::Duration;

pub fn handle_events(app: &mut App, timeout: Duration) -> Result<()> {
    if event::poll(timeout)?
        && let Event::Key(key) = event::read()?
    {
        handle_key_event(app, key)?;
    }
    Ok(())
}

fn handle_key_event(app: &mut App, key: KeyEvent) -> Result<()> {
    match app.mode {
        AppMode::Watch => handle_watch_mode_keys(app, key),
        AppMode::Settings => handle_settings_keys(app, key),
        AppMode::CityPicker => handle_city_picker_keys(app, key),
        AppMode::LocationInput => handle_location_input_keys(app, key),
        #[cfg(feature = "ai-insights")]
        AppMode::AiConfig => handle_ai_config_keys(app, key),
        #[cfg(not(feature = "ai-insights"))]
        AppMode::AiConfig => {
            // AI config mode not available without ai-insights feature
            app.mode = AppMode::Watch;
            Ok(())
        }
        AppMode::Calendar => handle_calendar_keys(app, key),
        AppMode::Reports => handle_reports_keys(app, key),
    }
}

fn handle_watch_mode_keys(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Char('q') | KeyCode::Char('Q') => {
            app.should_quit = true;
        }
        KeyCode::Char('s') | KeyCode::Char('S') => {
            // Open settings menu
            app.open_settings();
        }
        KeyCode::Char('r') | KeyCode::Char('R') => {
            // Open reports menu
            app.mode = AppMode::Reports;
            app.reports_selected_item = super::app::ReportsMenuItem::Calendar;
        }
        #[cfg(feature = "ai-insights")]
        KeyCode::Char('f') | KeyCode::Char('F') if app.ai_config.enabled => {
            // Fetch AI insights manually
            app.refresh_ai_insights();
            app.set_status_message("Refreshing AI insights...");
        }
        _ => {}
    }
    Ok(())
}

fn handle_settings_keys(app: &mut App, key: KeyEvent) -> Result<()> {
    use super::app::SettingsField;

    match key.code {
        KeyCode::Esc => {
            app.mode = AppMode::Watch;
            app.settings_draft.clear_error();
        }
        KeyCode::Enter => {
            let current = app.settings_draft.current_field();

            // Handle location mode - open appropriate picker/input
            if current == SettingsField::LocationMode {
                use crate::config::LocationMode;
                match app.settings_draft.location_mode {
                    LocationMode::City => {
                        // Open city picker
                        app.mode = AppMode::CityPicker;
                        app.city_search.clear();
                        app.city_results.clear();
                        return Ok(());
                    }
                    LocationMode::Manual => {
                        // Open manual location input
                        app.mode = AppMode::LocationInput;
                        app.location_input_draft = crate::tui::app::LocationInputDraft::new();
                        return Ok(());
                    }
                }
            }

            // Apply settings
            match app.apply_settings_changes() {
                Ok(()) => {
                    app.mode = AppMode::Watch;
                    // Save configuration
                    let config = app.build_config();
                    let _ = config.save();
                    app.should_save = false;
                    app.set_status_message("Settings saved and applied");
                }
                Err(e) => {
                    app.settings_draft.set_error(e.to_string());
                }
            }
        }
        KeyCode::Tab | KeyCode::Down => {
            let previous = app.settings_draft.current_field();
            app.settings_draft.next_field();
            // Probe AI server when navigating away from AI server field
            #[cfg(feature = "ai-insights")]
            if previous == SettingsField::AiServer && app.settings_draft.ai_enabled {
                app.probe_ai_server_for_settings();
            }
        }
        KeyCode::BackTab | KeyCode::Up => {
            let previous = app.settings_draft.current_field();
            app.settings_draft.prev_field();
            // Probe AI server when navigating away from AI server field
            #[cfg(feature = "ai-insights")]
            if previous == SettingsField::AiServer && app.settings_draft.ai_enabled {
                app.probe_ai_server_for_settings();
            }
        }
        #[cfg(feature = "ai-insights")]
        KeyCode::Left if app.settings_draft.current_field() == SettingsField::AiModel => {
            app.cycle_ai_model_in_settings(-1);
        }
        KeyCode::Left => {}
        #[cfg(feature = "ai-insights")]
        KeyCode::Right if app.settings_draft.current_field() == SettingsField::AiModel => {
            app.cycle_ai_model_in_settings(1);
        }
        KeyCode::Right => {}
        #[cfg(feature = "ai-insights")]
        KeyCode::Char('[') if app.settings_draft.current_field() == SettingsField::AiModel => {
            app.cycle_ai_model_in_settings(-1);
        }
        KeyCode::Char('[') => {
            app.settings_draft.input_char('[');
        }
        #[cfg(feature = "ai-insights")]
        KeyCode::Char(']') if app.settings_draft.current_field() == SettingsField::AiModel => {
            app.cycle_ai_model_in_settings(1);
        }
        KeyCode::Char(']') => {
            app.settings_draft.input_char(']');
        }
        KeyCode::Char(' ') => {
            let current = app.settings_draft.current_field();
            match current {
                SettingsField::LocationMode => {
                    app.settings_draft.cycle_location_mode();
                }
                SettingsField::TimeSyncEnabled
                | SettingsField::ShowLocationDate
                | SettingsField::ShowEvents
                | SettingsField::ShowPositions
                | SettingsField::ShowMoon
                | SettingsField::ShowLunarPhases
                | SettingsField::NightMode => {
                    app.settings_draft.toggle_current_bool();
                }
                #[cfg(feature = "ai-insights")]
                SettingsField::AiEnabled => {
                    let was_enabled = app.settings_draft.ai_enabled;
                    app.settings_draft.toggle_current_bool();
                    // Probe server when enabling AI
                    if !was_enabled && app.settings_draft.ai_enabled {
                        app.probe_ai_server_for_settings();
                    } else if was_enabled && !app.settings_draft.ai_enabled {
                        // Reset AI server status when disabling
                        app.settings_draft.ai_server_status =
                            crate::tui::app::AiServerStatus::Unknown;
                        app.settings_draft.ai_models.clear();
                        app.settings_draft.ai_model_index = None;
                    }
                }
                _ => {}
            }
        }
        KeyCode::Char('d') | KeyCode::Char('D') => {
            // Load defaults
            app.reset_settings_to_defaults();
            app.set_status_message("Loaded default settings");
        }
        KeyCode::Backspace => {
            app.settings_draft.backspace();
        }
        KeyCode::Char(c) => {
            app.settings_draft.input_char(c);
        }
        _ => {}
    }
    Ok(())
}

fn handle_city_picker_keys(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Esc => {
            // Check if we came from settings
            if app.settings_draft.location_mode == crate::config::LocationMode::City {
                app.mode = AppMode::Settings;
            } else {
                app.mode = AppMode::Watch;
            }
            app.city_search.clear();
            app.city_results.clear();
        }
        KeyCode::Enter => {
            app.select_current_city();
            // Check if we should return to settings
            if app.settings_draft.location_mode == crate::config::LocationMode::City {
                app.mode = AppMode::Settings;
            }
        }
        KeyCode::Up => {
            app.select_prev_city();
        }
        KeyCode::Down => {
            app.select_next_city();
        }
        KeyCode::Backspace => {
            app.city_search.pop();
            app.update_city_search(&app.city_search.clone());
        }
        KeyCode::Char(c) => {
            app.city_search.push(c);
            app.update_city_search(&app.city_search.clone());
        }
        _ => {}
    }
    Ok(())
}

fn handle_location_input_keys(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Esc => {
            // Check if we came from settings
            if app.settings_draft.location_mode == crate::config::LocationMode::Manual {
                app.mode = AppMode::Settings;
            } else {
                app.mode = AppMode::Watch;
            }
            app.location_input_draft = super::app::LocationInputDraft::new();
        }
        KeyCode::Enter => {
            // Validate and apply location
            match app.location_input_draft.validate() {
                Ok((lat, lon, tz_str)) => {
                    // Parse timezone
                    match tz_str.parse::<chrono_tz::Tz>() {
                        Ok(tz) => {
                            // Validate and create location
                            match crate::astro::Location::new(lat, lon) {
                                Ok(location) => {
                                    // Update app state
                                    app.location = location;
                                    app.timezone = tz;
                                    app.city_name = None;
                                    app.location_source = LocationSource::ManualCli;

                                    // Find nearest city for reference
                                    if let Ok(db) = crate::city::CityDatabase::load()
                                        && let Some((city, distance, bearing)) =
                                            db.find_nearest(lat, lon)
                                    {
                                        let city_display = if let Some(ref state) = city.state {
                                            format!("{},{}", city.name, state)
                                        } else {
                                            city.name.clone()
                                        };
                                        app.nearest_city_info =
                                            Some((city_display, distance, bearing));
                                    }
                                    // Check if we should return to settings
                                    if app.settings_draft.location_mode
                                        == crate::config::LocationMode::Manual
                                    {
                                        app.mode = AppMode::Settings;
                                    } else {
                                        app.mode = AppMode::Watch;
                                    }
                                    app.update_time();
                                    app.reset_cached_data();
                                    #[cfg(feature = "ai-insights")]
                                    {
                                        app.ai_last_refresh = None;
                                        app.ai_outcome = None;
                                    }
                                    app.location_input_draft.clear_error();
                                }
                                Err(e) => {
                                    app.location_input_draft.set_error(e);
                                }
                            }
                        }
                        Err(_) => {
                            app.location_input_draft
                                .set_error(format!("Invalid timezone: {}", tz_str));
                        }
                    }
                }
                Err(e) => {
                    app.location_input_draft.set_error(e.to_string());
                }
            }
        }
        KeyCode::Tab | KeyCode::Down => {
            app.location_input_draft.next_field();
        }
        KeyCode::BackTab | KeyCode::Up => {
            app.location_input_draft.prev_field();
        }
        KeyCode::Backspace | KeyCode::Delete => {
            app.location_input_draft.backspace();
        }
        KeyCode::Char(c) => {
            app.location_input_draft.input_char(c);
        }
        _ => {}
    }
    Ok(())
}

fn handle_calendar_keys(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Esc => {
            app.calendar_draft.reset(app.current_time);
            app.calendar_draft.clear_error();
            app.mode = AppMode::Watch;
        }
        KeyCode::Enter => match app.apply_calendar_generation() {
            Ok(path) => {
                app.calendar_draft.clear_error();
                app.mode = AppMode::Watch;
                app.set_status_message(format!("Calendar saved → {}", path));
            }
            Err(err) => {
                app.calendar_draft.set_error(err.to_string());
            }
        },
        KeyCode::Tab | KeyCode::Down => {
            app.calendar_draft.next_field();
        }
        KeyCode::BackTab | KeyCode::Up => {
            app.calendar_draft.prev_field();
        }
        KeyCode::Left if app.calendar_draft.current_field() == CalendarField::Format => {
            app.calendar_draft.cycle_format(-1);
        }
        KeyCode::Left => {}
        KeyCode::Right if app.calendar_draft.current_field() == CalendarField::Format => {
            app.calendar_draft.cycle_format(1);
        }
        KeyCode::Right => {}
        KeyCode::Char(' ') => {
            if app.calendar_draft.current_field() == CalendarField::Format {
                app.calendar_draft.cycle_format(1);
            } else {
                app.calendar_draft.input_char(' ');
            }
        }
        KeyCode::Backspace | KeyCode::Delete => {
            app.calendar_draft.backspace();
        }
        KeyCode::Char(c) => {
            if app.calendar_draft.current_field() == CalendarField::Format {
                match c {
                    'h' | 'H' => {
                        app.calendar_draft
                            .set_format(crate::calendar::CalendarFormat::Html);
                    }
                    'j' | 'J' => {
                        app.calendar_draft
                            .set_format(crate::calendar::CalendarFormat::Json);
                    }
                    _ => {}
                }
            } else {
                app.calendar_draft.input_char(c);
            }
        }
        _ => {}
    }
    Ok(())
}

#[cfg(feature = "ai-insights")]
fn handle_ai_config_keys(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Esc => {
            app.ai_config_draft.sync_from(&app.ai_config);
            app.mode = AppMode::Watch;
        }
        KeyCode::Enter => match app.apply_ai_config_changes() {
            Ok(_) => {
                app.ai_config_draft.clear_error();
                app.mode = AppMode::Watch;
            }
            Err(err) => {
                app.ai_config_draft.set_error(err.to_string());
            }
        },
        KeyCode::Up => app.retreat_ai_field(),
        KeyCode::Down => app.advance_ai_field(),
        KeyCode::Tab => app.advance_ai_field(),
        KeyCode::BackTab => app.retreat_ai_field(),
        KeyCode::Char(' ') => {
            if app.ai_config_draft.current_field() == AiConfigField::Enabled {
                app.toggle_ai_enabled();
            } else if app.ai_config_draft.current_field() == AiConfigField::RefreshMode {
                app.ai_config_draft.toggle_refresh_mode();
            } else {
                app.ai_config_draft.input_char(' ');
            }
        }
        KeyCode::Char('+') | KeyCode::Char('=') => {
            if app.ai_config_draft.current_field() == AiConfigField::RefreshMinutes {
                app.ai_config_draft.bump_refresh(1);
            } else {
                app.ai_config_draft.input_char('+');
            }
        }
        KeyCode::Char('-') | KeyCode::Char('_') => {
            if app.ai_config_draft.current_field() == AiConfigField::RefreshMinutes {
                app.ai_config_draft.bump_refresh(-1);
            } else {
                app.ai_config_draft.input_char('-');
            }
        }
        KeyCode::Left => {
            if app.ai_config_draft.current_field() == AiConfigField::RefreshMode {
                app.ai_config_draft.toggle_refresh_mode();
            } else {
                app.cycle_ai_model(-1);
            }
        }
        KeyCode::Right => {
            if app.ai_config_draft.current_field() == AiConfigField::RefreshMode {
                app.ai_config_draft.toggle_refresh_mode();
            } else {
                app.cycle_ai_model(1);
            }
        }
        KeyCode::Char('[') => app.cycle_ai_model(-1),
        KeyCode::Char(']') => app.cycle_ai_model(1),
        KeyCode::Backspace | KeyCode::Delete => app.ai_config_draft.backspace(),
        KeyCode::Char(c) => app.ai_config_draft.input_char(c),
        _ => {}
    }
    Ok(())
}

fn handle_reports_keys(app: &mut App, key: KeyEvent) -> Result<()> {
    use super::app::ReportsMenuItem;

    match key.code {
        KeyCode::Esc => {
            app.mode = AppMode::Watch;
        }
        KeyCode::Up => {
            app.reports_selected_item = match app.reports_selected_item {
                ReportsMenuItem::Calendar => ReportsMenuItem::Benchmark,
                ReportsMenuItem::UsnoValidation => ReportsMenuItem::Calendar,
                ReportsMenuItem::Benchmark => ReportsMenuItem::UsnoValidation,
            };
        }
        KeyCode::Down => {
            app.reports_selected_item = match app.reports_selected_item {
                ReportsMenuItem::Calendar => ReportsMenuItem::UsnoValidation,
                ReportsMenuItem::UsnoValidation => ReportsMenuItem::Benchmark,
                ReportsMenuItem::Benchmark => ReportsMenuItem::Calendar,
            };
        }
        KeyCode::Enter => {
            match app.reports_selected_item {
                ReportsMenuItem::Calendar => {
                    app.open_calendar_generator();
                }
                ReportsMenuItem::UsnoValidation => {
                    #[cfg(feature = "usno-validation")]
                    {
                        // Generate USNO validation report
                        app.mode = AppMode::Watch;
                        let now_tz = app.current_time.with_timezone(&app.timezone);

                        match crate::usno_validation::generate_validation_report(
                            &app.location,
                            &app.timezone,
                            app.city_name.clone(),
                            &now_tz,
                        ) {
                            Ok(report) => {
                                let html = crate::usno_validation::generate_html_report(&report);
                                let filename = format!(
                                    "solunatus-usno-validation-{}.html",
                                    now_tz.format("%Y%m%d-%H%M%S")
                                );

                                match std::fs::write(&filename, html) {
                                    Ok(_) => {
                                        app.set_status_message(format!(
                                            "USNO validation report saved → {}",
                                            filename
                                        ));
                                    }
                                    Err(e) => {
                                        app.set_status_message(format!(
                                            "Error saving report: {}",
                                            e
                                        ));
                                    }
                                }
                            }
                            Err(e) => {
                                app.set_status_message(format!("Error generating report: {}", e));
                            }
                        }
                    }
                    #[cfg(not(feature = "usno-validation"))]
                    {
                        app.mode = AppMode::Watch;
                        app.set_status_message(
                            "USNO validation not available. Rebuild with: cargo build --features usno-validation".to_string()
                        );
                    }
                }
                ReportsMenuItem::Benchmark => {
                    // Run performance benchmark
                    app.mode = AppMode::Watch;
                    app.set_status_message("Running benchmark across all cities...".to_string());

                    // Run benchmark (this may take a few seconds)
                    let result = crate::benchmark::run_benchmark();

                    let now_tz = app.current_time.with_timezone(&app.timezone);
                    let html = crate::benchmark::generate_html_report(&result);
                    let filename = format!(
                        "solunatus-benchmark-{}.html",
                        now_tz.format("%Y%m%d-%H%M%S")
                    );

                    match std::fs::write(&filename, html) {
                        Ok(_) => {
                            app.set_status_message(format!(
                                "Benchmark complete: {} cities in {:.2}s → {}",
                                result.total_cities,
                                result.total_duration_ms as f64 / 1000.0,
                                filename
                            ));
                        }
                        Err(e) => {
                            app.set_status_message(format!("Error saving benchmark report: {}", e));
                        }
                    }
                }
            }
        }
        _ => {}
    }
    Ok(())
}
