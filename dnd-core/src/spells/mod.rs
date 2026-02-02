//! Spell database and spellcasting mechanics.
//!
//! Contains SRD 5.2 spell definitions and lookup functions.

mod database;
mod types;

// Re-export all public types
pub use types::{
    AreaOfEffect, CastingTime, Components, DamageScaling, SpellAttackType, SpellClass, SpellData,
    SpellDuration, SpellRange, SpellSchool,
};

// Re-export database functions
pub use database::{all_spells, get_spell, spells_by_level, spells_for_class};
