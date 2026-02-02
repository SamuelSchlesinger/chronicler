//! Sound effect types.

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
