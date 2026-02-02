//! Main effect processor that coordinates all effect handling.
//!
//! This module ties together sound effects, narrative output, and screen shake
//! to provide a unified interface for processing game effects.

use bevy::prelude::*;
use dnd_core::rules::Effect;

use crate::animations;
use crate::sound::SoundEffect;
use crate::state::AppState;

use super::narrative::narrative_for_effect;
use super::screen_shake::screen_shake_for_effect;
use super::sound_mapping::sound_for_effect;

/// Process a game effect and trigger appropriate UI updates, sounds, and effects.
pub fn process_effect(
    app_state: &mut AppState,
    effect: &Effect,
    commands: &mut Commands,
    time: f64,
    sound_writer: &mut EventWriter<SoundEffect>,
) {
    // Play sound effect if applicable
    if let Some(sound) = sound_for_effect(effect) {
        sound_writer.send(sound);
    }

    // Trigger screen shake if applicable
    if let Some(intensity) = screen_shake_for_effect(effect) {
        animations::spawn_screen_shake(commands, intensity);
    }

    // Generate and display narrative
    if let Some(narrative) = narrative_for_effect(effect) {
        app_state.add_narrative(narrative.text, narrative.narrative_type, time);

        if let Some(status) = narrative.status {
            app_state.set_status(status, time);
        }
    }
}
