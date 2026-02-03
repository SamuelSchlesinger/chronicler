//! Helper functions for the rules engine.

use crate::dice::{self, ComponentResult, DiceExpression, DieType, RollResult};

/// Roll dice with a fallback expression. If both fail, returns a minimal result.
///
/// This avoids nested unwraps which could panic in edge cases.
pub fn roll_with_fallback(notation: &str, fallback: &str) -> RollResult {
    dice::roll(notation)
        .or_else(|_| dice::roll(fallback))
        .unwrap_or_else(|_| {
            // Create a minimal fallback result (1d4 = 1)
            let expr = DiceExpression {
                components: vec![],
                modifier: 1,
                original: fallback.to_string(),
            };
            RollResult {
                expression: expr,
                component_results: vec![ComponentResult {
                    die_type: DieType::D4,
                    rolls: vec![1],
                    kept: vec![1],
                    subtotal: 1,
                }],
                modifier: 0,
                total: 1,
                natural_20: false,
                natural_1: false,
            }
        })
}

/// Calculate the number of d6s for Sneak Attack based on Rogue level.
/// Sneak Attack scales: 1d6 at level 1, +1d6 every odd level.
pub fn sneak_attack_dice(rogue_level: u8) -> u8 {
    // 1d6 at 1, 2d6 at 3, 3d6 at 5, etc.
    rogue_level.div_ceil(2)
}
