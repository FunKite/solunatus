// UI rendering

use super::app::{AiConfigField, AiServerStatus, App, CalendarField, LocationInputField, SettingsField};
use crate::astro::*;
use crate::time_sync;
use chrono::{Offset, Utc};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};
use std::borrow::Cow;

fn get_footer_instructions(ai_enabled: bool) -> Vec<&'static str> {
    let mut instructions = vec!["q quit", "s settings", "r reports"];
    if ai_enabled {
        instructions.push("f fetch AI");
    }
    instructions
}

pub fn render(f: &mut Frame, app: &App) {
    match app.mode {
        super::app::AppMode::Watch => {
            let area = f.area();
            let footer_height = footer_line_count(area.width, app.ai_config.enabled);
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Title
                    Constraint::Min(10),   // Main content
                    Constraint::Length(footer_height),
                ])
                .split(area);

            render_title(f, chunks[0], app);
            render_main_content(f, chunks[1], app);
            render_footer(f, chunks[2], app);
        }
        super::app::AppMode::Settings => {
            render_settings(f, app);
        }
        super::app::AppMode::CityPicker => {
            render_city_picker(f, app);
        }
        super::app::AppMode::LocationInput => {
            render_location_input(f, app);
        }
        super::app::AppMode::AiConfig => {
            render_ai_config(f, app);
        }
        super::app::AppMode::Calendar => {
            render_calendar_generator(f, app);
        }
        super::app::AppMode::Reports => {
            render_reports_menu(f, app);
        }
    }
}

fn get_color(app: &App, default_color: Color) -> Color {
    if app.night_mode {
        Color::Red
    } else {
        default_color
    }
}

fn border_style(app: &App) -> Style {
    if app.night_mode {
        Style::default().fg(Color::Red)
    } else {
        Style::default().fg(Color::White)
    }
}

fn bordered_block<'a>(app: &App) -> Block<'a> {
    Block::default()
        .borders(Borders::ALL)
        .border_style(border_style(app))
}

fn label_with_symbol(app: &App, symbol: &str, text: String) -> String {
    if app.night_mode || symbol.is_empty() {
        text
    } else {
        format!("{} {}", symbol, text)
    }
}

fn symbol_prefix<'a>(app: &App, symbol: &'a str) -> &'a str {
    if app.night_mode {
        ""
    } else {
        symbol
    }
}

fn strip_symbolic_prefix(text: &str) -> &str {
    if let Some((_, rest)) = text.split_once(' ') {
        rest.trim_start()
    } else {
        text
    }
}

fn sanitized_event_label<'a>(app: &App, label: &'a str) -> Cow<'a, str> {
    if app.night_mode {
        Cow::Owned(strip_symbolic_prefix(label).to_string())
    } else {
        Cow::Borrowed(label)
    }
}

