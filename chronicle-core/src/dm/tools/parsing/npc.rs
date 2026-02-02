//! NPC tool parsing - converts NPC tool calls into game Intents.

use crate::rules::Intent;
use serde_json::Value;

/// Parse NPC-related tool calls into Intents.
pub fn parse_npc_tool(name: &str, input: &Value) -> Option<Intent> {
    match name {
        "create_npc" => {
            let npc_name = input.get("name")?.as_str()?.to_string();
            let description = input.get("description")?.as_str()?.to_string();
            let personality = input.get("personality")?.as_str()?.to_string();
            let occupation = input
                .get("occupation")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let disposition = input
                .get("disposition")
                .and_then(|v| v.as_str())
                .unwrap_or("neutral")
                .to_string();
            let location = input
                .get("location")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let known_information = input
                .get("known_information")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();

            Some(Intent::CreateNpc {
                name: npc_name,
                description,
                personality,
                occupation,
                disposition,
                location,
                known_information,
            })
        }

        "update_npc" => {
            let npc_name = input.get("npc_name")?.as_str()?.to_string();
            let disposition = input
                .get("disposition")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let add_information = input
                .get("add_information")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();
            let new_description = input
                .get("new_description")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let new_personality = input
                .get("new_personality")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            Some(Intent::UpdateNpc {
                npc_name,
                disposition,
                add_information,
                new_description,
                new_personality,
            })
        }

        "move_npc" => {
            let npc_name = input.get("npc_name")?.as_str()?.to_string();
            let destination = input.get("destination")?.as_str()?.to_string();
            let reason = input
                .get("reason")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            Some(Intent::MoveNpc {
                npc_name,
                destination,
                reason,
            })
        }

        "remove_npc" => {
            let npc_name = input.get("npc_name")?.as_str()?.to_string();
            let reason = input.get("reason")?.as_str()?.to_string();
            let permanent = input
                .get("permanent")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            Some(Intent::RemoveNpc {
                npc_name,
                reason,
                permanent,
            })
        }

        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_create_npc() {
        let input = json!({
            "name": "Grumble the Innkeeper",
            "description": "A stout dwarf with a braided beard and a perpetual scowl",
            "personality": "Gruff but secretly kind-hearted",
            "occupation": "Innkeeper",
            "disposition": "neutral",
            "location": "The Rusty Tankard",
            "known_information": ["The road to the north has been dangerous lately"]
        });

        let intent = parse_npc_tool("create_npc", &input);
        assert!(intent.is_some());

        if let Some(Intent::CreateNpc {
            name,
            description,
            occupation,
            disposition,
            location,
            known_information,
            ..
        }) = intent
        {
            assert_eq!(name, "Grumble the Innkeeper");
            assert!(description.contains("dwarf"));
            assert_eq!(occupation, Some("Innkeeper".to_string()));
            assert_eq!(disposition, "neutral");
            assert_eq!(location, Some("The Rusty Tankard".to_string()));
            assert_eq!(known_information.len(), 1);
        } else {
            panic!("Expected CreateNpc intent");
        }
    }

    #[test]
    fn test_update_npc() {
        let input = json!({
            "npc_name": "Grumble the Innkeeper",
            "disposition": "friendly",
            "add_information": ["Knows about a secret passage in the cellar"]
        });

        let intent = parse_npc_tool("update_npc", &input);
        assert!(intent.is_some());

        if let Some(Intent::UpdateNpc {
            npc_name,
            disposition,
            add_information,
            ..
        }) = intent
        {
            assert_eq!(npc_name, "Grumble the Innkeeper");
            assert_eq!(disposition, Some("friendly".to_string()));
            assert_eq!(add_information.len(), 1);
        } else {
            panic!("Expected UpdateNpc intent");
        }
    }

    #[test]
    fn test_move_npc() {
        let input = json!({
            "npc_name": "Grumble the Innkeeper",
            "destination": "Market Square",
            "reason": "Going to buy supplies"
        });

        let intent = parse_npc_tool("move_npc", &input);
        assert!(intent.is_some());

        if let Some(Intent::MoveNpc {
            npc_name,
            destination,
            reason,
        }) = intent
        {
            assert_eq!(npc_name, "Grumble the Innkeeper");
            assert_eq!(destination, "Market Square");
            assert_eq!(reason, Some("Going to buy supplies".to_string()));
        } else {
            panic!("Expected MoveNpc intent");
        }
    }

    #[test]
    fn test_remove_npc() {
        let input = json!({
            "npc_name": "The Bandit Leader",
            "reason": "Killed in combat by the party",
            "permanent": true
        });

        let intent = parse_npc_tool("remove_npc", &input);
        assert!(intent.is_some());

        if let Some(Intent::RemoveNpc {
            npc_name,
            reason,
            permanent,
        }) = intent
        {
            assert_eq!(npc_name, "The Bandit Leader");
            assert!(reason.contains("Killed"));
            assert!(permanent);
        } else {
            panic!("Expected RemoveNpc intent");
        }
    }
}
