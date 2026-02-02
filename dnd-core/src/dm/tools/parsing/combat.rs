//! Parsing for combat-related tools.

use super::super::converters::{parse_condition, parse_damage_type};
use crate::rules::{CombatantInit, Intent};
use crate::world::{CharacterId, GameWorld};
use serde_json::Value;

/// Parse combat-related tool calls.
pub fn parse_combat_tool(name: &str, input: &Value, world: &GameWorld) -> Option<Intent> {
    match name {
        "apply_damage" => {
            let amount = input["amount"].as_i64()? as i32;
            // Validate damage is positive
            if amount <= 0 {
                return None;
            }
            let damage_type = parse_damage_type(input["damage_type"].as_str()?)?;
            let source = input["source"].as_str().unwrap_or("unknown").to_string();
            Some(Intent::Damage {
                target_id: world.player_character.id,
                amount,
                damage_type,
                source,
            })
        }
        "apply_healing" => {
            let amount = input["amount"].as_i64()? as i32;
            // Validate healing is positive
            if amount <= 0 {
                return None;
            }
            let source = input["source"].as_str().unwrap_or("healing").to_string();
            Some(Intent::Heal {
                target_id: world.player_character.id,
                amount,
                source,
            })
        }
        "apply_condition" => {
            let condition = parse_condition(input["condition"].as_str()?)?;
            let source = input["source"].as_str().unwrap_or("unknown").to_string();
            let duration_rounds = input["duration_rounds"].as_i64().map(|d| d as u32);
            Some(Intent::ApplyCondition {
                target_id: world.player_character.id,
                condition,
                source,
                duration_rounds,
            })
        }
        "remove_condition" => {
            let condition = parse_condition(input["condition"].as_str()?)?;
            Some(Intent::RemoveCondition {
                target_id: world.player_character.id,
                condition,
            })
        }
        "start_combat" => {
            let enemies = input["enemies"].as_array()?;
            let player_hp = &world.player_character.hit_points;
            let mut combatants = vec![CombatantInit {
                id: world.player_character.id,
                name: world.player_character.name.clone(),
                is_player: true,
                is_ally: true,
                current_hp: player_hp.current,
                max_hp: player_hp.maximum,
                armor_class: world.player_character.current_ac(),
                initiative_modifier: world.player_character.initiative_modifier(),
            }];

            for enemy in enemies {
                let name = enemy["name"].as_str().unwrap_or("Enemy").to_string();
                let max_hp = enemy["max_hp"].as_i64().unwrap_or(10) as i32;
                let current_hp = enemy["current_hp"].as_i64().unwrap_or(max_hp as i64) as i32;
                let armor_class = enemy["armor_class"].as_u64().unwrap_or(10) as u8;
                let initiative_modifier = enemy["initiative_modifier"].as_i64().unwrap_or(0) as i8;
                combatants.push(CombatantInit {
                    id: CharacterId::new(),
                    name,
                    is_player: false,
                    is_ally: false,
                    current_hp,
                    max_hp,
                    armor_class,
                    initiative_modifier,
                });
            }

            Some(Intent::StartCombat { combatants })
        }
        "end_combat" => Some(Intent::EndCombat),
        "next_turn" => Some(Intent::NextTurn),
        "death_save" => Some(Intent::DeathSave {
            character_id: world.player_character.id,
        }),
        "concentration_check" => {
            let damage_taken = input["damage_taken"].as_i64()? as i32;
            let spell_name = input["spell_name"].as_str()?.to_string();
            Some(Intent::ConcentrationCheck {
                character_id: world.player_character.id,
                damage_taken,
                spell_name,
            })
        }
        _ => None,
    }
}
