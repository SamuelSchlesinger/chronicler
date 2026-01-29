//! D&D 5e game mechanics and state management
//!
//! This module contains all the core D&D rules implementations:
//! - Dice rolling with standard notation
//! - Character sheets with full 5e stats
//! - Combat tracking with initiative
//! - World state management

pub mod character;
pub mod combat;
pub mod conditions;
pub mod dice;
pub mod skills;
pub mod state;

pub use character::{Ability, AbilityScores, Character, HitPoints};
pub use combat::{CombatState, CombatantStatus, InitiativeEntry};
pub use conditions::Condition;
pub use dice::{Advantage, DiceExpression, DiceRoll, RollResult};
pub use skills::Skill;
pub use state::{GameMode, GameWorld, Location};
