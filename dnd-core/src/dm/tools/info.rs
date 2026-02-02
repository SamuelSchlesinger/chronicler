//! Informational tools that return data without creating Intents.

use crate::world::GameWorld;
use serde_json::Value;

/// Execute an informational tool that returns data without creating an Intent.
/// Returns Some(result_string) if the tool is an info tool, None otherwise.
pub fn execute_info_tool(name: &str, _input: &Value, world: &GameWorld) -> Option<String> {
    match name {
        "show_inventory" => Some(format_inventory(world)),
        _ => None,
    }
}

/// Format the player's inventory for display.
fn format_inventory(world: &GameWorld) -> String {
    let character = &world.player_character;
    let mut output = String::new();

    output.push_str(&format!("=== {}'s Inventory ===\n\n", character.name));

    // Currency
    output.push_str(&format!(
        "Currency: {} gp, {} sp\n\n",
        character.inventory.gold, character.inventory.silver
    ));

    // Current AC
    output.push_str(&format!("Current AC: {}\n\n", character.current_ac()));

    // Equipment
    output.push_str("Equipment:\n");
    if let Some(ref armor) = character.equipment.armor {
        let armor_type_str = match armor.armor_type {
            crate::world::ArmorType::Light => "Light",
            crate::world::ArmorType::Medium => "Medium",
            crate::world::ArmorType::Heavy => "Heavy",
        };
        let stealth_str = if armor.stealth_disadvantage {
            " [Stealth Disadvantage]"
        } else {
            ""
        };
        output.push_str(&format!(
            "  Armor: {} ({} armor, base AC {}){}\n",
            armor.base.name, armor_type_str, armor.base_ac, stealth_str
        ));
    } else {
        output.push_str("  Armor: None (unarmored)\n");
    }
    if let Some(ref shield) = character.equipment.shield {
        output.push_str(&format!("  Shield: {} (+2 AC)\n", shield.name));
    } else {
        output.push_str("  Shield: None\n");
    }
    if let Some(ref weapon) = character.equipment.main_hand {
        let two_handed = if weapon.is_two_handed() {
            " [Two-Handed]"
        } else {
            ""
        };
        output.push_str(&format!(
            "  Main Hand: {} ({} {}){}\n",
            weapon.base.name,
            weapon.damage_dice,
            weapon.damage_type.name(),
            two_handed
        ));
    } else {
        output.push_str("  Main Hand: Empty\n");
    }
    if let Some(ref item) = character.equipment.off_hand {
        output.push_str(&format!("  Off Hand: {}\n", item.name));
    }

    // Inventory items
    if character.inventory.items.is_empty() {
        output.push_str("\nInventory: Empty\n");
    } else {
        output.push_str("\nInventory:\n");
        for item in &character.inventory.items {
            let qty_str = if item.quantity > 1 {
                format!(" (x{})", item.quantity)
            } else {
                String::new()
            };
            let value_str = if item.value_gp > 0.0 {
                format!(" [{:.0} gp]", item.value_gp)
            } else {
                String::new()
            };
            output.push_str(&format!("  - {}{}{}\n", item.name, qty_str, value_str));
        }
    }

    output.push_str(&format!(
        "\nTotal Weight: {:.1} lb\n",
        character.inventory.total_weight()
    ));

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::{Character, CharacterClass, ClassLevel, GameWorld};
    use serde_json::json;

    fn create_test_world() -> GameWorld {
        let mut character = Character::new("Test Hero");
        character.classes.push(ClassLevel {
            class: CharacterClass::Fighter,
            level: 1,
            subclass: None,
        });
        GameWorld::new("Test Campaign", character)
    }

    #[test]
    fn test_info_tool_show_inventory() {
        let world = create_test_world();
        let input = json!({});

        let result = execute_info_tool("show_inventory", &input, &world);
        assert!(result.is_some());

        let inventory = result.unwrap();
        assert!(inventory.contains("Currency"));
        assert!(inventory.contains("gp"));
        assert!(inventory.contains("sp"));
    }

    #[test]
    fn test_info_tool_unknown() {
        let world = create_test_world();
        let input = json!({});

        let result = execute_info_tool("unknown_tool", &input, &world);
        assert!(result.is_none());
    }
}
