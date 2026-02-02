//! Game phase state machine.

use bevy::prelude::*;

/// Game phase state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, States)]
#[allow(dead_code)]
pub enum GamePhase {
    /// Main menu / title screen
    #[default]
    MainMenu,
    /// Character creation wizard
    CharacterCreation,
    /// Active gameplay
    Playing,
    /// Game over screen
    GameOver,
}
