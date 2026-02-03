//! Resolution methods for different intent domains.
//!
//! Each submodule contains `impl RulesEngine` blocks with resolution methods
//! for a specific domain (combat, checks, spells, etc.).

mod checks;
mod class_features;
mod combat;
mod inventory;
mod misc;
mod quests;
mod spells;
mod time;
mod world;
