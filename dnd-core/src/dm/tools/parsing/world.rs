//! Parsing for world/session-related tools.

use crate::rules::Intent;
use crate::world::GameWorld;
use serde_json::Value;

/// Parse world/session-related tool calls.
pub fn parse_world_tool(name: &str, input: &Value, world: &GameWorld) -> Option<Intent> {
    match name {
        "short_rest" => Some(Intent::ShortRest),
        "long_rest" => Some(Intent::LongRest),
        "change_location" => {
            let new_location = input["new_location"].as_str()?.to_string();
            let location_type = input["location_type"].as_str().map(|s| s.to_string());
            let description = input["description"].as_str().map(|s| s.to_string());
            Some(Intent::ChangeLocation {
                new_location,
                location_type,
                description,
            })
        }
        "remember_fact" => {
            let subject_name = input["subject_name"].as_str()?.to_string();
            let subject_type = input["subject_type"].as_str()?.to_string();
            let fact = input["fact"].as_str()?.to_string();
            let category = input["category"].as_str()?.to_string();
            let related_entities = input["related_entities"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();
            let importance = input["importance"].as_f64().unwrap_or(0.7) as f32;

            Some(Intent::RememberFact {
                subject_name,
                subject_type,
                fact,
                category,
                related_entities,
                importance,
            })
        }
        "register_consequence" => {
            let trigger_description = input["trigger_description"].as_str()?.to_string();
            let consequence_description = input["consequence_description"].as_str()?.to_string();
            let severity = input["severity"].as_str()?.to_string();
            let related_entities = input["related_entities"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();
            let default_importance = match severity.as_str() {
                "minor" => 0.3,
                "moderate" => 0.5,
                "major" => 0.8,
                "critical" => 1.0,
                _ => 0.5,
            };
            let importance = input["importance"].as_f64().unwrap_or(default_importance) as f32;
            let expires_in_turns = input["expires_in_turns"].as_u64().map(|v| v as u32);

            Some(Intent::RegisterConsequence {
                trigger_description,
                consequence_description,
                severity,
                related_entities,
                importance,
                expires_in_turns,
            })
        }
        "cast_spell" => {
            let spell_name = input["spell_name"].as_str()?.to_string();
            let slot_level = input["slot_level"].as_u64().unwrap_or(0) as u8;
            let targets = input["targets"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();

            Some(Intent::CastSpell {
                caster_id: world.player_character.id,
                spell_name,
                targets: vec![],
                spell_level: slot_level,
                target_names: targets,
            })
        }
        "award_experience" => {
            let amount = input["amount"].as_u64()? as u32;
            Some(Intent::GainExperience { amount })
        }
        _ => None,
    }
}
