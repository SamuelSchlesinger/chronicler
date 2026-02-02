//! Parsing for state assertion tools.

use crate::rules::{Intent, StateType};
use serde_json::Value;

/// Parse state-related tool calls.
pub fn parse_state_tool(name: &str, input: &Value) -> Option<Intent> {
    match name {
        "assert_state" => {
            let entity_name = input["entity_name"].as_str()?.to_string();
            let state_type_str = input["state_type"].as_str()?;
            let state_type = StateType::parse(state_type_str)?;
            let new_value = input["new_value"].as_str()?.to_string();
            let reason = input["reason"].as_str()?.to_string();
            let target_entity = input["target_entity"].as_str().map(|s| s.to_string());

            Some(Intent::AssertState {
                entity_name,
                state_type,
                new_value,
                reason,
                target_entity,
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
    fn test_parse_assert_state_disposition() {
        let input = json!({
            "entity_name": "Mira",
            "state_type": "disposition",
            "new_value": "friendly",
            "reason": "Player saved her shop"
        });

        let intent = parse_state_tool("assert_state", &input);
        assert!(intent.is_some());

        if let Some(Intent::AssertState {
            entity_name,
            state_type,
            new_value,
            reason,
            target_entity,
        }) = intent
        {
            assert_eq!(entity_name, "Mira");
            assert_eq!(state_type, StateType::Disposition);
            assert_eq!(new_value, "friendly");
            assert_eq!(reason, "Player saved her shop");
            assert!(target_entity.is_none());
        } else {
            panic!("Expected AssertState intent");
        }
    }

    #[test]
    fn test_parse_assert_state_relationship() {
        let input = json!({
            "entity_name": "Mira",
            "state_type": "relationship",
            "new_value": "ally",
            "reason": "Trust built through shared adventure",
            "target_entity": "Player"
        });

        let intent = parse_state_tool("assert_state", &input);
        assert!(intent.is_some());

        if let Some(Intent::AssertState {
            entity_name,
            state_type,
            new_value,
            reason,
            target_entity,
        }) = intent
        {
            assert_eq!(entity_name, "Mira");
            assert_eq!(state_type, StateType::Relationship);
            assert_eq!(new_value, "ally");
            assert_eq!(reason, "Trust built through shared adventure");
            assert_eq!(target_entity, Some("Player".to_string()));
        } else {
            panic!("Expected AssertState intent");
        }
    }

    #[test]
    fn test_parse_assert_state_location() {
        let input = json!({
            "entity_name": "Guard Captain",
            "state_type": "location",
            "new_value": "Northern Gate",
            "reason": "Responding to alarm"
        });

        let intent = parse_state_tool("assert_state", &input);
        assert!(intent.is_some());

        if let Some(Intent::AssertState {
            entity_name,
            state_type,
            new_value,
            ..
        }) = intent
        {
            assert_eq!(entity_name, "Guard Captain");
            assert_eq!(state_type, StateType::Location);
            assert_eq!(new_value, "Northern Gate");
        } else {
            panic!("Expected AssertState intent");
        }
    }

    #[test]
    fn test_parse_assert_state_invalid_type() {
        let input = json!({
            "entity_name": "Mira",
            "state_type": "invalid_type",
            "new_value": "friendly",
            "reason": "Test"
        });

        let intent = parse_state_tool("assert_state", &input);
        assert!(intent.is_none());
    }

    #[test]
    fn test_parse_assert_state_missing_fields() {
        let input = json!({
            "entity_name": "Mira"
        });

        let intent = parse_state_tool("assert_state", &input);
        assert!(intent.is_none());
    }
}
