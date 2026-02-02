//! Location tool parsing - converts location tool calls into game Intents.

use crate::rules::Intent;
use serde_json::Value;

/// Parse location-related tool calls into Intents.
pub fn parse_locations_tool(name: &str, input: &Value) -> Option<Intent> {
    match name {
        "create_location" => {
            let location_name = input.get("name")?.as_str()?.to_string();
            let location_type = input.get("location_type")?.as_str()?.to_string();
            let description = input.get("description")?.as_str()?.to_string();
            let parent_location = input
                .get("parent_location")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let items = input
                .get("items")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();
            let npcs_present = input
                .get("npcs_present")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();

            Some(Intent::CreateLocation {
                name: location_name,
                location_type,
                description,
                parent_location,
                items,
                npcs_present,
            })
        }

        "connect_locations" => {
            let from_location = input.get("from_location")?.as_str()?.to_string();
            let to_location = input.get("to_location")?.as_str()?.to_string();
            let direction = input
                .get("direction")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let travel_time_minutes = input
                .get("travel_time_minutes")
                .and_then(|v| v.as_u64())
                .map(|t| t as u32);
            let bidirectional = input
                .get("bidirectional")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);

            Some(Intent::ConnectLocations {
                from_location,
                to_location,
                direction,
                travel_time_minutes,
                bidirectional,
            })
        }

        "update_location" => {
            let location_name = input.get("location_name")?.as_str()?.to_string();
            let new_description = input
                .get("new_description")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let add_items = input
                .get("add_items")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();
            let remove_items = input
                .get("remove_items")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();
            let add_npcs = input
                .get("add_npcs")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();
            let remove_npcs = input
                .get("remove_npcs")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();

            Some(Intent::UpdateLocation {
                location_name,
                new_description,
                add_items,
                remove_items,
                add_npcs,
                remove_npcs,
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
    fn test_create_location() {
        let input = json!({
            "name": "The Rusty Tankard",
            "location_type": "building",
            "description": "A cozy tavern with a roaring fireplace and the smell of ale",
            "parent_location": "Riverside Village",
            "items": ["Dusty bottle of wine", "Mysterious letter"],
            "npcs_present": ["Grumble the Innkeeper"]
        });

        let intent = parse_locations_tool("create_location", &input);
        assert!(intent.is_some());

        if let Some(Intent::CreateLocation {
            name,
            location_type,
            description,
            parent_location,
            items,
            npcs_present,
        }) = intent
        {
            assert_eq!(name, "The Rusty Tankard");
            assert_eq!(location_type, "building");
            assert!(description.contains("tavern"));
            assert_eq!(parent_location, Some("Riverside Village".to_string()));
            assert_eq!(items.len(), 2);
            assert_eq!(npcs_present.len(), 1);
        } else {
            panic!("Expected CreateLocation intent");
        }
    }

    #[test]
    fn test_connect_locations() {
        let input = json!({
            "from_location": "Riverside Village",
            "to_location": "Dark Forest",
            "direction": "north",
            "travel_time_minutes": 30,
            "bidirectional": true
        });

        let intent = parse_locations_tool("connect_locations", &input);
        assert!(intent.is_some());

        if let Some(Intent::ConnectLocations {
            from_location,
            to_location,
            direction,
            travel_time_minutes,
            bidirectional,
        }) = intent
        {
            assert_eq!(from_location, "Riverside Village");
            assert_eq!(to_location, "Dark Forest");
            assert_eq!(direction, Some("north".to_string()));
            assert_eq!(travel_time_minutes, Some(30));
            assert!(bidirectional);
        } else {
            panic!("Expected ConnectLocations intent");
        }
    }

    #[test]
    fn test_update_location() {
        let input = json!({
            "location_name": "The Rusty Tankard",
            "new_description": "The tavern is now bustling with activity",
            "add_items": ["Gold coins on the bar"],
            "remove_items": ["Dusty bottle of wine"],
            "add_npcs": ["Mysterious Stranger"],
            "remove_npcs": []
        });

        let intent = parse_locations_tool("update_location", &input);
        assert!(intent.is_some());

        if let Some(Intent::UpdateLocation {
            location_name,
            new_description,
            add_items,
            remove_items,
            add_npcs,
            remove_npcs,
        }) = intent
        {
            assert_eq!(location_name, "The Rusty Tankard");
            assert!(new_description.is_some());
            assert_eq!(add_items.len(), 1);
            assert_eq!(remove_items.len(), 1);
            assert_eq!(add_npcs.len(), 1);
            assert!(remove_npcs.is_empty());
        } else {
            panic!("Expected UpdateLocation intent");
        }
    }
}
