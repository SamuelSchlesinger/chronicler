//! Window settings management.
//!
//! Handles window size, fullscreen mode, and persistence.

mod persistence;
mod plugin;
mod settings;

pub use persistence::load_settings;
pub use plugin::WindowSettingsPlugin;
pub use settings::WindowSettings;