fn render_title(f: &mut Frame, area: Rect, app: &App) {
    let title = Paragraph::new("Solunatus 0.2.2 — github.com/FunKite/solunatus")
        .style(
            Style::default()
                .fg(get_color(app, Color::Cyan))
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(bordered_block(app));

    f.render_widget(title, area);
}

fn render_main_content(f: &mut Frame, area: Rect, app: &App) {
    let now_tz = app.current_time.with_timezone(&app.timezone);
    let sun_pos = app.positions_cache.sun;
    let moon_pos_position = app.positions_cache.moon;
    let moon_overview_details = app.moon_overview_cache;
    let moon_overview = moon_overview_details.moon;
    let lunar_phases = &app.lunar_phases_cache;

    // Build the display text
    let mut lines = Vec::new();
    let mut sections_rendered = 0usize;

    if app.show_location_date {
        lines.push(Line::from(vec![Span::styled(
            "— Location & Date —",
            Style::default()
                .fg(get_color(app, Color::Yellow))
                .add_modifier(Modifier::BOLD),
        )]));

        // Combine Lat/Lon and Place/Nearest City on one line
        let location_text = if let Some(ref city) = app.city_name {
            if app.night_mode {
                format!(
                    "Lat,Lon~{:.3},{:.3}  Place: {}",
                    app.location.latitude.value(),
                    app.location.longitude.value(),
                    city
                )
            } else {
                format!(
                    "Lat,Lon~{:.3},{:.3}  🏙️ Place: {}",
                    app.location.latitude.value(),
                    app.location.longitude.value(),
                    city
                )
            }
        } else if let Some((ref nearest_city, distance, bearing)) = app.nearest_city_info {
            // Show nearest city with distance and bearing
            use crate::city::bearing_to_compass;
            let compass = bearing_to_compass(bearing);
            if app.night_mode {
                format!(
                    "Lat,Lon~{:.3},{:.3}  {}km {} to {}",
                    app.location.latitude.value(),
                    app.location.longitude.value(),
                    distance.round() as i32,
                    compass,
                    nearest_city
                )
            } else {
                format!(
                    "Lat,Lon~{:.3},{:.3}🧭 {}km {} to {}",
                    app.location.latitude.value(),
                    app.location.longitude.value(),
                    distance.round() as i32,
                    compass,
                    nearest_city
                )
            }
        } else {
            format!(
                "Lat,Lon~{:.3},{:.3}",
                app.location.latitude.value(),
                app.location.longitude.value()
            )
        };

        lines.push(Line::from(vec![Span::raw(label_with_symbol(
            app,
            "📍",
            location_text,
        ))]));

        let offset_seconds = now_tz.offset().fix().local_minus_utc();
        let offset_minutes = offset_seconds / 60;
        let sign = if offset_minutes >= 0 { '+' } else { '-' };
        let abs_minutes = offset_minutes.abs();
        let offset_hours = abs_minutes / 60;
        let offset_remaining_minutes = abs_minutes % 60;
        let offset_label = if offset_remaining_minutes == 0 {
            format!("UTC{}{:02}", sign, offset_hours)
        } else {
            format!(
                "UTC{}{:02}:{:02}",
                sign, offset_hours, offset_remaining_minutes
            )
        };
        let timezone_text = if app.night_mode {
            format!(
                "{} {}@{}",
                now_tz.format("%b %d %H:%M:%S"),
                app.timezone.name(),
                offset_label
            )
        } else {
            format!(
                "{} ⌚{}@{}",
                now_tz.format("%b %d %H:%M:%S"),
                app.timezone.name(),
                offset_label
            )
        };

        lines.push(Line::from(vec![Span::raw(label_with_symbol(
            app,
            "📅",
            timezone_text,
        ))]));

        // Only show Time Sync row if enabled
        if !app.time_sync_disabled {
            let countdown_text = app.time_sync_countdown().map(|remaining| {
                let total_secs = remaining.as_secs();
                let minutes = total_secs / 60;
                let seconds = total_secs % 60;
                format!("{:02}:{:02}", minutes, seconds)
            });
            let mut time_sync_text = format!("Sync {}:", app.time_sync.source);
            match (
                app.time_sync.delta,
                app.time_sync.direction(),
                app.time_sync.error_summary(),
            ) {
                (Some(delta), Some(direction), _) => {
                    let dir_symbol = match direction {
                        time_sync::TimeSyncDirection::Ahead => "↑",
                        time_sync::TimeSyncDirection::Behind => "↓",
                        time_sync::TimeSyncDirection::InSync => "✓",
                    };
                    time_sync_text.push_str(&format!(
                        " {} {}",
                        time_sync::format_offset(delta),
                        dir_symbol
                    ));
                }
                (Some(delta), None, _) => {
                    time_sync_text.push_str(&format!(" {}", time_sync::format_offset(delta)));
                }
                (None, _, Some(err)) => {
                    time_sync_text.push_str(&format!(" err {}", err));
                }
                _ => {
                    time_sync_text.push_str(" n/a");
                }
            }
            if let Some(countdown) = countdown_text {
                time_sync_text.push_str(&format!(" ↻{}", countdown));
            }
            lines.push(Line::from(vec![Span::raw(label_with_symbol(
                app,
                "🕒",
                time_sync_text,
            ))]));
        }
        if let Some(status) = app.current_status() {
            lines.push(Line::from(vec![Span::styled(
                format!("{}{}", symbol_prefix(app, "✓ "), status),
                Style::default()
                    .fg(get_color(app, Color::Green))
                    .add_modifier(Modifier::BOLD),
            )]));
        }
        sections_rendered += 1;
    }

    if app.show_events {
        if sections_rendered > 0 {
            lines.push(Line::from(""));
        }
        lines.push(Line::from(vec![Span::styled(
            "— Events —",
            Style::default()
                .fg(get_color(app, Color::Yellow))
                .add_modifier(Modifier::BOLD),
        )]));

        let timed_events = &app.events_cache.entries;
        // Filter events to only show those within ±12 hours of current time (sliding window)
        let window_hours = 12i64;
        let filtered_events: Vec<_> = timed_events
            .iter()
            .filter(|(dt, _)| {
                let delta = dt.signed_duration_since(now_tz).num_hours().abs();
                delta <= window_hours
            })
            .collect();
        let next_event_idx = filtered_events.iter().position(|(dt, _)| *dt > now_tz);

        for (idx, (event_time, event_name)) in filtered_events.iter().enumerate() {
            let time_diff = time_utils::time_until(&now_tz, event_time);
            let time_str = format!("{}", event_time.format("%H:%M:%S"));
            let mut diff_str = time_utils::format_duration_detailed(time_diff);

            if (event_name.contains("Civil dawn") || event_name.contains("Solar noon"))
                && !app.night_mode
            {
                diff_str = format!(" {}", diff_str);
            }

            let marker = if Some(idx) == next_event_idx {
                " (next)"
            } else {
                ""
            };

            let event_label = sanitized_event_label(app, event_name);
            let (event_width, diff_width) = if app.night_mode { (14, 15) } else { (16, 17) };
            lines.push(Line::from(vec![Span::raw(format!(
                "{}  {:<event_width$} {:<diff_width$}{}",
                time_str,
                event_label,
                diff_str,
                marker,
                event_width = event_width,
                diff_width = diff_width
            ))]));
        }
        sections_rendered += 1;
    }

    if app.show_positions {
        if sections_rendered > 0 {
            lines.push(Line::from(""));
        }
        let pos_countdown = app.position_countdown();
        let pos_seconds = pos_countdown.as_secs();
        lines.push(Line::from(vec![Span::styled(
            format!("— Position —  ↻{}s", pos_seconds),
            Style::default()
                .fg(get_color(app, Color::Yellow))
                .add_modifier(Modifier::BOLD),
        )]));
        lines.push(Line::from(vec![Span::raw(label_with_symbol(
            app,
            "☀️",
            format!(
                "Sun:  Alt {:>6.2}°, Az {:>6.2}° {}",
                sun_pos.altitude,
                sun_pos.azimuth,
                coordinates::azimuth_to_compass(sun_pos.azimuth)
            ),
        ))]));
        lines.push(Line::from(vec![Span::raw(label_with_symbol(
            app,
            "🌕",
            format!(
                "Moon: Alt {:>6.2}°, Az {:>6.2}° {}",
                moon_pos_position.altitude,
                moon_pos_position.azimuth,
                coordinates::azimuth_to_compass(moon_pos_position.azimuth)
            ),
        ))]));
        sections_rendered += 1;
    }

    if app.show_moon {
        if sections_rendered > 0 {
            lines.push(Line::from(""));
        }
        let moon_countdown = app.moon_countdown();
        let moon_minutes = moon_countdown.as_secs() / 60;
        let moon_seconds = moon_countdown.as_secs() % 60;
        lines.push(Line::from(vec![Span::styled(
            format!("— Moon —  ↻{:02}:{:02}", moon_minutes, moon_seconds),
            Style::default()
                .fg(get_color(app, Color::Yellow))
                .add_modifier(Modifier::BOLD),
        )]));

        let size_class = if moon_overview.angular_diameter > 33.0 {
            "Near Perigee"
        } else if moon_overview.angular_diameter > 32.0 {
            "Larger than Average"
        } else if moon_overview.angular_diameter > 30.5 {
            "Average"
        } else if moon_overview.angular_diameter > 29.5 {
            "Smaller than Average"
        } else {
            "Near Apogee"
        };

        lines.push(Line::from(vec![Span::raw(label_with_symbol(
            app,
            moon::phase_emoji(moon_overview.phase_angle),
            format!(
                "Phase:           {} (Age {:.1} days)",
                moon::phase_name(moon_overview.phase_angle),
                (moon_overview.phase_angle / 360.0 * 29.53)
            ),
        ))]));
        let trend_label = match moon_overview_details.altitude_trend {
            super::app::MoonAltitudeTrend::Down => "Down",
            super::app::MoonAltitudeTrend::Up => "Up",
        };
        lines.push(Line::from(vec![Span::raw(label_with_symbol(
            app,
            "💡",
            format!(
                "Fraction Illum.: {:.2}% ({})",
                moon_overview.illumination * 100.0,
                trend_label
            ),
        ))]));
        lines.push(Line::from(vec![Span::raw(label_with_symbol(
            app,
            "🔭",
            format!(
                "Apparent size:   {:.2}' ({})",
                moon_overview.angular_diameter, size_class
            ),
        ))]));
        sections_rendered += 1;
    }

    if app.show_lunar_phases {
        if sections_rendered > 0 {
            lines.push(Line::from(""));
        }
        lines.push(Line::from(vec![Span::styled(
            "— Lunar Phases —",
            Style::default()
                .fg(get_color(app, Color::Yellow))
                .add_modifier(Modifier::BOLD),
        )]));

        if lunar_phases.is_empty() {
            lines.push(Line::from(vec![Span::raw(
                "No lunar phase data available.",
            )]));
        } else {
            let now_utc = now_tz.with_timezone(&Utc);
            let future_start = lunar_phases
                .iter()
                .position(|phase| phase.datetime > now_utc)
                .unwrap_or(lunar_phases.len());
            let mut display_phases: Vec<&moon::LunarPhase> = Vec::new();

            let prev_start = future_start.saturating_sub(2);
            display_phases.extend(&lunar_phases[prev_start..future_start]);

            let future_end = (future_start + 2).min(lunar_phases.len());
            display_phases.extend(&lunar_phases[future_start..future_end]);

            for phase in display_phases {
                let phase_emoji = match phase.phase_type {
                    moon::LunarPhaseType::NewMoon => "🌑",
                    moon::LunarPhaseType::FirstQuarter => "🌓",
                    moon::LunarPhaseType::FullMoon => "🌕",
                    moon::LunarPhaseType::LastQuarter => "🌗",
                };
                let phase_name = match phase.phase_type {
                    moon::LunarPhaseType::NewMoon => "New:",
                    moon::LunarPhaseType::FirstQuarter => "First quarter:",
                    moon::LunarPhaseType::FullMoon => "Full:",
                    moon::LunarPhaseType::LastQuarter => "Last quarter:",
                };
                let phase_dt = phase.datetime.with_timezone(&app.timezone);
                let line_text = if app.night_mode {
                    format!("{:<16} {}", phase_name, phase_dt.format("%b %d %H:%M"))
                } else {
                    format!(
                        "{} {:<16} {}",
                        phase_emoji,
                        phase_name,
                        phase_dt.format("%b %d %H:%M")
                    )
                };
                lines.push(Line::from(vec![Span::raw(line_text)]));
            }
        }
        sections_rendered += 1;
    }

    if sections_rendered == 0 {
        lines.push(Line::from(vec![Span::styled(
            "All panels hidden. Use settings (s) to re-enable.",
            Style::default().fg(get_color(app, Color::Gray)),
        )]));
        sections_rendered += 1;
    }

    if app.ai_config.enabled {
        if sections_rendered > 0 {
            lines.push(Line::from(""));
        }

        // Build header with model and countdown on same line
        let mut header_text = "— AI Insights —".to_string();

        match &app.ai_outcome {
            Some(outcome) => {
                // Calculate countdown to next refresh
                let elapsed = app
                    .current_time
                    .with_timezone(&Utc)
                    .signed_duration_since(outcome.updated_at);
                let elapsed_secs = elapsed.num_seconds().max(0) as u64;
                let refresh_secs = app.ai_config.refresh.as_secs();
                let remaining_secs = refresh_secs.saturating_sub(elapsed_secs);
                let minutes = remaining_secs / 60;
                let seconds = remaining_secs % 60;

                header_text.push_str(&format!(
                    "  Model: {}  ↻{:02}:{:02}",
                    outcome.model, minutes, seconds
                ));
            }
            None => {
                header_text.push_str("  Fetching…");
            }
        }

        lines.push(Line::from(vec![Span::styled(
            header_text,
            Style::default()
                .fg(get_color(app, Color::Yellow))
                .add_modifier(Modifier::BOLD),
        )]));

        if let Some(outcome) = &app.ai_outcome {
            if let Some(content) = &outcome.content {
                for line in content.lines() {
                    lines.push(Line::from(Span::raw(line.trim_end().to_string())));
                }
            } else {
                lines.push(Line::from(Span::raw("No insights available.")));
            }

            if let Some(err) = &outcome.error {
                lines.push(Line::from(Span::styled(
                    format!("{}{}", symbol_prefix(app, "⚠️ "), err),
                    Style::default().fg(get_color(app, Color::LightRed)),
                )));
            }
        }

        lines.push(Line::from(""));
    }

    let text = Text::from(lines);
    let paragraph = Paragraph::new(text)
        .style(Style::default().fg(get_color(app, Color::White)))
        .wrap(Wrap { trim: true })
        .block(bordered_block(app));

    f.render_widget(paragraph, area);
}

fn render_footer(f: &mut Frame, area: Rect, app: &App) {
    let footer_instructions = get_footer_instructions(app.ai_config.enabled);
    let mut lines = Vec::new();
    let max_width = area.width.saturating_sub(4) as usize;
    let mut current_line = String::new();

    for entry in footer_instructions {
        let entry_len = entry.len();
        let candidate_len = if current_line.is_empty() {
            entry_len
        } else {
            current_line.len() + 3 + entry_len
        };

        if !current_line.is_empty() && candidate_len > max_width {
            lines.push(Line::from(Span::styled(
                current_line.clone(),
                Style::default().fg(get_color(app, Color::Gray)),
            )));
            current_line.clear();
            current_line.push_str(entry);
        } else {
            if !current_line.is_empty() {
                current_line.push_str(" | ");
            }
            current_line.push_str(entry);
        }
    }

    if !current_line.is_empty() {
        lines.push(Line::from(Span::styled(
            current_line,
            Style::default().fg(get_color(app, Color::Gray)),
        )));
    }

    let footer = Paragraph::new(Text::from(lines))
        .alignment(Alignment::Center)
        .block(bordered_block(app));

    f.render_widget(footer, area);
}

fn footer_line_count(width: u16, ai_enabled: bool) -> u16 {
    let footer_instructions = get_footer_instructions(ai_enabled);
    let max_width = width.saturating_sub(4) as usize;
    if max_width == 0 {
        return 3;
    }

    let mut line_count = 0usize;
    let mut current_len = 0usize;

    for entry in footer_instructions {
        let entry_len = entry.len();
        let candidate_len = if current_len == 0 {
            entry_len
        } else {
            current_len + 3 + entry_len
        };

        if current_len != 0 && candidate_len > max_width {
            line_count += 1;
            current_len = entry_len;
        } else if current_len != 0 {
            current_len += 3 + entry_len;
        } else {
            current_len = entry_len;
        }
    }

    if current_len > 0 {
        line_count += 1;
    }

    (line_count as u16 + 2).max(3)
}

fn render_city_picker(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Length(3), // Search input
            Constraint::Min(10),   // Results list
            Constraint::Length(2), // Footer
        ])
        .split(f.area());

    // Title
    let title = Paragraph::new("City Selector")
        .style(
            Style::default()
                .fg(get_color(app, Color::Cyan))
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(bordered_block(app));
    f.render_widget(title, chunks[0]);

    // Search input
    let search_text = format!("Search: {}", app.city_search);
    let search = Paragraph::new(search_text)
        .style(Style::default().fg(get_color(app, Color::White)))
        .block(bordered_block(app).title("Type to search"));
    f.render_widget(search, chunks[1]);

    // Results list
    let mut lines = Vec::new();
    if app.city_results.is_empty() {
        if app.city_search.is_empty() {
            lines.push(Line::from(Span::styled(
                "Type a city name to search...",
                Style::default().fg(get_color(app, Color::Gray)),
            )));
        } else {
            lines.push(Line::from(Span::styled(
                "No cities found",
                Style::default().fg(get_color(app, Color::Gray)),
            )));
        }
    } else {
        for (idx, city) in app.city_results.iter().enumerate() {
            let style = if idx == app.city_selected {
                Style::default()
                    .fg(get_color(app, Color::Yellow))
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(get_color(app, Color::White))
            };

            let marker = if idx == app.city_selected { "> " } else { "  " };
            let line_text = if let Some(state) = &city.state {
                format!("{}{} ({}, {})", marker, city.name, city.country, state)
            } else {
                format!("{}{} ({})", marker, city.name, city.country)
            };
            lines.push(Line::from(Span::styled(line_text, style)));
        }
    }

    let results = Paragraph::new(lines)
        .style(Style::default().fg(get_color(app, Color::White)))
        .block(bordered_block(app).title("Results"));
    f.render_widget(results, chunks[2]);

    // Footer
    let footer = Paragraph::new("↑/↓: Navigate | Enter: Select | Esc: Cancel")
        .style(Style::default().fg(get_color(app, Color::Gray)))
        .alignment(Alignment::Center);
    f.render_widget(footer, chunks[3]);
}

