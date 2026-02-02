//! Sound settings persistence (load/save to disk).

use super::settings::SoundSettings;

/// Get the settings file path for the given saves directory.
fn settings_path(saves_path: &str) -> String {
    format!("{}/audio_settings.json", saves_path)
}

/// Load sound settings from disk.
pub fn load_settings(saves_path: &str) -> SoundSettings {
    let path = settings_path(saves_path);
    let path = std::path::Path::new(&path);
    if path.exists() {
        if let Ok(contents) = std::fs::read_to_string(path) {
            if let Ok(data) = serde_json::from_str::<serde_json::Value>(&contents) {
                let volume = data
                    .get("volume")
                    .and_then(|v| v.as_f64())
                    .map(|v| v as f32)
                    .unwrap_or(0.7);
                let enabled = data
                    .get("enabled")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true);
                return SoundSettings::new(volume, enabled);
            }
        }
    }
    SoundSettings::default()
}

/// Save sound settings to disk.
pub fn save_settings(settings: &mut SoundSettings, saves_path: &str) {
    let data = serde_json::json!({
        "volume": settings.volume,
        "enabled": settings.enabled
    });
    if let Ok(contents) = serde_json::to_string_pretty(&data) {
        let _ = std::fs::write(settings_path(saves_path), contents);
    }
    settings.clear_changed();
}
