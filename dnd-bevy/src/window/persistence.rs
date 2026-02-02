//! Window settings persistence (load/save to disk).

use super::settings::WindowSettings;

/// Get the settings file path for the given saves directory.
fn settings_path(saves_path: &str) -> String {
    format!("{}/window_settings.json", saves_path)
}

/// Load window settings from disk.
/// Note: Fullscreen is always disabled due to macOS keyboard/scaling issues.
pub fn load_settings(saves_path: &str) -> WindowSettings {
    let path = settings_path(saves_path);
    let path = std::path::Path::new(&path);
    if path.exists() {
        if let Ok(contents) = std::fs::read_to_string(path) {
            if let Ok(data) = serde_json::from_str::<serde_json::Value>(&contents) {
                let width = data
                    .get("width")
                    .and_then(|v| v.as_f64())
                    .map(|v| v as f32)
                    .unwrap_or(1280.0);
                let height = data
                    .get("height")
                    .and_then(|v| v.as_f64())
                    .map(|v| v as f32)
                    .unwrap_or(800.0);
                // Force windowed mode - fullscreen causes keyboard/scaling issues on macOS
                return WindowSettings::new(width, height, false);
            }
        }
    }
    WindowSettings::default()
}

/// Save window settings to disk.
pub fn save_settings(settings: &mut WindowSettings, saves_path: &str) {
    let data = serde_json::json!({
        "width": settings.width,
        "height": settings.height,
        "fullscreen": settings.fullscreen
    });
    if let Ok(contents) = serde_json::to_string_pretty(&data) {
        let _ = std::fs::write(settings_path(saves_path), contents);
    }
    settings.clear_changed();
}
