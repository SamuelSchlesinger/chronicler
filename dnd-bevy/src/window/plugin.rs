//! Window settings plugin and systems.

use bevy::prelude::*;
use bevy::window::{MonitorSelection, WindowMode};

use super::persistence::save_settings;
use super::settings::WindowSettings;
use crate::AppConfig;

/// Plugin to manage window settings.
pub struct WindowSettingsPlugin;

impl Plugin for WindowSettingsPlugin {
    fn build(&self, app: &mut App) {
        // WindowSettings is inserted by main() after loading from disk
        app.add_systems(Startup, apply_initial_settings)
            .add_systems(Update, (apply_window_settings, auto_save_settings));
    }
}

/// Apply initial window settings on startup.
fn apply_initial_settings(settings: Res<WindowSettings>, mut windows: Query<&mut Window>) {
    for mut window in windows.iter_mut() {
        apply_settings_to_window(&mut window, &settings);
    }
}

/// Apply window settings when they change.
fn apply_window_settings(settings: Res<WindowSettings>, mut windows: Query<&mut Window>) {
    if !settings.is_changed() {
        return;
    }

    for mut window in windows.iter_mut() {
        apply_settings_to_window(&mut window, &settings);
    }
}

/// Helper to apply settings to a window.
fn apply_settings_to_window(window: &mut Window, settings: &WindowSettings) {
    if settings.fullscreen {
        window.mode = WindowMode::BorderlessFullscreen(MonitorSelection::Current);
    } else {
        window.mode = WindowMode::Windowed;
        window.resolution = (settings.width, settings.height).into();
    }
}

/// Auto-save window settings when changed.
fn auto_save_settings(mut settings: ResMut<WindowSettings>, config: Res<AppConfig>) {
    if settings.needs_save() {
        save_settings(&mut settings, &config.saves_path);
    }
}
