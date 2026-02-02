//! Main application state resource.

use bevy::prelude::*;
use dnd_core::world::NarrativeType;
use tokio::sync::mpsc;

use super::{WorkerRequest, WorkerResponse, WorldUpdate};

/// A narrative entry with styling.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct NarrativeEntry {
    pub text: String,
    pub entry_type: NarrativeType,
    pub timestamp: f64,
}

/// Active overlay screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ActiveOverlay {
    #[default]
    None,
    Inventory,
    CharacterSheet,
    QuestLog,
    Help,
    Settings,
    LoadCharacter,
    LoadGame,
    Onboarding,
}

/// Main application state resource.
#[derive(Resource)]
#[allow(dead_code)]
pub struct AppState {
    /// Current world state snapshot.
    pub world: WorldUpdate,
    /// Narrative history.
    pub narrative: Vec<NarrativeEntry>,
    /// Current streaming text (not yet complete).
    pub streaming_text: String,
    /// Player input text.
    pub input_text: String,
    /// Whether we're waiting for AI response.
    pub is_processing: bool,
    /// Status bar message.
    pub status_message: Option<String>,
    /// Current overlay.
    pub overlay: ActiveOverlay,
    /// Request channel sender.
    pub request_tx: Option<mpsc::Sender<WorkerRequest>>,
    /// Response channel receiver.
    pub response_rx: Option<mpsc::Receiver<WorkerResponse>>,
    /// Whether in combat.
    pub in_combat: bool,
    /// Whether it's the player's turn.
    pub is_player_turn: bool,
    /// Error message to display.
    pub error_message: Option<String>,
    /// Time since last effect (for animation timing).
    pub last_effect_time: f64,
    /// Whether the character panel is expanded.
    pub character_panel_expanded: bool,
    /// Whether a save operation is in progress.
    pub is_saving: bool,
    /// Whether a load operation is in progress.
    pub is_loading: bool,
    /// When the status message was set (for auto-clear).
    pub status_set_time: Option<f64>,
    /// History of player commands for up/down navigation.
    pub input_history: Vec<String>,
    /// Current position in input history (-1 means not browsing history).
    pub history_index: i32,
    /// Saved input text when browsing history.
    pub saved_input: String,
    /// Spell currently being viewed in detail (None if not viewing any).
    pub viewing_spell: Option<String>,
    /// Queued UI sounds to play (processed by separate system).
    pub pending_sounds: Vec<crate::sound::SoundEffect>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            world: WorldUpdate::default(),
            narrative: Vec::new(),
            streaming_text: String::new(),
            input_text: String::new(),
            is_processing: false,
            status_message: None,
            overlay: ActiveOverlay::None,
            request_tx: None,
            response_rx: None,
            in_combat: false,
            is_player_turn: false,
            error_message: None,
            last_effect_time: 0.0,
            character_panel_expanded: true,
            is_saving: false,
            is_loading: false,
            status_set_time: None,
            input_history: Vec::new(),
            history_index: -1,
            saved_input: String::new(),
            viewing_spell: None,
            pending_sounds: Vec::new(),
        }
    }
}

impl AppState {
    /// Add a narrative entry.
    pub fn add_narrative(&mut self, text: String, entry_type: NarrativeType, time: f64) {
        self.narrative.push(NarrativeEntry {
            text,
            entry_type,
            timestamp: time,
        });
        // Keep narrative history bounded
        if self.narrative.len() > 500 {
            self.narrative.remove(0);
        }
    }

    /// Set a status message (with timestamp for auto-clear).
    pub fn set_status(&mut self, message: impl Into<String>, current_time: f64) {
        self.status_message = Some(message.into());
        self.status_set_time = Some(current_time);
    }

    /// Set a status message without timestamp (won't auto-clear).
    pub fn set_status_persistent(&mut self, message: impl Into<String>) {
        self.status_message = Some(message.into());
        self.status_set_time = None;
    }

    /// Clear status message.
    pub fn clear_status(&mut self) {
        self.status_message = None;
        self.status_set_time = None;
    }

    /// Toggle an overlay.
    pub fn toggle_overlay(&mut self, overlay: ActiveOverlay) {
        if self.overlay == overlay {
            self.overlay = ActiveOverlay::None;
        } else {
            self.overlay = overlay;
        }
    }

    /// Send a player action to the AI worker.
    pub fn send_action(&mut self, action: String) {
        if let Some(tx) = &self.request_tx {
            if !action.trim().is_empty() && !self.is_processing {
                let _ = tx.try_send(WorkerRequest::PlayerAction(action));
                self.is_processing = true;
                self.streaming_text.clear();
            }
        }
    }

    /// Check if the game session is active.
    pub fn has_session(&self) -> bool {
        self.request_tx.is_some()
    }

    /// Add a command to input history.
    pub fn add_to_history(&mut self, command: String) {
        // Don't add empty or duplicate consecutive commands
        if !command.trim().is_empty() && self.input_history.last() != Some(&command) {
            self.input_history.push(command);
        }
        // Keep history bounded
        if self.input_history.len() > 100 {
            self.input_history.remove(0);
        }
        // Reset history navigation
        self.history_index = -1;
        self.saved_input.clear();
    }

    /// Navigate up in history (older commands).
    pub fn history_up(&mut self) {
        if self.input_history.is_empty() {
            return;
        }

        if self.history_index == -1 {
            // Starting to browse history, save current input
            self.saved_input = self.input_text.clone();
            self.history_index = self.input_history.len() as i32 - 1;
        } else if self.history_index > 0 {
            self.history_index -= 1;
        }

        if let Some(cmd) = self.input_history.get(self.history_index as usize) {
            self.input_text = cmd.clone();
        }
    }

    /// Navigate down in history (newer commands).
    pub fn history_down(&mut self) {
        if self.history_index == -1 {
            return; // Not browsing history
        }

        self.history_index += 1;

        if self.history_index >= self.input_history.len() as i32 {
            // Back to current input
            self.history_index = -1;
            self.input_text = std::mem::take(&mut self.saved_input);
        } else if let Some(cmd) = self.input_history.get(self.history_index as usize) {
            self.input_text = cmd.clone();
        }
    }

    /// Queue a click sound to be played.
    pub fn play_click(&mut self) {
        self.pending_sounds.push(crate::sound::SoundEffect::Click);
    }
}
