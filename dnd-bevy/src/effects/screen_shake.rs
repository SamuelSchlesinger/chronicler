//! Determines screen shake intensity for game effects.
//!
//! This module maps game effects to screen shake animations.

use dnd_core::rules::Effect;

/// Returns the screen shake intensity for an effect, if any.
///
/// Returns `None` for effects that should not trigger screen shake,
/// or `Some(intensity)` where intensity is typically between 0.0 and 1.0.
pub fn screen_shake_for_effect(effect: &Effect) -> Option<f32> {
    match effect {
        Effect::AttackHit { is_critical, .. } => {
            if *is_critical {
                Some(1.0)
            } else {
                Some(0.5)
            }
        }

        Effect::CombatStarted => Some(0.3),

        Effect::ConsequenceTriggered { .. } => Some(0.4),

        Effect::CharacterDied { .. } => Some(1.0),

        // All other effects do not trigger screen shake
        _ => None,
    }
}
