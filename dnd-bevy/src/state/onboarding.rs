//! Onboarding state tracking.

use bevy::prelude::*;

/// Tracks whether the user has seen the onboarding modal.
#[derive(Resource, Default)]
pub struct OnboardingState {
    /// Whether the user has completed onboarding.
    pub has_seen: bool,
    /// Current page of the onboarding flow (0-2).
    pub current_page: usize,
}

impl OnboardingState {
    /// Load onboarding state from disk.
    pub fn load() -> Self {
        let path = std::path::Path::new("saves/onboarding.json");
        if path.exists() {
            if let Ok(contents) = std::fs::read_to_string(path) {
                if let Ok(data) = serde_json::from_str::<serde_json::Value>(&contents) {
                    if let Some(has_seen) = data.get("has_seen").and_then(|v| v.as_bool()) {
                        return Self {
                            has_seen,
                            current_page: 0,
                        };
                    }
                }
            }
        }
        Self::default()
    }

    /// Save onboarding state to disk.
    pub fn save(&self) {
        let data = serde_json::json!({
            "has_seen": self.has_seen
        });
        if let Ok(contents) = serde_json::to_string_pretty(&data) {
            let _ = std::fs::write("saves/onboarding.json", contents);
        }
    }

    /// Mark onboarding as complete and save.
    pub fn complete(&mut self) {
        self.has_seen = true;
        self.save();
    }
}
