//! World building resolution methods (NPCs, locations, state assertions).

use crate::rules::types::{Effect, Resolution, StateType};
use crate::rules::RulesEngine;
use crate::world::GameWorld;

impl RulesEngine {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn resolve_create_npc(
        &self,
        world: &GameWorld,
        name: &str,
        _description: &str,
        _personality: &str,
        occupation: Option<&str>,
        disposition: &str,
        location: Option<&str>,
        _known_information: &[String],
    ) -> Resolution {
        // Check if an NPC with this name already exists (case-insensitive)
        let existing_npc = world
            .npcs
            .values()
            .find(|n| n.name.eq_ignore_ascii_case(name));

        if let Some(existing) = existing_npc {
            // NPC already exists - return an error with guidance
            return Resolution::new(format!(
                "DUPLICATE NPC ERROR: An NPC named '{}' already exists (disposition: {:?}). \
                Use 'update_npc' instead to modify their disposition, add information, or change their description. \
                Do NOT call create_npc again for this character.",
                existing.name,
                existing.disposition
            ));
        }

        let location_text = location.map(|l| format!(" at {}", l)).unwrap_or_default();
        let occupation_text = occupation.map(|o| format!(" ({})", o)).unwrap_or_default();

        Resolution::new(format!(
            "NPC {} ({}){}{}  enters the world",
            name, disposition, occupation_text, location_text
        ))
        .with_effect(Effect::NpcCreated {
            name: name.to_string(),
            location: location.map(|s| s.to_string()),
        })
    }

    pub(crate) fn resolve_update_npc(
        &self,
        world: &GameWorld,
        npc_name: &str,
        disposition: Option<&str>,
        add_information: &[String],
        new_description: Option<&str>,
        new_personality: Option<&str>,
    ) -> Resolution {
        // Check if NPC exists
        let npc_exists = world
            .npcs
            .values()
            .any(|n| n.name.eq_ignore_ascii_case(npc_name));

        if !npc_exists {
            return Resolution::new(format!("NPC '{}' not found in the world", npc_name));
        }

        let mut changes = Vec::new();
        if disposition.is_some() {
            changes.push("disposition changed");
        }
        if !add_information.is_empty() {
            changes.push("new information learned");
        }
        if new_description.is_some() {
            changes.push("description updated");
        }
        if new_personality.is_some() {
            changes.push("personality updated");
        }

        let changes_text = if changes.is_empty() {
            "no changes".to_string()
        } else {
            changes.join(", ")
        };

        Resolution::new(format!("NPC {} updated: {}", npc_name, changes_text)).with_effect(
            Effect::NpcUpdated {
                npc_name: npc_name.to_string(),
                changes: changes_text,
            },
        )
    }

    pub(crate) fn resolve_move_npc(
        &self,
        world: &GameWorld,
        npc_name: &str,
        destination: &str,
        reason: Option<&str>,
    ) -> Resolution {
        // Find the NPC's current location
        let from_location = world
            .npcs
            .values()
            .find(|n| n.name.eq_ignore_ascii_case(npc_name))
            .and_then(|n| n.location_id)
            .and_then(|loc_id| world.known_locations.get(&loc_id))
            .map(|loc| loc.name.clone());

        let reason_text = reason.map(|r| format!(" ({})", r)).unwrap_or_default();

        Resolution::new(format!(
            "NPC {} moves to {}{}",
            npc_name, destination, reason_text
        ))
        .with_effect(Effect::NpcMoved {
            npc_name: npc_name.to_string(),
            from_location,
            to_location: destination.to_string(),
        })
    }

    pub(crate) fn resolve_remove_npc(
        &self,
        npc_name: &str,
        reason: &str,
        permanent: bool,
    ) -> Resolution {
        let permanence = if permanent {
            "permanently"
        } else {
            "temporarily"
        };

        Resolution::new(format!(
            "NPC {} {} removed: {}",
            npc_name, permanence, reason
        ))
        .with_effect(Effect::NpcRemoved {
            npc_name: npc_name.to_string(),
            reason: reason.to_string(),
        })
    }

