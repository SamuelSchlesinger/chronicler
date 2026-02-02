//! Parsing for dice rolls and check-related tools.

use super::super::converters::{parse_ability, parse_advantage, parse_skill};
use crate::rules::Intent;
use crate::world::GameWorld;
use serde_json::Value;

/// Parse check-related tool calls: roll_dice, skill_check, ability_check, saving_throw.
pub fn parse_checks_tool(name: &str, input: &Value, world: &GameWorld) -> Option<Intent> {
    match name {
        "roll_dice" => {
            let notation = input["notation"].as_str()?;
            let purpose = input["purpose"].as_str().unwrap_or("general roll");
            Some(Intent::RollDice {
                notation: notation.to_string(),
                purpose: purpose.to_string(),
            })
        }
        "skill_check" => {
            let skill = parse_skill(input["skill"].as_str()?)?;
            let dc = input["dc"].as_i64()? as i32;
            let description = input["description"].as_str().unwrap_or("").to_string();
            let advantage = parse_advantage(input["advantage"].as_str());
            Some(Intent::SkillCheck {
                character_id: world.player_character.id,
                skill,
                dc,
                advantage,
                description,
            })
        }
        "ability_check" => {
            let ability = parse_ability(input["ability"].as_str()?)?;
            let dc = input["dc"].as_i64()? as i32;
            let description = input["description"].as_str().unwrap_or("").to_string();
            let advantage = parse_advantage(input["advantage"].as_str());
            Some(Intent::AbilityCheck {
                character_id: world.player_character.id,
                ability,
                dc,
                advantage,
                description,
            })
        }
        "saving_throw" => {
            let ability = parse_ability(input["ability"].as_str()?)?;
            let dc = input["dc"].as_i64()? as i32;
            let source = input["source"].as_str().unwrap_or("unknown").to_string();
            let advantage = parse_advantage(input["advantage"].as_str());
            Some(Intent::SavingThrow {
                character_id: world.player_character.id,
                ability,
                dc,
                advantage,
                source,
            })
        }
        _ => None,
    }
}
