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