    pub(crate) fn resolve_create_location(
        &self,
        name: &str,
        location_type: &str,
        description: &str,
        parent_location: Option<&str>,
        items: &[String],
        npcs_present: &[String],
    ) -> Resolution {
        let parent_text = parent_location
            .map(|p| format!(" in {}", p))
            .unwrap_or_default();
        let items_text = if items.is_empty() {
            String::new()
        } else {
            format!(" with items: {}", items.join(", "))
        };
        let npcs_text = if npcs_present.is_empty() {
            String::new()
        } else {
            format!(" featuring NPCs: {}", npcs_present.join(", "))
        };

        Resolution::new(format!(
            "New location created: {} ({}){}{}{} - {}",
            name, location_type, parent_text, items_text, npcs_text, description
        ))
        .with_effect(Effect::LocationCreated {
            name: name.to_string(),
            location_type: location_type.to_string(),
        })
    }

    pub(crate) fn resolve_connect_locations(
        &self,
        from_location: &str,
        to_location: &str,
        direction: Option<&str>,
        travel_time_minutes: Option<u32>,
        bidirectional: bool,
    ) -> Resolution {
        let direction_text = direction
            .map(|d| format!(" ({} direction)", d))
            .unwrap_or_default();
        let travel_text = travel_time_minutes
            .map(|t| format!(", {} minutes travel time", t))
            .unwrap_or_default();
        let bidirectional_text = if bidirectional {
            " (bidirectional)"
        } else {
            " (one-way)"
        };

        Resolution::new(format!(
            "Locations connected: {} to {}{}{}{}",
            from_location, to_location, direction_text, travel_text, bidirectional_text
        ))
        .with_effect(Effect::LocationsConnected {
            from: from_location.to_string(),
            to: to_location.to_string(),
            direction: direction.map(|s| s.to_string()),
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn resolve_update_location(
        &self,
        world: &GameWorld,
        location_name: &str,
        new_description: Option<&str>,
        add_items: &[String],
        remove_items: &[String],
        add_npcs: &[String],
        remove_npcs: &[String],
    ) -> Resolution {
        // Check if location exists
        let location_exists = world
            .known_locations
            .values()
            .any(|l| l.name.eq_ignore_ascii_case(location_name))
            || world
                .current_location
                .name
                .eq_ignore_ascii_case(location_name);

        if !location_exists {
            return Resolution::new(format!(
                "Location '{}' not found in the world",
                location_name
            ));
        }

        let mut changes: Vec<String> = Vec::new();
        if new_description.is_some() {
            changes.push("description updated".to_string());
        }
        if !add_items.is_empty() {
            changes.push(format!("added items: {}", add_items.join(", ")));
        }
        if !remove_items.is_empty() {
            changes.push(format!("removed items: {}", remove_items.join(", ")));
        }
        if !add_npcs.is_empty() {
            changes.push(format!("NPCs arrived: {}", add_npcs.join(", ")));
        }
        if !remove_npcs.is_empty() {
            changes.push(format!("NPCs left: {}", remove_npcs.join(", ")));
        }

        let changes_text = if changes.is_empty() {
            "no changes".to_string()
        } else {
            changes.join("; ")
        };

        Resolution::new(format!(
            "Location {} updated: {}",
            location_name, changes_text
        ))
        .with_effect(Effect::LocationUpdated {
            location_name: location_name.to_string(),
            changes: changes_text,
        })
    }

    pub(crate) fn resolve_assert_state(
        &self,
        world: &GameWorld,
        entity_name: &str,
        state_type: StateType,
        new_value: &str,
        reason: &str,
        target_entity: Option<&str>,
    ) -> Resolution {
        // Try to find the NPC to get old value
        let old_value = world
            .npcs
            .values()
            .find(|npc| npc.name.eq_ignore_ascii_case(entity_name))
            .and_then(|npc| {
                match state_type {
                    StateType::Disposition => {
                        let disp_str = match npc.disposition {
                            crate::world::Disposition::Hostile => "hostile",
                            crate::world::Disposition::Unfriendly => "unfriendly",
                            crate::world::Disposition::Neutral => "neutral",
                            crate::world::Disposition::Friendly => "friendly",
                            crate::world::Disposition::Helpful => "helpful",
                        };
                        Some(disp_str.to_string())
                    }
                    StateType::Location => npc
                        .location_id
                        .and_then(|id| world.known_locations.get(&id))
                        .map(|loc| loc.name.clone()),
                    StateType::Status => None, // NPC struct doesn't have a status field
                    _ => None,
                }
            });

        let narrative = match state_type {
            StateType::Disposition => {
                format!(
                    "{}'s disposition is now {} (reason: {})",
                    entity_name, new_value, reason
                )
            }
            StateType::Location => {
                format!(
                    "{} is now at {} (reason: {})",
                    entity_name, new_value, reason
                )
            }
            StateType::Status => {
                format!(
                    "{}'s status is now {} (reason: {})",
                    entity_name, new_value, reason
                )
            }
            StateType::Knowledge => {
                format!(
                    "{} now knows: {} (reason: {})",
                    entity_name, new_value, reason
                )
            }
            StateType::Relationship => {
                if let Some(target) = target_entity {
                    format!(
                        "{}'s relationship with {} is now {} (reason: {})",
                        entity_name, target, new_value, reason
                    )
                } else {
                    format!(
                        "{}'s relationship status: {} (reason: {})",
                        entity_name, new_value, reason
                    )
                }
            }
        };

        Resolution::new(narrative).with_effect(Effect::StateAsserted {
            entity_name: entity_name.to_string(),
            state_type,
            old_value,
            new_value: new_value.to_string(),
            reason: reason.to_string(),
            target_entity: target_entity.map(|s| s.to_string()),
        })
    }

    pub(crate) fn resolve_share_knowledge(
        &self,
        knowing_entity: &str,
        content: &str,
        source: &str,
        verification: &str,
        context: Option<&str>,
    ) -> Resolution {
        let narrative = if let Some(ctx) = context {
            format!(
                "{} now knows: \"{}\" (from: {}, {}) [{}]",
                knowing_entity, content, source, verification, ctx
            )
        } else {
            format!(
                "{} now knows: \"{}\" (from: {}, {})",
                knowing_entity, content, source, verification
            )
        };

        Resolution::new(narrative).with_effect(Effect::KnowledgeShared {
            knowing_entity: knowing_entity.to_string(),
            content: content.to_string(),
            source: source.to_string(),
            verification: verification.to_string(),
            context: context.map(|s| s.to_string()),
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn resolve_schedule_event(
        &self,
        _world: &GameWorld,
        description: &str,
        minutes: Option<u32>,
        hours: Option<u32>,
        day: Option<u8>,
        month: Option<u8>,
        year: Option<i32>,
        hour: Option<u8>,
        daily_hour: Option<u8>,
        daily_minute: Option<u8>,
        location: Option<&str>,
        visibility: &str,
        repeating: bool,
    ) -> Resolution {
        // Build the trigger description
        let trigger_description = if let (Some(dh), Some(dm)) = (daily_hour, daily_minute) {
            if repeating {
                format!("daily at {:02}:{:02}", dh, dm)
            } else {
                format!("at {:02}:{:02}", dh, dm)
            }
        } else if let (Some(d), Some(m), Some(y)) = (day, month, year) {
            if let Some(h) = hour {
                format!("on {}/{}/{} at {:02}:00", m, d, y, h)
            } else {
                format!("on {}/{}/{}", m, d, y)
            }
        } else if minutes.is_some() || hours.is_some() {
            let total_minutes = minutes.unwrap_or(0) + hours.unwrap_or(0) * 60;
            if total_minutes < 60 {
                format!("in {} minutes", total_minutes)
            } else if total_minutes < 1440 {
                let h = total_minutes / 60;
                let m = total_minutes % 60;
                if m > 0 {
                    format!("in {} hours and {} minutes", h, m)
                } else {
                    format!("in {} hours", h)
                }
            } else {
                let days = total_minutes / 1440;
                format!("in {} days", days)
            }
        } else {
            "at an unspecified time".to_string()
        };

        let loc_desc = location.map(|l| format!(" at {}", l)).unwrap_or_default();

        let vis_desc = match visibility {
            "private" | "secret" => " (private)",
            "hinted" => " (hinted)",
            _ => "",
        };

        let narrative = format!(
            "Scheduled: \"{}\" {}{}{}",
            description, trigger_description, loc_desc, vis_desc
        );

        Resolution::new(narrative).with_effect(Effect::EventScheduled {
            description: description.to_string(),
            trigger_description,
            location: location.map(|s| s.to_string()),
            visibility: visibility.to_string(),
        })
    }

    pub(crate) fn resolve_cancel_event(&self, event_description: &str, reason: &str) -> Resolution {
        let narrative = format!("Event cancelled: \"{}\" - {}", event_description, reason);

        Resolution::new(narrative).with_effect(Effect::EventCancelled {
            description: event_description.to_string(),
            reason: reason.to_string(),
        })
    }
}
