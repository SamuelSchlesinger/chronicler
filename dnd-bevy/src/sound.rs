//! Sound effects for the game.
//!
//! Uses Bevy's audio system to play sound effects on game events.

use bevy::prelude::*;

/// Sound effect types that can be played.
#[derive(Event, Clone, Copy, Debug)]
pub enum SoundEffect {
    /// Dice rolling sound
    DiceRoll,
    /// Attack hits target
    Hit,
    /// Attack misses
    Miss,
    /// Critical hit
    CriticalHit,
    /// Taking damage
    Damage,
    /// Healing
    Heal,
    /// Spell cast
    SpellCast,
    /// Level up fanfare
    LevelUp,
    /// Combat starts
    CombatStart,
    /// Death/defeat
    Death,
    /// Button click
    Click,
}

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

/// Resource to control sound settings.
#[derive(Resource)]
pub struct SoundSettings {
    /// Master volume (0.0 to 1.0)
    pub volume: f32,
    /// Whether sound is enabled
    pub enabled: bool,
}

impl Default for SoundSettings {
    fn default() -> Self {
        Self {
            volume: 0.7,
            enabled: true,
        }
    }
}

/// Plugin to add sound functionality.
pub struct SoundPlugin;

impl Plugin for SoundPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SoundAssets>()
            .init_resource::<SoundSettings>()
            .add_event::<SoundEffect>()
            .add_systems(Startup, load_sounds)
            .add_systems(Update, play_sounds);
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
fn play_sounds(
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

/// Helper to send a sound event.
pub fn play_sound(writer: &mut EventWriter<SoundEffect>, effect: SoundEffect) {
    writer.send(effect);
}
