//! Window settings state management.

use bevy::prelude::*;

/// Resource to control window settings.
#[derive(Resource, Clone)]
pub struct WindowSettings {
    /// Window width in pixels
    pub width: f32,
    /// Window height in pixels
    pub height: f32,
    /// Whether the window is fullscreen
    pub fullscreen: bool,
    /// Track if settings changed (for auto-save)
    changed: bool,
}

impl Default for WindowSettings {
    fn default() -> Self {
        Self {
            width: 1280.0,
            height: 800.0,
            fullscreen: false,
            changed: false,
        }
    }
}

impl WindowSettings {
    /// Create settings with specific values.
    pub fn new(width: f32, height: f32, fullscreen: bool) -> Self {
        Self {
            width: width.max(800.0),
            height: height.max(600.0),
            fullscreen,
            changed: false,
        }
    }

    /// Mark settings as changed (will trigger auto-save).
    pub fn mark_changed(&mut self) {
        self.changed = true;
    }

    /// Check if settings need saving.
    pub fn needs_save(&self) -> bool {
        self.changed
    }

    /// Clear the changed flag after saving.
    pub fn clear_changed(&mut self) {
        self.changed = false;
    }
}
