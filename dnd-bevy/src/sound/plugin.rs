//! Sound plugin and systems.

use bevy::prelude::*;

use super::assets::SoundAssets;
use super::effect::SoundEffect;
use super::persistence::save_settings;
use super::settings::SoundSettings;
use crate::AppConfig;

/// Plugin to add sound functionality.
pub struct SoundPlugin;

impl Plugin for SoundPlugin {
    fn build(&self, app: &mut App) {
        // Note: SoundSettings is inserted by main() after loading from disk
        app.init_resource::<SoundAssets>()
            .add_event::<SoundEffect>()
            .add_systems(Startup, load_sounds)
            .add_systems(Update, (play_sounds, auto_save_settings));
    }
}

/// Auto-save sound settings when changed.
fn auto_save_settings(mut settings: ResMut<SoundSettings>, config: Res<AppConfig>) {
    if settings.needs_save() {
        save_settings(&mut settings, &config.saves_path);
    }
}

/// Load sound assets from the assets/sounds directory.
fn load_sounds(asset_server: Res<AssetServer>, mut sounds: ResMut<SoundAssets>) {
    // Try to load each sound file. If the file doesn't exist, the handle will be invalid
    // but won't cause a crash - Bevy just won't play anything.
    sounds.dice_roll = try_load_sound(&asset_server, "sounds/dice_roll.ogg");
    sounds.hit = try_load_sound(&asset_server, "sounds/hit.ogg");
    sounds.miss = try_load_sound(&asset_server, "sounds/miss.ogg");
    sounds.critical_hit = try_load_sound(&asset_server, "sounds/critical_hit.ogg");
    sounds.damage = try_load_sound(&asset_server, "sounds/damage.ogg");
    sounds.heal = try_load_sound(&asset_server, "sounds/heal.ogg");
    sounds.spell_cast = try_load_sound(&asset_server, "sounds/spell_cast.ogg");
    sounds.level_up = try_load_sound(&asset_server, "sounds/level_up.ogg");
    sounds.combat_start = try_load_sound(&asset_server, "sounds/combat_start.ogg");
    sounds.death = try_load_sound(&asset_server, "sounds/death.ogg");
    sounds.click = try_load_sound(&asset_server, "sounds/click.ogg");
}

/// Try to load a sound file. Returns None if the path doesn't exist.
fn try_load_sound(asset_server: &AssetServer, path: &str) -> Option<Handle<AudioSource>> {
    // Always try to load - Bevy will handle missing files gracefully
    Some(asset_server.load(path))
}

/// System to play sounds when events are received.
pub fn play_sounds(
    mut commands: Commands,
    mut events: EventReader<SoundEffect>,
    sounds: Res<SoundAssets>,
    settings: Res<SoundSettings>,
) {
    if !settings.enabled {
        events.clear();
        return;
    }

    for event in events.read() {
        let handle = match event {
            SoundEffect::DiceRoll => sounds.dice_roll.clone(),
            SoundEffect::Hit => sounds.hit.clone(),
            SoundEffect::Miss => sounds.miss.clone(),
            SoundEffect::CriticalHit => sounds.critical_hit.clone(),
            SoundEffect::Damage => sounds.damage.clone(),
            SoundEffect::Heal => sounds.heal.clone(),
            SoundEffect::SpellCast => sounds.spell_cast.clone(),
            SoundEffect::LevelUp => sounds.level_up.clone(),
            SoundEffect::CombatStart => sounds.combat_start.clone(),
            SoundEffect::Death => sounds.death.clone(),
            SoundEffect::Click => sounds.click.clone(),
        };

        if let Some(source) = handle {
            commands.spawn((
                AudioPlayer::new(source),
                PlaybackSettings {
                    mode: bevy::audio::PlaybackMode::Despawn,
                    volume: bevy::audio::Volume::new(settings.volume),
                    ..default()
                },
            ));
        }
    }
}
