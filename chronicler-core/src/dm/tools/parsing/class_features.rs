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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::{create_sample_fighter, GameWorld};
    use serde_json::json;

    fn setup_world() -> GameWorld {
        let character = create_sample_fighter("Roland");
        GameWorld::new("Test", character)
    }

    #[test]
    fn test_parse_use_rage() {
        let world = setup_world();
        let input = json!({});

        let intent = parse_class_features_tool("use_rage", &input, &world);

        assert!(intent.is_some());
        let intent = intent.unwrap();
        assert!(matches!(intent, Intent::UseRage { .. }));
    }

    #[test]
    fn test_parse_end_rage() {
        let world = setup_world();
        let input = json!({
            "reason": "duration_expired"
        });

        let intent = parse_class_features_tool("end_rage", &input, &world);

        assert!(intent.is_some());
        if let Some(Intent::EndRage { reason, .. }) = intent {
            assert_eq!(reason, "duration_expired");
        } else {
            panic!("Expected EndRage intent");
        }
    }

    #[test]
    fn test_parse_end_rage_default_reason() {
        let world = setup_world();
        let input = json!({});

        let intent = parse_class_features_tool("end_rage", &input, &world);

        assert!(intent.is_some());
        if let Some(Intent::EndRage { reason, .. }) = intent {
            assert_eq!(reason, "voluntary");
        }
    }

    #[test]
    fn test_parse_use_ki() {
        let world = setup_world();
        let input = json!({
            "points": 1,
            "ability": "flurry_of_blows"
        });

        let intent = parse_class_features_tool("use_ki", &input, &world);

        assert!(intent.is_some());
        if let Some(Intent::UseKi {
            points, ability, ..
        }) = intent
        {
            assert_eq!(points, 1);
            assert_eq!(ability, "flurry_of_blows");
        } else {
            panic!("Expected UseKi intent");
        }
    }

    #[test]
    fn test_parse_use_ki_missing_required_field() {
        let world = setup_world();
        let input = json!({
            "points": 1
            // Missing "ability"
        });

        let intent = parse_class_features_tool("use_ki", &input, &world);
        assert!(intent.is_none());
    }

    #[test]
    fn test_parse_use_lay_on_hands() {
        let world = setup_world();
        let input = json!({
            "target": "Wounded Ally",
            "hp_amount": 10,
            "cure_disease": true,
            "neutralize_poison": false
        });

        let intent = parse_class_features_tool("use_lay_on_hands", &input, &world);

        assert!(intent.is_some());
        if let Some(Intent::UseLayOnHands {
            target_name,
            hp_amount,
            cure_disease,
            neutralize_poison,
            ..
        }) = intent
        {
            assert_eq!(target_name, "Wounded Ally");
            assert_eq!(hp_amount, 10);
            assert!(cure_disease);
            assert!(!neutralize_poison);
        } else {
            panic!("Expected UseLayOnHands intent");
        }
    }

    #[test]
    fn test_parse_use_divine_smite() {
        let world = setup_world();
        let input = json!({
            "spell_slot_level": 2,
            "target_is_undead_or_fiend": true
        });

        let intent = parse_class_features_tool("use_divine_smite", &input, &world);

        assert!(intent.is_some());
        if let Some(Intent::UseDivineSmite {
            spell_slot_level,
            target_is_undead_or_fiend,
            ..
        }) = intent
        {
            assert_eq!(spell_slot_level, 2);
            assert!(target_is_undead_or_fiend);
        } else {
            panic!("Expected UseDivineSmite intent");
        }
    }

    #[test]
    fn test_parse_use_wild_shape() {
        let world = setup_world();
        let input = json!({
            "beast_form": "Wolf",
            "beast_hp": 11,
            "beast_ac": 13
        });

        let intent = parse_class_features_tool("use_wild_shape", &input, &world);

        assert!(intent.is_some());
        if let Some(Intent::UseWildShape {
            beast_form,
            beast_hp,
            beast_ac,
            ..
        }) = intent
        {
            assert_eq!(beast_form, "Wolf");
            assert_eq!(beast_hp, 11);
            assert_eq!(beast_ac, Some(13));
        } else {
            panic!("Expected UseWildShape intent");
        }
    }

    #[test]
    fn test_parse_end_wild_shape() {
        let world = setup_world();
        let input = json!({
            "reason": "hp_zero",
            "excess_damage": 5
        });

        let intent = parse_class_features_tool("end_wild_shape", &input, &world);

        assert!(intent.is_some());
        if let Some(Intent::EndWildShape {
            reason,
            excess_damage,
            ..
        }) = intent
        {
            assert_eq!(reason, "hp_zero");
            assert_eq!(excess_damage, 5);
        } else {
            panic!("Expected EndWildShape intent");
        }
    }

    #[test]
    fn test_parse_use_channel_divinity() {
        let world = setup_world();
        let input = json!({
            "option": "Turn Undead",
            "targets": ["Zombie", "Skeleton"]
        });

        let intent = parse_class_features_tool("use_channel_divinity", &input, &world);

        assert!(intent.is_some());
        if let Some(Intent::UseChannelDivinity {
            option, targets, ..
        }) = intent
        {
            assert_eq!(option, "Turn Undead");
            assert_eq!(targets.len(), 2);
            assert!(targets.contains(&"Zombie".to_string()));
        } else {
            panic!("Expected UseChannelDivinity intent");
        }
    }

    #[test]
    fn test_parse_use_bardic_inspiration() {
        let world = setup_world();
        let input = json!({
            "target": "Ally",
            "die_size": "d8"
        });

        let intent = parse_class_features_tool("use_bardic_inspiration", &input, &world);

        assert!(intent.is_some());
        if let Some(Intent::UseBardicInspiration {
            target_name,
            die_size,
            ..
        }) = intent
        {
            assert_eq!(target_name, "Ally");
            assert_eq!(die_size, "d8");
        } else {
            panic!("Expected UseBardicInspiration intent");
        }
    }

    #[test]
    fn test_parse_use_action_surge() {
        let world = setup_world();
        let input = json!({
            "action_taken": "Attack action"
        });

        let intent = parse_class_features_tool("use_action_surge", &input, &world);

        assert!(intent.is_some());
        if let Some(Intent::UseActionSurge { action_taken, .. }) = intent {
            assert_eq!(action_taken, "Attack action");
        } else {
            panic!("Expected UseActionSurge intent");
        }
    }

    #[test]
    fn test_parse_use_second_wind() {
        let world = setup_world();
        let input = json!({});

        let intent = parse_class_features_tool("use_second_wind", &input, &world);

        assert!(intent.is_some());
        assert!(matches!(intent, Some(Intent::UseSecondWind { .. })));
    }

    #[test]
    fn test_parse_use_sorcery_points() {
        let world = setup_world();
        let input = json!({
            "points": 2,
            "metamagic": "quickened",
            "spell_name": "Fireball",
            "slot_level": 3
        });

        let intent = parse_class_features_tool("use_sorcery_points", &input, &world);

        assert!(intent.is_some());
        if let Some(Intent::UseSorceryPoints {
            points,
            metamagic,
            spell_name,
            slot_level,
            ..
        }) = intent
        {
            assert_eq!(points, 2);
            assert_eq!(metamagic, "quickened");
            assert_eq!(spell_name, Some("Fireball".to_string()));
            assert_eq!(slot_level, Some(3));
        } else {
            panic!("Expected UseSorceryPoints intent");
        }
    }

    #[test]
    fn test_parse_unknown_tool() {
        let world = setup_world();
        let input = json!({});

        let intent = parse_class_features_tool("unknown_tool", &input, &world);
        assert!(intent.is_none());
    }
}
