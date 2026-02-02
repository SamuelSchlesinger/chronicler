//! Sound settings state management.

use bevy::prelude::*;

/// Resource to control sound settings.
#[derive(Resource)]
pub struct SoundSettings {
    /// Master volume (0.0 to 1.0)
    pub volume: f32,
    /// Whether sound is enabled
    pub enabled: bool,
    /// Track if settings changed (for auto-save)
    changed: bool,
}

impl Default for SoundSettings {
    fn default() -> Self {
        Self {
            volume: 0.7,
            enabled: true,
            changed: false,
        }
    }
}

impl SoundSettings {
    /// Create settings with specific volume and enabled state.
    pub fn new(volume: f32, enabled: bool) -> Self {
        Self {
            volume: volume.clamp(0.0, 1.0),
            enabled,
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
