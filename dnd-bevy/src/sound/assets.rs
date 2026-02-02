//! Sound asset loading and management.

use bevy::prelude::*;

/// Resource holding loaded sound assets.
#[derive(Resource, Default)]
pub struct SoundAssets {
    pub dice_roll: Option<Handle<AudioSource>>,
    pub hit: Option<Handle<AudioSource>>,
    pub miss: Option<Handle<AudioSource>>,
    pub critical_hit: Option<Handle<AudioSource>>,
    pub damage: Option<Handle<AudioSource>>,
    pub heal: Option<Handle<AudioSource>>,
    pub spell_cast: Option<Handle<AudioSource>>,
    pub level_up: Option<Handle<AudioSource>>,
    pub combat_start: Option<Handle<AudioSource>>,
    pub death: Option<Handle<AudioSource>>,
    pub click: Option<Handle<AudioSource>>,
}
