//! Sound effects for the game.
//!
//! Uses Bevy's audio system to play sound effects on game events.

mod assets;
mod effect;
mod persistence;
mod plugin;
mod settings;

pub use effect::SoundEffect;
pub use persistence::load_settings;
pub use plugin::SoundPlugin;
pub use settings::SoundSettings;
