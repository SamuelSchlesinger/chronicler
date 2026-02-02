//! Tool call parsing - converts JSON tool inputs into game Intents.
//!
//! This module is organized by domain:
//! - `checks`: dice rolls, skill checks, ability checks, saving throws
//! - `combat`: damage, healing, conditions, combat flow
//! - `inventory`: items, equipment, currency
//! - `class_features`: class-specific abilities
//! - `world`: rests, locations, facts, spells, experience
//! - `quests`: quest creation, objectives, completion
//! - `npc`: NPC creation, updates, movement, removal
//! - `locations`: location creation, connections, updates
//! - `gameplay`: ability score modifications, time advancement, spell slot restoration
//! - `state`: declarative state assertions (disposition, location, status, etc.)
//! - `knowledge`: knowledge tracking and information asymmetry
//! - `schedule`: scheduled events and time-based triggers

mod checks;
mod class_features;
mod combat;
mod gameplay;
mod inventory;
mod knowledge;
mod locations;
mod npc;
mod quests;
mod schedule;
mod state;
mod world;

pub use checks::parse_checks_tool;
pub use class_features::parse_class_features_tool;
pub use combat::parse_combat_tool;
pub use gameplay::parse_gameplay_tool;
pub use inventory::parse_inventory_tool;
pub use knowledge::parse_knowledge_tool;
pub use locations::parse_locations_tool;
pub use npc::parse_npc_tool;
pub use quests::parse_quests_tool;
pub use schedule::parse_schedule_tool;
pub use state::parse_state_tool;
pub use world::parse_world_tool;

use crate::rules::Intent;
use crate::world::GameWorld;
use serde_json::Value;

/// Parse a tool call into an Intent.
///
/// Tries each domain parser in turn until one returns a result.
pub fn parse_tool_call(name: &str, input: &Value, world: &GameWorld) -> Option<Intent> {
    // Try each domain parser in turn
    parse_checks_tool(name, input, world)
        .or_else(|| parse_combat_tool(name, input, world))
        .or_else(|| parse_inventory_tool(name, input))
        .or_else(|| parse_class_features_tool(name, input, world))
        .or_else(|| parse_world_tool(name, input, world))
        .or_else(|| parse_quests_tool(name, input))
        .or_else(|| parse_npc_tool(name, input))
        .or_else(|| parse_locations_tool(name, input))
        .or_else(|| parse_gameplay_tool(name, input, world))
        .or_else(|| parse_state_tool(name, input))
        .or_else(|| parse_knowledge_tool(name, input))
        .or_else(|| parse_schedule_tool(name, input))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::{DamageType, Intent};
    use crate::world::{Character, CharacterClass, ClassLevel, Condition, GameWorld, Skill};
    use serde_json::json;

    fn create_test_world() -> GameWorld {
        let mut character = Character::new("Test Hero");
        character.classes.push(ClassLevel {
            class: CharacterClass::Fighter,
            level: 1,
            subclass: None,
        });
        GameWorld::new("Test Campaign", character)
    }

    #[test]
    fn test_parse_tool_call_roll_dice() {
        let world = create_test_world();
        let input = json!({
            "notation": "2d6+3",
            "purpose": "attack damage"
        });

        let intent = parse_tool_call("roll_dice", &input, &world);
        assert!(intent.is_some());

        if let Some(Intent::RollDice { notation, purpose }) = intent {
            assert_eq!(notation, "2d6+3");
            assert_eq!(purpose, "attack damage");
        } else {
            panic!("Expected RollDice intent");
        }
    }

    #[test]
    fn test_parse_tool_call_skill_check() {
        let world = create_test_world();
        let input = json!({
            "skill": "athletics",
            "dc": 15,
            "description": "climbing the wall"
        });

        let intent = parse_tool_call("skill_check", &input, &world);
        assert!(intent.is_some());

        if let Some(Intent::SkillCheck {
            skill,
            dc,
            description,
            ..
        }) = intent
        {
            assert_eq!(skill, Skill::Athletics);
            assert_eq!(dc, 15);
            assert_eq!(description, "climbing the wall");
        } else {
            panic!("Expected SkillCheck intent");
        }
    }

    #[test]
    fn test_parse_tool_call_apply_damage() {
        let world = create_test_world();
        let input = json!({
            "amount": 10,
            "damage_type": "slashing",
            "source": "sword"
        });

        let intent = parse_tool_call("apply_damage", &input, &world);
        assert!(intent.is_some());

        if let Some(Intent::Damage {
            amount,
            damage_type,
            source,
            ..
        }) = intent
        {
            assert_eq!(amount, 10);
            assert_eq!(damage_type, DamageType::Slashing);
            assert_eq!(source, "sword");
        } else {
            panic!("Expected Damage intent");
        }
    }

    #[test]
    fn test_parse_tool_call_invalid_damage_amount() {
        let world = create_test_world();
        let input = json!({
            "amount": 0,
            "damage_type": "slashing",
            "source": "sword"
        });

        let intent = parse_tool_call("apply_damage", &input, &world);
        assert!(intent.is_none(), "Should reject zero damage");

        let input = json!({
            "amount": -5,
            "damage_type": "slashing",
            "source": "sword"
        });

        let intent = parse_tool_call("apply_damage", &input, &world);
        assert!(intent.is_none(), "Should reject negative damage");
    }

    #[test]
    fn test_parse_tool_call_apply_healing() {
        let world = create_test_world();
        let input = json!({
            "amount": 8,
            "source": "potion"
        });

        let intent = parse_tool_call("apply_healing", &input, &world);
        assert!(intent.is_some());

        if let Some(Intent::Heal { amount, source, .. }) = intent {
            assert_eq!(amount, 8);
            assert_eq!(source, "potion");
        } else {
            panic!("Expected Heal intent");
        }
    }

    #[test]
    fn test_parse_tool_call_invalid_healing_amount() {
        let world = create_test_world();
        let input = json!({
            "amount": 0,
            "source": "potion"
        });

        let intent = parse_tool_call("apply_healing", &input, &world);
        assert!(intent.is_none(), "Should reject zero healing");
    }

    #[test]
    fn test_parse_tool_call_apply_condition() {
        let world = create_test_world();
        let input = json!({
            "condition": "poisoned",
            "source": "trap",
            "duration_rounds": 3
        });

        let intent = parse_tool_call("apply_condition", &input, &world);
        assert!(intent.is_some());

        if let Some(Intent::ApplyCondition {
            condition, source, ..
        }) = intent
        {
            assert_eq!(condition, Condition::Poisoned);
            assert_eq!(source, "trap");
        } else {
            panic!("Expected ApplyCondition intent");
        }
    }

    #[test]
    fn test_parse_tool_call_unknown_tool() {
        let world = create_test_world();
        let input = json!({});

        let intent = parse_tool_call("unknown_tool", &input, &world);
        assert!(intent.is_none());
    }
}
