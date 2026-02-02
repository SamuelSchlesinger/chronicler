//! Maps game effects to sound effects.
//!
//! This module determines which sound to play for each game effect.

use dnd_core::rules::Effect;

use crate::sound::SoundEffect;

/// Determines which sound effect (if any) should play for a given game effect.
pub fn sound_for_effect(effect: &Effect) -> Option<SoundEffect> {
    match effect {
        Effect::DiceRolled { .. } => Some(SoundEffect::DiceRoll),

        Effect::AttackHit { is_critical, .. } => {
            if *is_critical {
                Some(SoundEffect::CriticalHit)
            } else {
                Some(SoundEffect::Hit)
            }
        }

        Effect::AttackMissed { .. } => Some(SoundEffect::Miss),

        Effect::HpChanged { amount, .. } => {
            if *amount > 0 {
                Some(SoundEffect::Heal)
            } else {
                Some(SoundEffect::Damage)
            }
        }

        Effect::CombatStarted => Some(SoundEffect::CombatStart),

        Effect::LevelUp { .. } => Some(SoundEffect::LevelUp),

        Effect::SpellSlotUsed { .. } => Some(SoundEffect::SpellCast),

        Effect::CharacterDied { .. } => Some(SoundEffect::Death),

        // Effects with no associated sound
        Effect::ConditionApplied { .. }
        | Effect::ConditionRemoved { .. }
        | Effect::CombatEnded
        | Effect::TurnAdvanced { .. }
        | Effect::InitiativeRolled { .. }
        | Effect::CombatantAdded { .. }
        | Effect::TimeAdvanced { .. }
        | Effect::ExperienceGained { .. }
        | Effect::FeatureUsed { .. }
        | Effect::RestCompleted { .. }
        | Effect::CheckSucceeded { .. }
        | Effect::CheckFailed { .. }
        | Effect::FactRemembered { .. }
        | Effect::ConsequenceRegistered { .. }
        | Effect::ConsequenceTriggered { .. }
        | Effect::ItemAdded { .. }
        | Effect::ItemRemoved { .. }
        | Effect::ItemEquipped { .. }
        | Effect::ItemUnequipped { .. }
        | Effect::ItemUsed { .. }
        | Effect::GoldChanged { .. }
        | Effect::SilverChanged { .. }
        | Effect::AcChanged { .. }
        | Effect::DeathSaveFailure { .. }
        | Effect::DeathSavesReset { .. }
        | Effect::DeathSaveSuccess { .. }
        | Effect::Stabilized { .. }
        | Effect::ConcentrationBroken { .. }
        | Effect::ConcentrationMaintained { .. }
        | Effect::LocationChanged { .. }
        | Effect::ClassResourceUsed { .. }
        | Effect::RageStarted { .. }
        | Effect::RageEnded { .. } => None,
    }
}