fn render_location_input(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Length(8), // Input fields
            Constraint::Min(5),    // Help text
            Constraint::Length(2), // Footer
        ])
        .split(f.area());

    // Title
    let title = Paragraph::new("Manual Location Input")
        .style(
            Style::default()
                .fg(get_color(app, Color::Cyan))
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(bordered_block(app));
    f.render_widget(title, chunks[0]);

    // Input fields
    let draft = &app.location_input_draft;
    let current_field = draft.current_field();

    let field_style = |field: LocationInputField| {
        if field == current_field {
            Style::default()
                .fg(get_color(app, Color::Yellow))
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(get_color(app, Color::White))
        }
    };

    let marker = |field: LocationInputField| {
        if field == current_field {
            "► "
        } else {
            "  "
        }
    };

    let lat_display = if draft.latitude.is_empty() {
        "".to_string()
    } else {
        draft.latitude.clone()
    };

    let lon_display = if draft.longitude.is_empty() {
        "".to_string()
    } else {
        draft.longitude.clone()
    };

    let mut input_lines = vec![
        Line::from(vec![
            Span::raw(marker(LocationInputField::Latitude)),
            Span::styled("Latitude:  ", field_style(LocationInputField::Latitude)),
            Span::styled(lat_display, field_style(LocationInputField::Latitude)),
            Span::styled(
                "  (-90 to 90)",
                Style::default().fg(get_color(app, Color::Gray)),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw(marker(LocationInputField::Longitude)),
            Span::styled("Longitude: ", field_style(LocationInputField::Longitude)),
            Span::styled(lon_display, field_style(LocationInputField::Longitude)),
            Span::styled(
                "  (-180 to 180)",
                Style::default().fg(get_color(app, Color::Gray)),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw(marker(LocationInputField::Timezone)),
            Span::styled("Timezone:  ", field_style(LocationInputField::Timezone)),
            Span::styled(&draft.timezone, field_style(LocationInputField::Timezone)),
            Span::styled(
                "  (e.g., America/New_York)",
                Style::default().fg(get_color(app, Color::Gray)),
            ),
        ]),
    ];

    // Add error message if present
    if let Some(error) = &draft.error {
        input_lines.push(Line::from(""));
        input_lines.push(Line::from(Span::styled(
            format!("Error: {}", error),
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )));
    }

    let input_fields = Paragraph::new(input_lines)
        .style(Style::default().fg(get_color(app, Color::White)))
        .block(bordered_block(app).title("Enter Location"));
    f.render_widget(input_fields, chunks[1]);

    // Help text
    let help_text = vec![
        Line::from(Span::styled(
            "Enter your location coordinates:",
            Style::default()
                .fg(get_color(app, Color::Green))
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "All astronomical calculations use sea level (0m elevation) per USNO conventions.",
            Style::default().fg(get_color(app, Color::White)),
        )),
        Line::from(Span::styled(
            "Specify latitude and longitude in decimal degrees (e.g., 42.3834, -71.4162).",
            Style::default().fg(get_color(app, Color::Gray)),
        )),
    ];

    let help = Paragraph::new(help_text)
        .style(Style::default().fg(get_color(app, Color::White)))
        .block(bordered_block(app).title(if app.night_mode { "Info" } else { "ℹ Info" }))
        .wrap(Wrap { trim: true });
    f.render_widget(help, chunks[2]);

    // Footer
    let footer = Paragraph::new("Tab/↑↓: Navigate | Enter: Confirm | Esc: Cancel")
        .style(Style::default().fg(get_color(app, Color::Gray)))
        .alignment(Alignment::Center);
    f.render_widget(footer, chunks[3]);
}

fn render_calendar_generator(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Length(11), // Form
            Constraint::Min(5),     // Guidance
            Constraint::Length(2),  // Footer
        ])
        .split(f.area());

    let title = Paragraph::new("Generate Astronomical Calendar")
        .style(
            Style::default()
                .fg(get_color(app, Color::Cyan))
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(bordered_block(app));
    f.render_widget(title, chunks[0]);

    let draft = &app.calendar_draft;
    let current_field = draft.current_field();

    let field_style = |field: CalendarField| {
        if field == current_field {
            Style::default()
                .fg(get_color(app, Color::Yellow))
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(get_color(app, Color::White))
        }
    };

    let marker = |field: CalendarField| if field == current_field { "► " } else { "  " };

    let start_value = if draft.start.is_empty() {
        Span::styled(
            "YYYY-MM-DD",
            Style::default().fg(get_color(app, Color::Gray)),
        )
    } else {
        Span::styled(draft.start.as_str(), field_style(CalendarField::StartDate))
    };

    let end_value = if draft.end.is_empty() {
        Span::styled(
            "YYYY-MM-DD",
            Style::default().fg(get_color(app, Color::Gray)),
        )
    } else {
        Span::styled(draft.end.as_str(), field_style(CalendarField::EndDate))
    };

    let output_display = if draft.output_path.trim().is_empty() {
        Span::styled(
            "(auto-named on save)",
            Style::default().fg(get_color(app, Color::Gray)),
        )
    } else {
        Span::styled(
            draft.output_path.as_str(),
            field_style(CalendarField::OutputPath),
        )
    };

    let mut lines = vec![
        Line::from(vec![
            Span::raw(marker(CalendarField::StartDate)),
            Span::styled("Start date: ", field_style(CalendarField::StartDate)),
            start_value,
            Span::styled(
                "  (YYYY-MM-DD)",
                Style::default().fg(get_color(app, Color::Gray)),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw(marker(CalendarField::EndDate)),
            Span::styled("End date:   ", field_style(CalendarField::EndDate)),
            end_value,
            Span::styled(
                "  (YYYY-MM-DD)",
                Style::default().fg(get_color(app, Color::Gray)),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw(marker(CalendarField::Format)),
            Span::styled("Format:     ", field_style(CalendarField::Format)),
            Span::styled(
                draft.current_format_label(),
                field_style(CalendarField::Format),
            ),
            Span::styled(
                "  (space/←/→ to toggle)",
                Style::default().fg(get_color(app, Color::Gray)),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw(marker(CalendarField::OutputPath)),
            Span::styled("Output file:", field_style(CalendarField::OutputPath)),
            output_display,
        ]),
    ];

    if let Some(error) = &draft.error {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!("Error: {}", error),
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )));
    }

    let form = Paragraph::new(lines)
        .style(Style::default().fg(get_color(app, Color::White)))
        .block(bordered_block(app).title("Calendar Parameters"));
    f.render_widget(form, chunks[1]);

    let guidance_text = vec![
        Line::from(Span::raw(
            "• Enter BCE years with a leading minus (e.g., -0999-01-01 = 1000 BCE).",
        )),
        Line::from(Span::raw(
            "• Range must fall between 1000 BCE and 3000 CE (inclusive).",
        )),
        Line::from(Span::raw(
            "• Files include sunrise, sunset, twilight, moonrise, moonset, and phase details.",
        )),
    ];

    let guidance = Paragraph::new(guidance_text)
        .style(Style::default().fg(get_color(app, Color::Gray)))
        .wrap(Wrap { trim: false })
        .block(bordered_block(app).title("Tips"));
    f.render_widget(guidance, chunks[2]);

    let footer = Paragraph::new("Enter: Generate | Esc: Cancel | Tab/Shift+Tab: Move")
        .style(Style::default().fg(get_color(app, Color::Gray)))
        .alignment(Alignment::Center);
    f.render_widget(footer, chunks[3]);
}

fn render_ai_config(f: &mut Frame, app: &App) {
    let area = f.area();
    let block = bordered_block(app).title("AI Insights Settings");
    let inner = block.inner(area);
    f.render_widget(block, area);

    let draft = &app.ai_config_draft;
    let current_field = draft.current_field();

    let server_display = if draft.server.trim().is_empty() {
        "<default (localhost)>".to_string()
    } else {
        draft.server.clone()
    };
    let model_display = if draft.model.trim().is_empty() {
        "<pick a model>".to_string()
    } else {
        draft.model.clone()
    };
    let refresh_display = if draft.refresh_minutes.trim().is_empty() {
        "<empty>".to_string()
    } else {
        draft.refresh_minutes.clone()
    };
    let enabled_display = if draft.enabled { "[x] On" } else { "[ ] Off" };
    let refresh_mode_display = draft.refresh_mode;
    let refresh_mode_label = match refresh_mode_display {
        crate::config::AiRefreshMode::AutoAndManual => "Auto & Manual",
        crate::config::AiRefreshMode::ManualOnly => "Manual Only",
    };

    let fields: Vec<(AiConfigField, &str, String)> = vec![
        (
            AiConfigField::Enabled,
            "Enabled",
            enabled_display.to_string(),
        ),
        (AiConfigField::Server, "Server", server_display),
        (AiConfigField::Model, "Model", model_display),
        (
            AiConfigField::RefreshMinutes,
            "Refresh (min)",
            refresh_display,
        ),
        (
            AiConfigField::RefreshMode,
            "Refresh Mode",
            refresh_mode_label.to_string(),
        ),
    ];

    let mut lines = Vec::new();
    lines.push(Line::from(Span::styled(
        "Dial in your Ollama connection, then let the AI narrate what matters.",
        Style::default().fg(get_color(app, Color::Gray)),
    )));
    lines.push(Line::from(""));

    for (idx, (field, label, value)) in fields.iter().enumerate() {
        let is_selected = draft.field_index == idx;
        let prefix = if is_selected { "› " } else { "  " };
        let mut spans = vec![
            Span::styled(prefix, Style::default().fg(get_color(app, Color::Cyan))),
            Span::styled(
                format!("{:<14}", label),
                Style::default()
                    .fg(get_color(app, Color::Yellow))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                value.clone(),
                Style::default()
                    .fg(get_color(app, Color::White))
                    .add_modifier(if is_selected {
                        Modifier::BOLD
                    } else {
                        Modifier::empty()
                    }),
            ),
        ];

        if *field == AiConfigField::Server && draft.server.trim().is_empty() {
            spans.push(Span::styled(
                "  (auto-checks http://localhost:11434)",
                Style::default().fg(get_color(app, Color::Gray)),
            ));
        }

        lines.push(Line::from(spans));
    }

    if draft.enabled {
        lines.push(Line::from(""));
        match &draft.server_status {
            AiServerStatus::Connected { server } => {
                lines.push(Line::from(Span::styled(
                    format!(
                        "{}Connected to {} — {} model{} available",
                        symbol_prefix(app, "✅ "),
                        server,
                        draft.models.len(),
                        if draft.models.len() == 1 { "" } else { "s" }
                    ),
                    Style::default()
                        .fg(get_color(app, Color::LightGreen))
                        .add_modifier(Modifier::BOLD),
                )));
            }
            AiServerStatus::Failed { server, message } => {
                lines.push(Line::from(Span::styled(
                    format!(
                        "{}Unable to reach {} ({}) — edit the server and press Tab to retry.",
                        symbol_prefix(app, "⚠️ "),
                        server,
                        message.replace('\n', " ")
                    ),
                    Style::default().fg(get_color(app, Color::LightRed)),
                )));
            }
            AiServerStatus::Unknown => {
                lines.push(Line::from(Span::styled(
                    format!(
                        "{}Edit the server field (if needed) then press Tab to scan for Ollama.",
                        symbol_prefix(app, "⏳ ")
                    ),
                    Style::default().fg(get_color(app, Color::Gray)),
                )));
            }
        }

        if !draft.models.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Available models (←/→ or [ ] to browse instantly):",
                Style::default()
                    .fg(get_color(app, Color::Yellow))
                    .add_modifier(Modifier::BOLD),
            )));

            for (idx, model_name) in draft.models.iter().enumerate() {
                let selected = Some(idx) == draft.model_index;
                let indicator = if selected { "▶" } else { " " };
                let style = if selected {
                    Style::default()
                        .fg(get_color(app, Color::Green))
                        .add_modifier(Modifier::BOLD)
                } else if current_field == AiConfigField::Model {
                    Style::default().fg(get_color(app, Color::White))
                } else {
                    Style::default().fg(get_color(app, Color::Gray))
                };

                lines.push(Line::from(vec![Span::styled(
                    format!(" {} {}", indicator, model_name),
                    style,
                )]));
            }

            if current_field == AiConfigField::Model && draft.models.len() > 1 {
                lines.push(Line::from(Span::styled(
                    "Tip: Tap ←/→ to audition the models without leaving the field.",
                    Style::default().fg(get_color(app, Color::Gray)),
                )));
            }
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Enter: save  Esc: cancel  ↑/↓ or Tab: move  Space: toggle enabled  ←/→ or [ ]: cycle models  +/-: adjust refresh",
        Style::default().fg(get_color(app, Color::Gray)),
    )));

    if let Some(err) = &draft.error {
        lines.push(Line::from(Span::styled(
            format!("{}{}", symbol_prefix(app, "⚠️ "), err),
            Style::default().fg(get_color(app, Color::LightRed)),
        )));
    }

    let paragraph = Paragraph::new(Text::from(lines))
        .style(Style::default().fg(get_color(app, Color::White)))
        .wrap(Wrap { trim: false });
    f.render_widget(paragraph, inner);
}

