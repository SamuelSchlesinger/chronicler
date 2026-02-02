//! Informational tools that return data without creating Intents.

use crate::dm::story_memory::StoryMemory;
use crate::world::GameWorld;
use serde_json::Value;

/// Execute an informational tool that may need StoryMemory access.
/// This is called when story_memory is available (from DungeonMaster).
pub fn execute_info_tool_with_memory(
    name: &str,
    input: &Value,
    world: &GameWorld,
    story_memory: &StoryMemory,
) -> Option<String> {
    match name {
        "show_inventory" => Some(format_inventory(world)),
        "query_state" => Some(query_entity_state(input, world)),
        "query_knowledge" => Some(query_entity_knowledge(input, world)),
        "check_schedule" => Some(check_schedule(input, world, story_memory)),
        _ => None,
    }
}

/// Check the schedule for upcoming events.
fn check_schedule(input: &Value, world: &GameWorld, story_memory: &StoryMemory) -> String {
    let location_filter = input["location"].as_str();
    let include_private = input["include_private"].as_bool().unwrap_or(false);

    let mut result = String::from("=== Upcoming Events ===\n\n");

    // Get events based on visibility preference
    let events: Vec<_> = if include_private {
        story_memory.pending_events()
    } else {
        story_memory.visible_pending_events()
    };

    // Filter by location if specified
    let events: Vec<_> = if let Some(loc) = location_filter {
        let loc_lower = loc.to_lowercase();
        events
            .into_iter()
            .filter(|e| {
                e.location
                    .as_ref()
                    .is_some_and(|l| l.to_lowercase().contains(&loc_lower))
            })
            .collect()
    } else {
        events
    };

    if events.is_empty() {
        if let Some(loc) = location_filter {
            result.push_str(&format!("No upcoming events at {}.\n", loc));
        } else {
            result.push_str("No upcoming events scheduled.\n");
        }
    } else {
        // Build the schedule summary using story_memory's helper
        let summary = story_memory.build_schedule_summary(&world.game_time);
        if !summary.is_empty() {
            result.push_str(&summary);
        } else {
            // Fallback if no visible events
            for event in events.iter().take(10) {
                let loc_str = event
                    .location
                    .as_ref()
                    .map(|l| format!(" at {}", l))
                    .unwrap_or_default();
                let vis_str = if include_private {
                    match event.visibility {
                        crate::dm::story_memory::EventVisibility::Private => " [PRIVATE]",
                        crate::dm::story_memory::EventVisibility::Hinted => " [hinted]",
                        _ => "",
                    }
                } else {
                    ""
                };
                result.push_str(&format!("- {}{}{}\n", event.description, loc_str, vis_str));
            }
        }
    }

    result
}

/// Query the state of an entity.
fn query_entity_state(input: &Value, world: &GameWorld) -> String {
    let entity_name = match input["entity_name"].as_str() {
        Some(name) => name,
        None => return "Error: entity_name is required".to_string(),
    };

    let state_type = input["state_type"].as_str().unwrap_or("all");

    // Find the NPC
    let npc = world
        .npcs
        .values()
        .find(|npc| npc.name.eq_ignore_ascii_case(entity_name));

    match npc {
        Some(npc) => {
            let mut result = format!("=== State of {} ===\n\n", npc.name);

            let disposition_str = match npc.disposition {
                crate::world::Disposition::Hostile => "hostile",
                crate::world::Disposition::Unfriendly => "unfriendly",
                crate::world::Disposition::Neutral => "neutral",
                crate::world::Disposition::Friendly => "friendly",
                crate::world::Disposition::Helpful => "helpful",
            };

            match state_type {
                "disposition" => {
                    result.push_str(&format!("Disposition: {}\n", disposition_str));
                }
                "location" => {
                    let location_name = npc
                        .location_id
                        .and_then(|id| world.known_locations.get(&id))
                        .map(|loc| loc.name.as_str())
                        .unwrap_or("Unknown");
                    result.push_str(&format!("Location: {}\n", location_name));
                }
                "knowledge" => {
                    if npc.known_information.is_empty() {
                        result.push_str("Knowledge: (none recorded)\n");
                    } else {
                        result.push_str("Knowledge:\n");
                        for info in &npc.known_information {
                            result.push_str(&format!("  - {}\n", info));
                        }
                    }
                }
                _ => {
                    result.push_str(&format!("Disposition: {}\n", disposition_str));
                    let location_name = npc
                        .location_id
                        .and_then(|id| world.known_locations.get(&id))
                        .map(|loc| loc.name.as_str())
                        .unwrap_or("Unknown");
                    result.push_str(&format!("Location: {}\n", location_name));
                    result.push_str(&format!(
                        "Occupation: {}\n",
                        npc.occupation.as_deref().unwrap_or("Unknown")
                    ));
                    if !npc.known_information.is_empty() {
                        result.push_str("Knowledge:\n");
                        for info in &npc.known_information {
                            result.push_str(&format!("  - {}\n", info));
                        }
                    }
                }
            }

            result
        }
        None => format!(
            "No entity found with name '{}'. Check spelling or create the NPC first.",
            entity_name
        ),
    }
}

/// Query what an entity knows.
fn query_entity_knowledge(input: &Value, world: &GameWorld) -> String {
    let entity_name = match input["entity_name"].as_str() {
        Some(name) => name,
        None => return "Error: entity_name is required".to_string(),
    };

    let topic = input["topic"].as_str();

    // Find the NPC
    let npc = world
        .npcs
        .values()
        .find(|npc| npc.name.eq_ignore_ascii_case(entity_name));

    match npc {
        Some(npc) => {
            let mut result = format!("=== Knowledge of {} ===\n\n", npc.name);

            if npc.known_information.is_empty() {
                result.push_str("No recorded knowledge.\n");
            } else {
                // Filter by topic if provided
                let knowledge: Vec<&String> = if let Some(topic_str) = topic {
                    let topic_lower = topic_str.to_lowercase();
                    npc.known_information
                        .iter()
                        .filter(|info| info.to_lowercase().contains(&topic_lower))
                        .collect()
                } else {
                    npc.known_information.iter().collect()
                };

                if knowledge.is_empty() {
                    if let Some(topic_str) = topic {
                        result.push_str(&format!("No knowledge about '{}'.\n", topic_str));
                    } else {
                        result.push_str("No recorded knowledge.\n");
                    }
                } else {
                    result.push_str(&format!("{} knows:\n", npc.name));
                    for info in knowledge {
                        result.push_str(&format!("  - {}\n", info));
                    }
                }
            }

            result
        }
        None => format!(
            "No entity found with name '{}'. Check spelling or create the NPC first.",
            entity_name
        ),
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

    fn create_test_story_memory() -> StoryMemory {
        StoryMemory::new()
    }

    #[test]
    fn test_info_tool_show_inventory() {
        let world = create_test_world();
        let story_memory = create_test_story_memory();
        let input = json!({});

        let result = execute_info_tool_with_memory("show_inventory", &input, &world, &story_memory);
        assert!(result.is_some());

        let inventory = result.unwrap();
        assert!(inventory.contains("Currency"));
        assert!(inventory.contains("gp"));
        assert!(inventory.contains("sp"));
    }

    #[test]
    fn test_info_tool_unknown() {
        let world = create_test_world();
        let story_memory = create_test_story_memory();
        let input = json!({});

        let result = execute_info_tool_with_memory("unknown_tool", &input, &world, &story_memory);
        assert!(result.is_none());
    }
}
