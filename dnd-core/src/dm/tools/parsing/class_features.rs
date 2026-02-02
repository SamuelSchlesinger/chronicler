//! Parsing for class feature tools.

use crate::rules::Intent;
use crate::world::GameWorld;
use serde_json::Value;

/// Parse class feature tool calls.
pub fn parse_class_features_tool(name: &str, input: &Value, world: &GameWorld) -> Option<Intent> {
    match name {
        "use_rage" => Some(Intent::UseRage {
            character_id: world.player_character.id,
        }),
        "end_rage" => {
            let reason = input["reason"].as_str().unwrap_or("voluntary").to_string();
            Some(Intent::EndRage {
                character_id: world.player_character.id,
                reason,
            })
        }
        "use_ki" => {
            let points = input["points"].as_u64()? as u8;
            let ability = input["ability"].as_str()?.to_string();
            Some(Intent::UseKi {
                character_id: world.player_character.id,
                points,
                ability,
            })
        }
        "use_lay_on_hands" => {
            let target_name = input["target"].as_str()?.to_string();
            let hp_amount = input["hp_amount"].as_u64().unwrap_or(0) as u32;
            let cure_disease = input["cure_disease"].as_bool().unwrap_or(false);
            let neutralize_poison = input["neutralize_poison"].as_bool().unwrap_or(false);
            Some(Intent::UseLayOnHands {
                character_id: world.player_character.id,
                target_name,
                hp_amount,
                cure_disease,
                neutralize_poison,
            })
        }
        "use_divine_smite" => {
            let spell_slot_level = input["spell_slot_level"].as_u64()? as u8;
            let target_is_undead_or_fiend = input["target_is_undead_or_fiend"]
                .as_bool()
                .unwrap_or(false);
            Some(Intent::UseDivineSmite {
                character_id: world.player_character.id,
                spell_slot_level,
                target_is_undead_or_fiend,
            })
        }
        "use_wild_shape" => {
            let beast_form = input["beast_form"].as_str()?.to_string();
            let beast_hp = input["beast_hp"].as_i64()? as i32;
            let beast_ac = input["beast_ac"].as_u64().map(|ac| ac as u8);
            Some(Intent::UseWildShape {
                character_id: world.player_character.id,
                beast_form,
                beast_hp,
                beast_ac,
            })
        }
        "end_wild_shape" => {
            let reason = input["reason"].as_str()?.to_string();
            let excess_damage = input["excess_damage"].as_i64().unwrap_or(0) as i32;
            Some(Intent::EndWildShape {
                character_id: world.player_character.id,
                reason,
                excess_damage,
            })
        }
        "use_channel_divinity" => {
            let option = input["option"].as_str()?.to_string();
            let targets = input["targets"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();
            Some(Intent::UseChannelDivinity {
                character_id: world.player_character.id,
                option,
                targets,
            })
        }
        "use_bardic_inspiration" => {
            let target_name = input["target"].as_str()?.to_string();
            let die_size = input["die_size"].as_str()?.to_string();
            Some(Intent::UseBardicInspiration {
                character_id: world.player_character.id,
                target_name,
                die_size,
            })
        }
        "use_action_surge" => {
            let action_taken = input["action_taken"].as_str()?.to_string();
            Some(Intent::UseActionSurge {
                character_id: world.player_character.id,
                action_taken,
            })
        }
        "use_second_wind" => Some(Intent::UseSecondWind {
            character_id: world.player_character.id,
        }),
        "use_sorcery_points" => {
            let points = input["points"].as_u64()? as u8;
            let metamagic = input["metamagic"].as_str()?.to_string();
            let spell_name = input["spell_name"].as_str().map(|s| s.to_string());
            let slot_level = input["slot_level"].as_u64().map(|l| l as u8);
            Some(Intent::UseSorceryPoints {
                character_id: world.player_character.id,
                points,
                metamagic,
                spell_name,
                slot_level,
            })
        }
        _ => None,
    }
}