fn render_settings(f: &mut Frame, app: &App) {
    let area = f.area();
    let block = bordered_block(app).title("Settings");
    let inner = block.inner(area);
    f.render_widget(block, area);

    let draft = &app.settings_draft;
    let current_field = draft.current_field();

    let mut lines = Vec::new();

    // Header
    lines.push(Line::from(Span::styled(
        "Configure your Solunatus experience",
        Style::default().fg(get_color(app, Color::Gray)),
    )));
    lines.push(Line::from(""));

    // Location section
    lines.push(Line::from(Span::styled(
        "— Location —",
        Style::default().fg(get_color(app, Color::Yellow)).add_modifier(Modifier::BOLD),
    )));

    let location_mode_str = match draft.location_mode {
        crate::config::LocationMode::City => "City (pick from database)",
        crate::config::LocationMode::Manual => "Manual (lat/lon)",
    };
    let location_hint = match draft.location_mode {
        crate::config::LocationMode::City => "(Space: cycle | Enter: pick city)",
        crate::config::LocationMode::Manual => "(Space: cycle | Enter: input coords)",
    };
    render_setting_field(
        &mut lines,
        app,
        current_field == SettingsField::LocationMode,
        "Location Mode",
        location_mode_str.to_string(),
        Some(location_hint.to_string()),
    );

    // Display selected city or nearest city if location is set
    if draft.location_mode == crate::config::LocationMode::City {
        if let Some(city) = &app.city_name {
            render_setting_field(
                &mut lines,
                app,
                false,
                "Selected City",
                city.clone(),
                None,
            );
        }
    } else {
        // Manual mode - show nearest city if available
        if let Some((nearest_city, distance_km, _bearing)) = &app.nearest_city_info {
            let distance_str = format!("{} ({:.1} km away)", nearest_city, distance_km);
            render_setting_field(
                &mut lines,
                app,
                false,
                "Nearest City",
                distance_str,
                None,
            );
        }
    }

    // Always display current coordinates and timezone
    let coords_str = format!("{:.3}, {:.3}", app.location.lat_degrees(), app.location.lon_degrees());
    render_setting_field(
        &mut lines,
        app,
        false,
        "Coordinates",
        coords_str,
        None,
    );

    render_setting_field(
        &mut lines,
        app,
        false,
        "Timezone",
        app.timezone.to_string(),
        None,
    );

    lines.push(Line::from(""));

    // Time Sync section
    lines.push(Line::from(Span::styled(
        "— Time Sync —",
        Style::default().fg(get_color(app, Color::Yellow)).add_modifier(Modifier::BOLD),
    )));

    let time_sync_enabled_str = if draft.time_sync_enabled { "[x] Enabled" } else { "[ ] Disabled" };
    render_setting_field(
        &mut lines,
        app,
        current_field == SettingsField::TimeSyncEnabled,
        "Time Sync",
        time_sync_enabled_str.to_string(),
        None,
    );

    if draft.time_sync_enabled {
        render_setting_field(
            &mut lines,
            app,
            current_field == SettingsField::TimeSyncServer,
            "NTP Server",
            draft.time_sync_server.clone(),
            None,
        );
    }
    lines.push(Line::from(""));

    // Panel Visibility section
    lines.push(Line::from(Span::styled(
        "— Panel Visibility —",
        Style::default().fg(get_color(app, Color::Yellow)).add_modifier(Modifier::BOLD),
    )));

    render_setting_field(
        &mut lines,
        app,
        current_field == SettingsField::ShowLocationDate,
        "Location & Date",
        if draft.show_location_date { "[x] Show" } else { "[ ] Hide" }.to_string(),
        None,
    );

    render_setting_field(
        &mut lines,
        app,
        current_field == SettingsField::ShowEvents,
        "Events",
        if draft.show_events { "[x] Show" } else { "[ ] Hide" }.to_string(),
        None,
    );

    render_setting_field(
        &mut lines,
        app,
        current_field == SettingsField::ShowPositions,
        "Positions",
        if draft.show_positions { "[x] Show" } else { "[ ] Hide" }.to_string(),
        None,
    );

    render_setting_field(
        &mut lines,
        app,
        current_field == SettingsField::ShowMoon,
        "Moon Details",
        if draft.show_moon { "[x] Show" } else { "[ ] Hide" }.to_string(),
        None,
    );

    render_setting_field(
        &mut lines,
        app,
        current_field == SettingsField::ShowLunarPhases,
        "Lunar Phases",
        if draft.show_lunar_phases { "[x] Show" } else { "[ ] Hide" }.to_string(),
        None,
    );
    lines.push(Line::from(""));

    // Display section
    lines.push(Line::from(Span::styled(
        "— Display —",
        Style::default().fg(get_color(app, Color::Yellow)).add_modifier(Modifier::BOLD),
    )));

    render_setting_field(
        &mut lines,
        app,
        current_field == SettingsField::NightMode,
        "Night Mode",
        if draft.night_mode { "[x] Enabled (red)" } else { "[ ] Disabled" }.to_string(),
        Some("Press Space or Enter to toggle".to_string()),
    );
    lines.push(Line::from(""));

    // AI Configuration section
    lines.push(Line::from(Span::styled(
        "— AI Configuration —",
        Style::default().fg(get_color(app, Color::Yellow)).add_modifier(Modifier::BOLD),
    )));

    let ai_enabled_str = if draft.ai_enabled { "[x] Enabled" } else { "[ ] Disabled" };
    render_setting_field(
        &mut lines,
        app,
        current_field == SettingsField::AiEnabled,
        "AI Insights",
        ai_enabled_str.to_string(),
        None,
    );

    if draft.ai_enabled {
        render_setting_field(
            &mut lines,
            app,
            current_field == SettingsField::AiServer,
            "Ollama Server",
            draft.ai_server.clone(),
            None,
        );

        // Show server status
        match &draft.ai_server_status {
            crate::tui::app::AiServerStatus::Connected { server } => {
                lines.push(Line::from(Span::styled(
                    format!(
                        "  {}Connected to {} — {} model{} available",
                        symbol_prefix(app, "✅ "),
                        server,
                        draft.ai_models.len(),
                        if draft.ai_models.len() == 1 { "" } else { "s" }
                    ),
                    Style::default()
                        .fg(get_color(app, Color::LightGreen))
                        .add_modifier(Modifier::BOLD),
                )));
            }
            crate::tui::app::AiServerStatus::Failed { server, message } => {
                lines.push(Line::from(Span::styled(
                    format!(
                        "  {}Unable to reach {} ({})",
                        symbol_prefix(app, "⚠️ "),
                        server,
                        message.replace('\n', " ")
                    ),
                    Style::default().fg(get_color(app, Color::LightRed)),
                )));
            }
            crate::tui::app::AiServerStatus::Unknown => {}
        }

        let model_hint = if draft.ai_models.is_empty() {
            None
        } else {
            Some("(←/→ or [ ] to browse)".to_string())
        };
        render_setting_field(
            &mut lines,
            app,
            current_field == SettingsField::AiModel,
            "Model",
            draft.ai_model.clone(),
            model_hint,
        );

        // Show available models list if we have them
        if !draft.ai_models.is_empty() {
            for (idx, model_name) in draft.ai_models.iter().enumerate() {
                let selected = Some(idx) == draft.ai_model_index;
                let indicator = if selected { "▶" } else { " " };
                let style = if selected {
                    Style::default()
                        .fg(get_color(app, Color::Green))
                        .add_modifier(Modifier::BOLD)
                } else if current_field == SettingsField::AiModel {
                    Style::default().fg(get_color(app, Color::White))
                } else {
                    Style::default().fg(get_color(app, Color::Gray))
                };

                lines.push(Line::from(vec![
                    Span::raw("   "),
                    Span::styled(
                        format!("{} {}", indicator, model_name),
                        style,
                    )
                ]));
            }

            // Show hint for using arrow keys when on AI Model field
            if current_field == SettingsField::AiModel && draft.ai_models.len() > 1 {
                lines.push(Line::from(Span::styled(
                    "   Tip: Use ←/→ arrows to select a different model",
                    Style::default().fg(get_color(app, Color::Gray)),
                )));
            }
        }

        render_setting_field(
            &mut lines,
            app,
            current_field == SettingsField::AiRefreshMinutes,
            "Refresh (min)",
            draft.ai_refresh_minutes.clone(),
            None,
        );
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Enter: save and apply  Esc: cancel  ↑/↓ or Tab: navigate  Space: toggle  ←/→: select model  d: load defaults",
        Style::default().fg(get_color(app, Color::Gray)),
    )));

    if let Some(err) = &draft.error {
        lines.push(Line::from(Span::styled(
            format!("{}{}", symbol_prefix(app, "⚠️ "), err),
            Style::default().fg(get_color(app, Color::LightRed)),
        )));
    }

    let paragraph = Paragraph::new(Text::from(lines))
        .style(Style::default().fg(get_color(app, Color::White)))
        .wrap(Wrap { trim: false });
    f.render_widget(paragraph, inner);
}

