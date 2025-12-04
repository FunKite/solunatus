// Terminal User Interface module

pub mod app;
pub mod cache;
pub mod drafts;
pub mod events;
pub mod ui;

pub use app::{App, AppConfig, AppMode, ReportsMenuItem};
pub use cache::{CachedEvents, CachedMoonDetails, CachedPositions, MoonAltitudeTrend};
pub use drafts::{CalendarDraft, CalendarField, LocationInputDraft, LocationInputField, SettingsDraft, SettingsField};
pub use events::handle_events;
pub use ui::render;

#[cfg(feature = "ai-insights")]
pub use drafts::{AiConfigDraft, AiConfigField, AiServerStatus};
