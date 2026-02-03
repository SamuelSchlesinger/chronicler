//! Parsing for inventory-related tools.

use crate::rules::Intent;
use serde_json::Value;

/// Parse inventory-related tool calls.
pub fn parse_inventory_tool(name: &str, input: &Value) -> Option<Intent> {
    match name {
        "give_item" => {
            let item_name = input["item_name"].as_str()?.to_string();
            let quantity = input["quantity"].as_u64().unwrap_or(1) as u32;
            let item_type = input["item_type"].as_str().map(|s| s.to_string());
            let description = input["description"].as_str().map(|s| s.to_string());
            let magical = input["magical"].as_bool().unwrap_or(false);
            let weight = input["weight"].as_f64().map(|w| w as f32);
            let value_gp = input["value_gp"].as_f64().map(|v| v as f32);

            Some(Intent::AddItem {
                item_name,
                quantity,
                item_type,
                description,
                magical,
                weight,
                value_gp,
            })
        }
        "remove_item" => {
            let item_name = input["item_name"].as_str()?.to_string();
            let quantity = input["quantity"].as_u64().unwrap_or(1) as u32;

            Some(Intent::RemoveItem {
                item_name,
                quantity,
            })
        }
        "use_item" => {
            let item_name = input["item_name"].as_str()?.to_string();
            let target_id = None;

            Some(Intent::UseItem {
                item_name,
                target_id,
            })
        }
        "equip_item" => {
            let item_name = input["item_name"].as_str()?.to_string();
            Some(Intent::EquipItem { item_name })
        }
        "unequip_item" => {
            let slot = input["slot"].as_str()?.to_string();
            Some(Intent::UnequipItem { slot })
        }
        "adjust_gold" => {
            let amount = input["amount"].as_i64()? as i32;
            let reason = input["reason"]
                .as_str()
                .unwrap_or("gold adjustment")
                .to_string();
            Some(Intent::AdjustGold { amount, reason })
        }
        "adjust_silver" => {
            let amount = input["amount"].as_i64()? as i32;
            let reason = input["reason"]
                .as_str()
                .unwrap_or("silver adjustment")
                .to_string();
            Some(Intent::AdjustSilver { amount, reason })
        }
        // show_inventory is handled specially via execute_info_tool
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_give_item() {
        let input = json!({
            "item_name": "Longsword",
            "quantity": 1,
            "item_type": "weapon",
            "description": "A fine sword",
            "magical": false,
            "weight": 3.0,
            "value_gp": 15.0
        });

        let intent = parse_inventory_tool("give_item", &input);

        assert!(intent.is_some());
        if let Some(Intent::AddItem {
            item_name,
            quantity,
            item_type,
            description,
            magical,
            weight,
            value_gp,
        }) = intent
        {
            assert_eq!(item_name, "Longsword");
            assert_eq!(quantity, 1);
            assert_eq!(item_type, Some("weapon".to_string()));
            assert_eq!(description, Some("A fine sword".to_string()));
            assert!(!magical);
            assert_eq!(weight, Some(3.0));
            assert_eq!(value_gp, Some(15.0));
        } else {
            panic!("Expected AddItem intent");
        }
    }

    #[test]
    fn test_parse_give_item_minimal() {
        let input = json!({
            "item_name": "Arrow"
        });

        let intent = parse_inventory_tool("give_item", &input);

        assert!(intent.is_some());
        if let Some(Intent::AddItem {
            item_name,
            quantity,
            item_type,
            magical,
            ..
        }) = intent
        {
            assert_eq!(item_name, "Arrow");
            assert_eq!(quantity, 1); // Default
            assert!(item_type.is_none());
            assert!(!magical); // Default
        } else {
            panic!("Expected AddItem intent");
        }
    }

    #[test]
    fn test_parse_give_item_missing_name() {
        let input = json!({
            "quantity": 5
        });

        let intent = parse_inventory_tool("give_item", &input);
        assert!(intent.is_none());
    }

    #[test]
    fn test_parse_remove_item() {
        let input = json!({
            "item_name": "Healing Potion",
            "quantity": 2
        });

        let intent = parse_inventory_tool("remove_item", &input);

        assert!(intent.is_some());
        if let Some(Intent::RemoveItem {
            item_name,
            quantity,
        }) = intent
        {
            assert_eq!(item_name, "Healing Potion");
            assert_eq!(quantity, 2);
        } else {
            panic!("Expected RemoveItem intent");
        }
    }

    #[test]
    fn test_parse_remove_item_default_quantity() {
        let input = json!({
            "item_name": "Torch"
        });

        let intent = parse_inventory_tool("remove_item", &input);

        assert!(intent.is_some());
        if let Some(Intent::RemoveItem { quantity, .. }) = intent {
            assert_eq!(quantity, 1); // Default
        }
    }

    #[test]
    fn test_parse_use_item() {
        let input = json!({
            "item_name": "Potion of Healing"
        });

        let intent = parse_inventory_tool("use_item", &input);

        assert!(intent.is_some());
        if let Some(Intent::UseItem {
            item_name,
            target_id,
        }) = intent
        {
            assert_eq!(item_name, "Potion of Healing");
            assert!(target_id.is_none());
        } else {
            panic!("Expected UseItem intent");
        }
    }

    #[test]
    fn test_parse_equip_item() {
        let input = json!({
            "item_name": "Chain Mail"
        });

        let intent = parse_inventory_tool("equip_item", &input);

        assert!(intent.is_some());
        if let Some(Intent::EquipItem { item_name }) = intent {
            assert_eq!(item_name, "Chain Mail");
        } else {
            panic!("Expected EquipItem intent");
        }
    }

    #[test]
    fn test_parse_unequip_item() {
        let input = json!({
            "slot": "armor"
        });

        let intent = parse_inventory_tool("unequip_item", &input);

        assert!(intent.is_some());
        if let Some(Intent::UnequipItem { slot }) = intent {
            assert_eq!(slot, "armor");
        } else {
            panic!("Expected UnequipItem intent");
        }
    }

    #[test]
    fn test_parse_adjust_gold() {
        let input = json!({
            "amount": -50,
            "reason": "bought supplies"
        });

        let intent = parse_inventory_tool("adjust_gold", &input);

        assert!(intent.is_some());
        if let Some(Intent::AdjustGold { amount, reason }) = intent {
            assert_eq!(amount, -50);
            assert_eq!(reason, "bought supplies");
        } else {
            panic!("Expected AdjustGold intent");
        }
    }

    #[test]
    fn test_parse_adjust_gold_default_reason() {
        let input = json!({
            "amount": 100
        });

        let intent = parse_inventory_tool("adjust_gold", &input);

        assert!(intent.is_some());
        if let Some(Intent::AdjustGold { amount, reason }) = intent {
            assert_eq!(amount, 100);
            assert_eq!(reason, "gold adjustment"); // Default
        }
    }

    #[test]
    fn test_parse_adjust_silver() {
        let input = json!({
            "amount": 25,
            "reason": "tips"
        });

        let intent = parse_inventory_tool("adjust_silver", &input);

        assert!(intent.is_some());
        if let Some(Intent::AdjustSilver { amount, reason }) = intent {
            assert_eq!(amount, 25);
            assert_eq!(reason, "tips");
        } else {
            panic!("Expected AdjustSilver intent");
        }
    }

    #[test]
    fn test_parse_unknown_inventory_tool() {
        let input = json!({});

        let intent = parse_inventory_tool("unknown_tool", &input);
        assert!(intent.is_none());
    }
}