fn render_setting_field(
    lines: &mut Vec<Line>,
    app: &App,
    is_selected: bool,
    label: &str,
    value: String,
    hint: Option<String>,
) {
    let prefix = if is_selected { "› " } else { "  " };
    let mut spans = vec![
        Span::styled(prefix, Style::default().fg(get_color(app, Color::Cyan))),
        Span::styled(
            format!("{:<18}", label),
            Style::default()
                .fg(get_color(app, Color::Yellow))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            value,
            Style::default()
                .fg(get_color(app, Color::White))
                .add_modifier(if is_selected {
                    Modifier::BOLD
                } else {
                    Modifier::empty()
                }),
        ),
    ];

    if let Some(hint_text) = hint {
        spans.push(Span::styled(
            format!("  {}", hint_text),
            Style::default().fg(get_color(app, Color::Gray)),
        ));
    }

    lines.push(Line::from(spans));
}

fn render_reports_menu(f: &mut Frame, app: &App) {
    use super::app::ReportsMenuItem;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Min(10),    // Menu items
            Constraint::Length(2),  // Footer
        ])
        .split(f.area());

    // Title
    let title = Paragraph::new("Reports")
        .style(
            Style::default()
                .fg(get_color(app, Color::Cyan))
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(bordered_block(app));
    f.render_widget(title, chunks[0]);

    // Menu items
    let mut lines = Vec::new();
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Select a report to generate:",
        Style::default().fg(get_color(app, Color::Gray)),
    )));
    lines.push(Line::from(""));

    let calendar_selected = app.reports_selected_item == ReportsMenuItem::Calendar;
    let calendar_style = if calendar_selected {
        Style::default()
            .fg(get_color(app, Color::Yellow))
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(get_color(app, Color::White))
    };
    lines.push(Line::from(vec![
        Span::raw(if calendar_selected { "► " } else { "  " }),
        Span::styled("Astronomical Calendar", calendar_style),
    ]));
    lines.push(Line::from(Span::styled(
        "    Generate HTML or JSON calendar with sun/moon events",
        Style::default().fg(get_color(app, Color::Gray)),
    )));
    lines.push(Line::from(""));

    let validation_selected = app.reports_selected_item == ReportsMenuItem::UsnoValidation;
    let validation_style = if validation_selected {
        Style::default()
            .fg(get_color(app, Color::Yellow))
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(get_color(app, Color::White))
    };
    lines.push(Line::from(vec![
        Span::raw(if validation_selected { "► " } else { "  " }),
        Span::styled("USNO Validation Report", validation_style),
    ]));
    lines.push(Line::from(Span::styled(
        "    Compare astrotimes calculations against U.S. Naval Observatory",
        Style::default().fg(get_color(app, Color::Gray)),
    )));
    lines.push(Line::from(""));

    let benchmark_selected = app.reports_selected_item == ReportsMenuItem::Benchmark;
    let benchmark_style = if benchmark_selected {
        Style::default()
            .fg(get_color(app, Color::Yellow))
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(get_color(app, Color::White))
    };
    lines.push(Line::from(vec![
        Span::raw(if benchmark_selected { "► " } else { "  " }),
        Span::styled("Performance Benchmark", benchmark_style),
    ]));
    lines.push(Line::from(Span::styled(
        "    Test calculation speed across all cities in database",
        Style::default().fg(get_color(app, Color::Gray)),
    )));

    let menu = Paragraph::new(lines)
        .style(Style::default().fg(get_color(app, Color::White)))
        .block(bordered_block(app));
    f.render_widget(menu, chunks[1]);

    // Footer
    let footer = Paragraph::new("↑/↓: Navigate | Enter: Select | Esc: Back")
        .style(Style::default().fg(get_color(app, Color::Gray)))
        .alignment(Alignment::Center);
    f.render_widget(footer, chunks[2]);
}
