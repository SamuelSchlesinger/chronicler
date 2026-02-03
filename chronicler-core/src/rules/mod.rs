//! D&D 5e Rules Engine with Intent/Effect system.
//!
//! This module implements the core game mechanic pipeline:
//! 1. AI suggests an Intent (what the player/NPC wants to do)
//! 2. RulesEngine resolves the Intent using D&D 5e rules
//! 3. Effects are produced that describe state changes
//! 4. Effects are applied to the GameWorld
//!
//! This separation ensures deterministic, testable game mechanics
//! independent of AI decision-making.

mod effects;
mod engine;
mod helpers;
mod resolve;
#[cfg(test)]
mod tests;
mod types;

// Re-export public API
pub use effects::{apply_effect, apply_effects};
pub use engine::RulesEngine;
pub use types::{CombatantInit, DamageType, Effect, Intent, Resolution, RestType, StateType};
