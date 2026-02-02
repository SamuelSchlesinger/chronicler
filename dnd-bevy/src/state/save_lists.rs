//! Save and load list resources.

use bevy::prelude::*;
use dnd_core::GameSession;

/// List of saved characters for the load character overlay.
#[derive(Resource, Default)]
pub struct CharacterSaveList {
    /// The list of character saves.
    pub saves: Vec<dnd_core::CharacterSaveInfo>,
    /// Whether the list is being loaded.
    pub loading: bool,
    /// Whether the list has been loaded at least once.
    pub loaded: bool,
    /// Selected character index.
    pub selected: Option<usize>,
    /// Error message if loading failed.
    pub error: Option<String>,
}

/// Information about a game save file.
#[derive(Debug, Clone)]
pub struct GameSaveInfo {
    pub path: String,
    pub campaign_name: String,
    pub character_name: String,
    pub character_level: u8,
    pub saved_at: String,
}

/// List of saved games for the load game overlay.
#[derive(Resource, Default)]
pub struct GameSaveList {
    pub saves: Vec<GameSaveInfo>,
    pub loading: bool,
    pub loaded: bool,
    pub selected: Option<usize>,
    pub error: Option<String>,
}

/// Pending character list load - holds the receiver for async character list loading.
#[derive(Resource)]
pub struct PendingCharacterList {
    pub receiver: std::sync::Mutex<
        std::sync::mpsc::Receiver<
            Result<Vec<dnd_core::CharacterSaveInfo>, dnd_core::persist::PersistError>,
        >,
    >,
}

/// Pending game list load.
#[derive(Resource)]
pub struct PendingGameList {
    pub receiver: std::sync::Mutex<std::sync::mpsc::Receiver<Result<Vec<GameSaveInfo>, String>>>,
}

/// Pending game session load from a save file.
#[derive(Resource)]
pub struct PendingGameLoad {
    pub receiver: std::sync::Mutex<std::sync::mpsc::Receiver<Result<GameSession, String>>>,
}

/// Pending session creation - holds the receiver for async session creation.
#[derive(Resource)]
pub struct PendingSession {
    pub receiver: std::sync::Mutex<std::sync::mpsc::Receiver<Result<GameSession, String>>>,
}
