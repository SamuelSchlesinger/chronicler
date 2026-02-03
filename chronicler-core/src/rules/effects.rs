//! Effect application to the game world.

use crate::rules::types::{Effect, RestType, StateType};
use crate::world::{
    Ability, CharacterClass, Combatant, Condition, GameWorld, Item, ItemType, SlotInfo, SpellSlots,
    SpellcastingData,
};

/// Apply effects to the game world.
pub fn apply_effects(world: &mut GameWorld, effects: &[Effect]) {
    for effect in effects {
        apply_effect(world, effect);
    }
}

/// Apply a single effect to the game world.
pub fn apply_effect(world: &mut GameWorld, effect: &Effect) {
    match effect {
        Effect::HpChanged {
            amount,
            dropped_to_zero,
            ..
        } => {
            let was_unconscious = world.player_character.hit_points.current <= 0;

            if *amount < 0 {
                world.player_character.hit_points.take_damage(-*amount);
            } else {
                world.player_character.hit_points.heal(*amount);
            }

            // Add Unconscious condition if dropped to 0 (only if not already unconscious)
            if *dropped_to_zero {
                world
                    .player_character
                    .add_condition(Condition::Unconscious, "Dropped to 0 HP");
            }

            // Remove Unconscious condition and reset death saves if healed above 0
            if was_unconscious && world.player_character.hit_points.current > 0 {
                world
                    .player_character
                    .conditions
                    .retain(|c| c.condition != Condition::Unconscious);
                // Reset death saves when regaining consciousness
                world.player_character.death_saves.reset();
            }

            // Sync HP to combat state if in combat
            if let Some(ref mut combat) = world.combat {
                let player_id = world.player_character.id;
                combat.update_combatant_hp(player_id, world.player_character.hit_points.current);
            }
        }
        Effect::ConditionApplied {
            condition,
            source,
            duration_rounds,
            ..
        } => {
            world.player_character.add_condition_with_duration(
                *condition,
                source.clone(),
                *duration_rounds,
            );
        }
        Effect::ConditionRemoved { condition, .. } => {
            world
                .player_character
                .conditions
                .retain(|c| c.condition != *condition);
        }
        Effect::CombatStarted => {
            world.start_combat();
        }
        Effect::CombatEnded => {
            world.end_combat();
        }
        Effect::CombatantAdded {
            id,
            name,
            initiative,
            is_ally,
            current_hp,
            max_hp,
            armor_class,
        } => {
            if let Some(ref mut combat) = world.combat {
                combat.add_combatant(Combatant {
                    id: *id,
                    name: name.clone(),
                    initiative: *initiative,
                    is_player: *id == world.player_character.id,
                    is_ally: *is_ally,
                    current_hp: *current_hp,
                    max_hp: *max_hp,
                    armor_class: *armor_class,
                });
            }
        }
        Effect::TurnAdvanced { .. } => {
            if let Some(ref mut combat) = world.combat {
                combat.next_turn();
            }

            // Decrement condition durations and remove expired conditions
            world.player_character.conditions.retain_mut(|c| {
                if let Some(ref mut duration) = c.duration_rounds {
                    if *duration > 0 {
                        *duration -= 1;
                    }
                    *duration > 0 // Keep only if duration remaining
                } else {
                    true // Keep permanent conditions
                }
            });
        }
        Effect::TimeAdvanced { minutes } => {
            world.game_time.advance_minutes(*minutes);
        }
        Effect::RestCompleted { rest_type } => match rest_type {
            RestType::Short => world.short_rest(),
            RestType::Long => world.long_rest(),
        },
        Effect::ExperienceGained { amount, .. } => {
            world.player_character.experience += amount;
        }
        Effect::LevelUp { new_level } => {
            let character = &mut world.player_character;
            let old_level = character.level;
            character.level = *new_level;

            // Get the primary class for level-up calculations
            if let Some(class_level) = character.classes.first_mut() {
                let class = class_level.class;
                let hit_die = class.hit_die();

                // Update class level
                class_level.level = *new_level;

                // Add HP: roll hit die + CON modifier (use average for consistency)
                // Average is (max/2 + 1), e.g., d8 = 5, d10 = 6, d12 = 7
                let hit_die_average = (hit_die.sides() / 2 + 1) as i32;
                let con_mod = character.ability_scores.modifier(Ability::Constitution) as i32;
                let hp_gained = (hit_die_average + con_mod).max(1);
                character.hit_points.maximum += hp_gained;
                character.hit_points.current += hp_gained;

                // Add a hit die
                character.hit_dice.add(hit_die, 1);

                // Update spell slots for spellcasters
                if class.spellcasting_ability().is_some() {
                    let new_slots = class.spell_slots_at_level(*new_level);

                    if let Some(ref mut spellcasting) = character.spellcasting {
                        for (i, &total) in new_slots.iter().enumerate() {
                            let old_total = spellcasting.spell_slots.slots[i].total;
                            spellcasting.spell_slots.slots[i].total = total;
                            // Restore any new slots gained
                            if total > old_total {
                                let gained = total - old_total;
                                spellcasting.spell_slots.slots[i].used =
                                    spellcasting.spell_slots.slots[i]
                                        .used
                                        .saturating_sub(gained);
                            }
                        }
                    } else if new_slots.iter().any(|&s| s > 0) {
                        // Class just gained spellcasting (e.g., Paladin/Ranger at level 2)
                        character.spellcasting = Some(SpellcastingData {
                            ability: class.spellcasting_ability().unwrap(),
                            spells_known: Vec::new(),
                            spells_prepared: Vec::new(),
                            cantrips_known: Vec::new(),
                            spell_slots: SpellSlots {
                                slots: std::array::from_fn(|i| SlotInfo {
                                    total: new_slots[i],
                                    used: 0,
                                }),
                            },
                        });
                    }

                    // Track spell learning capacity changes for narrative purposes
                    // (actual spell selection happens via DM tools or character choices)
                    let old_cantrips = class.cantrips_known_at_level(old_level);
                    let new_cantrips = class.cantrips_known_at_level(*new_level);
                    let _cantrips_gained = new_cantrips.saturating_sub(old_cantrips);

                    // For "known" casters, track new spell capacity
                    if let Some(old_known) = class.spells_known_at_level(old_level) {
                        if let Some(new_known) = class.spells_known_at_level(*new_level) {
                            let _spells_gained = new_known.saturating_sub(old_known);
                            // Player can now learn more spells up to new_known total
                        }
                    }

                    // For Wizard, track spellbook expansion
                    let _wizard_spells_added = class.wizard_spellbook_spells_at_level(*new_level);
                }

                // Update class resources based on class and level
                match class {
                    CharacterClass::Monk => {
                        // Ki points = Monk level
                        character.class_resources.max_ki_points = *new_level;
                        character.class_resources.ki_points = *new_level;
                    }
                    CharacterClass::Sorcerer => {
                        // Sorcery points = Sorcerer level (gained at level 2)
                        if *new_level >= 2 {
                            character.class_resources.max_sorcery_points = *new_level;
                            // Give the new points
                            let gained = *new_level - old_level;
                            character.class_resources.sorcery_points =
                                (character.class_resources.sorcery_points + gained)
                                    .min(character.class_resources.max_sorcery_points);
                        }
                    }
                    CharacterClass::Paladin => {
                        // Lay on Hands pool = 5 × Paladin level
                        character.class_resources.lay_on_hands_max = (*new_level as u32) * 5;
                        // Restore to full on level up
                        character.class_resources.lay_on_hands_pool =
                            character.class_resources.lay_on_hands_max;
                    }
                    CharacterClass::Barbarian => {
                        // Rage uses increase at certain levels
                        let rage_uses = match *new_level {
                            1..=2 => 2,
                            3..=5 => 3,
                            6..=11 => 4,
                            12..=16 => 5,
                            17..=19 => 6,
                            20 => u8::MAX, // Unlimited at 20
                            _ => 2,
                        };
                        // Update rage feature uses
                        if let Some(rage_feature) =
                            character.features.iter_mut().find(|f| f.name == "Rage")
                        {
                            if let Some(ref mut uses) = rage_feature.uses {
                                uses.maximum = rage_uses;
                                uses.current = rage_uses;
                            }
                        }
                        // Rage damage bonus increases at levels 9 and 16
                        character.class_resources.rage_damage_bonus = match *new_level {
                            1..=8 => 2,
                            9..=15 => 3,
                            16..=20 => 4,
                            _ => 2,
                        };
                    }
                    _ => {}
                }
            }
        }
        Effect::FeatureUsed {
            feature_name,
            uses_remaining,
        } => {
            if let Some(feature) = world
                .player_character
                .features
                .iter_mut()
                .find(|f| f.name == *feature_name)
            {
                if let Some(ref mut uses) = feature.uses {
                    uses.current = *uses_remaining;
                }
            }
        }
        Effect::SpellSlotUsed { level, .. } => {
            if let Some(ref mut spellcasting) = world.player_character.spellcasting {
                spellcasting.spell_slots.use_slot(*level);
            }
        }
        // Effects that don't modify state (informational)
        Effect::DiceRolled { .. } => {}
        Effect::CheckSucceeded { .. } => {}
        Effect::CheckFailed { .. } => {}
        Effect::AttackHit { .. } => {}
        Effect::AttackMissed { .. } => {}
        Effect::InitiativeRolled { .. } => {}
        Effect::SneakAttackUsed { character_id, .. } => {
            // Mark that this character has used their sneak attack this turn
            if let Some(ref mut combat) = world.combat {
                combat.sneak_attack_used.insert(*character_id);
            }
        }
        // FactRemembered is handled by the DM agent's memory system, not world state
        Effect::FactRemembered { .. } => {}

        // Inventory effects
        Effect::ItemAdded {
            item_name,
            quantity,
            ..
        } => {
            // Try to look up item from standard database first
            let item = if let Some(standard_item) = crate::items::find_item(item_name) {
                let mut item = standard_item.as_item();
                item.quantity = *quantity;
                item
            } else {
                // Fall back to generic item
                Item {
                    name: item_name.clone(),
                    quantity: *quantity,
                    weight: 0.0,
                    value_gp: 0.0,
                    description: None,
                    item_type: ItemType::Other,
                    magical: false,
                }
            };
            world.player_character.inventory.add_item(item);
        }
        Effect::ItemRemoved {
            item_name,
            quantity,
            ..
        } => {
            world
                .player_character
                .inventory
                .remove_item(item_name, *quantity);
        }
        Effect::ItemEquipped { item_name, slot } => {
            // Look up item from database for proper stats, fall back to defaults
            match slot.as_str() {
                "armor" => {
                    if world
                        .player_character
                        .inventory
                        .find_item(item_name)
                        .is_some()
                    {
                        // Try to get proper armor stats from database
                        let armor = if let Some(db_armor) = crate::items::get_armor(item_name) {
                            db_armor
                        } else {
                            // Fall back to medium armor defaults
                            crate::world::ArmorItem::new(
                                item_name.clone(),
                                crate::world::ArmorType::Medium,
                                14,
                            )
                        };
                        world.player_character.equipment.armor = Some(armor);
                        world.player_character.inventory.remove_item(item_name, 1);
                    }
                }
                "shield" => {
                    if let Some(item) = world.player_character.inventory.find_item(item_name) {
                        let shield_item = item.clone();
                        world.player_character.equipment.shield = Some(shield_item);
                        world.player_character.inventory.remove_item(item_name, 1);
                    }
                }
                "main_hand" | "weapon" => {
                    if world
                        .player_character
                        .inventory
                        .find_item(item_name)
                        .is_some()
                    {
                        // Try to get proper weapon stats from database
                        let weapon = if let Some(db_weapon) = crate::items::get_weapon(item_name) {
                            db_weapon
                        } else {
                            // Fall back to generic 1d8 slashing
                            crate::world::WeaponItem::new(
                                item_name.clone(),
                                "1d8",
                                crate::world::WeaponDamageType::Slashing,
                            )
                        };
                        world.player_character.equipment.main_hand = Some(weapon);
                        world.player_character.inventory.remove_item(item_name, 1);
                    }
                }
                "off_hand" => {
                    if let Some(item) = world.player_character.inventory.find_item(item_name) {
                        let off_hand_item = item.clone();
                        world.player_character.equipment.off_hand = Some(off_hand_item);
                        world.player_character.inventory.remove_item(item_name, 1);
                    }
                }
                _ => {}
            }
        }
        Effect::ItemUnequipped { slot, .. } => match slot.as_str() {
            "armor" => {
                if let Some(armor) = world.player_character.equipment.armor.take() {
                    world.player_character.inventory.add_item(armor.base);
                }
            }
            "shield" => {
                if let Some(shield) = world.player_character.equipment.shield.take() {
                    world.player_character.inventory.add_item(shield);
                }
            }
            "main_hand" | "weapon" => {
                if let Some(weapon) = world.player_character.equipment.main_hand.take() {
                    world.player_character.inventory.add_item(weapon.base);
                }
            }
            "off_hand" => {
                if let Some(item) = world.player_character.equipment.off_hand.take() {
                    world.player_character.inventory.add_item(item);
                }
            }
            _ => {}
        },
        // ItemUsed is informational - the actual effects (healing, etc.) are separate effects
        Effect::ItemUsed { .. } => {}
        Effect::GoldChanged { new_total, .. } => {
            world.player_character.inventory.gold = *new_total;
        }
        Effect::SilverChanged { new_total, .. } => {
            world.player_character.inventory.silver = *new_total;
        }
        // AcChanged is informational - AC is recalculated from equipment
        Effect::AcChanged { .. } => {}

        Effect::DeathSaveFailure { failures, .. } => {
            for _ in 0..*failures {
                world.player_character.death_saves.add_failure();
            }
        }

        Effect::DeathSavesReset { .. } => {
            world.player_character.death_saves.reset();
        }

        Effect::CharacterDied { .. } => {
            // Character death is tracked via the effect itself
            // The UI/game can check for this effect and handle appropriately
            // For now, we don't modify world state further (could add a `dead: bool` flag)
        }

        Effect::DeathSaveSuccess {
            total_successes, ..
        } => {
            world.player_character.death_saves.successes = *total_successes;
        }

        Effect::Stabilized { .. } => {
            // Character is stable - still unconscious but no longer making death saves
            world.player_character.death_saves.reset();
            // Note: Character remains Unconscious until healed
        }

        Effect::ConcentrationBroken { .. } => {
            // Concentration tracking would be handled here if we had it
            // For now, this is informational for the UI/narrative
        }

        Effect::ConcentrationMaintained { .. } => {
            // Informational - concentration continues
        }
        Effect::LocationChanged { new_location, .. } => {
            world.current_location.name = new_location.clone();
        }
        Effect::ConsequenceRegistered { .. } => {
            // Consequence storage is handled by the DM agent in story_memory
            // This effect is informational for the rules layer
        }
        Effect::ConsequenceTriggered { .. } => {
            // Consequence triggering is handled by the relevance checker
            // This effect is informational for the UI/narrative
        }
        Effect::ClassResourceUsed { .. } => {
            // Class resource usage is tracked in ClassResources
            // The actual state changes are handled by the DM based on the effect
            // This effect is informational for the narrative/UI
        }
        Effect::RageStarted { damage_bonus, .. } => {
            world.player_character.class_resources.rage_active = true;
            world.player_character.class_resources.rage_damage_bonus = *damage_bonus;
            world.player_character.class_resources.rage_rounds_remaining = Some(10);
            // 1 minute = 10 rounds
        }
        Effect::RageEnded { .. } => {
            world.player_character.class_resources.rage_active = false;
            world.player_character.class_resources.rage_damage_bonus = 0;
            world.player_character.class_resources.rage_rounds_remaining = None;
        }

        // Quest effects
        Effect::QuestCreated {
            name,
            description,
            giver,
            objectives,
            rewards,
        } => {
            use crate::world::{Quest, QuestObjective};
            let mut quest = Quest::new(name.clone(), description.clone());
            quest.giver = giver.clone();
            quest.objectives = objectives
                .iter()
                .map(|(desc, optional)| QuestObjective {
                    description: desc.clone(),
                    completed: false,
                    optional: *optional,
                })
                .collect();
            quest.rewards = rewards.clone();
            world.quests.push(quest);
        }

        Effect::QuestObjectiveAdded {
            quest_name,
            objective,
            optional,
        } => {
            use crate::world::QuestObjective;
            if let Some(quest) = world.quests.iter_mut().find(|q| q.name == *quest_name) {
                quest.objectives.push(QuestObjective {
                    description: objective.clone(),
                    completed: false,
                    optional: *optional,
                });
            }
        }

        Effect::QuestObjectiveCompleted {
            quest_name,
            objective_description,
        } => {
            if let Some(quest) = world.quests.iter_mut().find(|q| q.name == *quest_name) {
                // Find objective by partial match
                if let Some(obj) = quest.objectives.iter_mut().find(|o| {
                    o.description
                        .to_lowercase()
                        .contains(&objective_description.to_lowercase())
                }) {
                    obj.completed = true;
                }
            }
        }

        Effect::QuestCompleted { quest_name, .. } => {
            use crate::world::QuestStatus;
            if let Some(quest) = world.quests.iter_mut().find(|q| q.name == *quest_name) {
                quest.status = QuestStatus::Completed;
                // Mark all non-optional objectives as complete
                for obj in &mut quest.objectives {
                    if !obj.optional {
                        obj.completed = true;
                    }
                }
            }
        }

        Effect::QuestFailed { quest_name, .. } => {
            use crate::world::QuestStatus;
            if let Some(quest) = world.quests.iter_mut().find(|q| q.name == *quest_name) {
                quest.status = QuestStatus::Failed;
            }
        }

        Effect::QuestUpdated {
            quest_name,
            new_description,
            add_rewards,
        } => {
            if let Some(quest) = world.quests.iter_mut().find(|q| q.name == *quest_name) {
                if let Some(desc) = new_description {
                    quest.description = desc.clone();
                }
                quest.rewards.extend(add_rewards.iter().cloned());
            }
        }

        // World Building effects
        Effect::NpcCreated { name, location } => {
            use crate::world::NPC;
            let mut npc = NPC::new(name.clone());

            // Set location if provided
            if let Some(loc_name) = location {
                // Find the location by name
                if let Some(loc) = world
                    .known_locations
                    .values()
                    .find(|l| l.name.eq_ignore_ascii_case(loc_name))
                {
                    npc.location_id = Some(loc.id);
                }
            }

            world.npcs.insert(npc.id, npc);
        }

        Effect::NpcUpdated { npc_name, .. } => {
            // The actual updates are passed through the Intent
            // This effect is informational for the narrative/UI
            // The real state changes would need the full update data
            // which is handled by the DM agent passing updated NPC data
            let _ = npc_name; // Suppress unused warning
        }

        Effect::NpcMoved {
            npc_name,
            to_location,
            ..
        } => {
            // Find NPC by name and update their location
            if let Some(npc) = world
                .npcs
                .values_mut()
                .find(|n| n.name.eq_ignore_ascii_case(npc_name))
            {
                // Find the destination location
                if let Some(loc) = world
                    .known_locations
                    .values()
                    .find(|l| l.name.eq_ignore_ascii_case(to_location))
                {
                    npc.location_id = Some(loc.id);
                } else {
                    // Location not found, clear location_id
                    npc.location_id = None;
                }
            }
        }

        Effect::NpcRemoved { npc_name, .. } => {
            // Remove NPC from the world
            let npc_id = world
                .npcs
                .values()
                .find(|n| n.name.eq_ignore_ascii_case(npc_name))
                .map(|n| n.id);

            if let Some(id) = npc_id {
                world.npcs.remove(&id);
            }
        }

        Effect::LocationCreated {
            name,
            location_type,
        } => {
            use crate::world::{Location, LocationType};

            let loc_type = match location_type.to_lowercase().as_str() {
                "wilderness" => LocationType::Wilderness,
                "town" => LocationType::Town,
                "city" => LocationType::City,
                "dungeon" => LocationType::Dungeon,
                "building" => LocationType::Building,
                "room" => LocationType::Room,
                "road" => LocationType::Road,
                "cave" => LocationType::Cave,
                _ => LocationType::Other,
            };

            let location = Location::new(name.clone(), loc_type);
            world.known_locations.insert(location.id, location);
        }

        Effect::LocationsConnected {
            from,
            to,
            direction,
        } => {
            use crate::world::LocationConnection;

            // Find the source and destination locations
            let from_id = world
                .known_locations
                .values()
                .find(|l| l.name.eq_ignore_ascii_case(from))
                .map(|l| l.id);

            let to_loc = world
                .known_locations
                .values()
                .find(|l| l.name.eq_ignore_ascii_case(to));

            if let (Some(from_id), Some(to_loc)) = (from_id, to_loc) {
                let connection = LocationConnection {
                    destination_id: to_loc.id,
                    destination_name: to_loc.name.clone(),
                    direction: direction.clone(),
                    travel_time_minutes: 0,
                };

                if let Some(from_loc) = world.known_locations.get_mut(&from_id) {
                    from_loc.connections.push(connection);
                }
            }
        }

        Effect::LocationUpdated { location_name, .. } => {
            // The actual updates are passed through the Intent
            // This effect is informational for the narrative/UI
            let _ = location_name; // Suppress unused warning
        }

        Effect::AbilityScoreModified {
            ability, modifier, ..
        } => {
            // Apply ability score modifier
            let score = match ability {
                Ability::Strength => &mut world.player_character.ability_scores.strength,
                Ability::Dexterity => &mut world.player_character.ability_scores.dexterity,
                Ability::Constitution => &mut world.player_character.ability_scores.constitution,
                Ability::Intelligence => &mut world.player_character.ability_scores.intelligence,
                Ability::Wisdom => &mut world.player_character.ability_scores.wisdom,
                Ability::Charisma => &mut world.player_character.ability_scores.charisma,
            };
            *score = (*score as i16 + *modifier as i16).clamp(1, 30) as u8;
        }

        Effect::SpellSlotRestored { level, .. } => {
            if let Some(ref mut spellcasting) = world.player_character.spellcasting {
                if *level >= 1 && *level <= 9 {
                    let slot_idx = (*level - 1) as usize;
                    if spellcasting.spell_slots.slots[slot_idx].used > 0 {
                        spellcasting.spell_slots.slots[slot_idx].used -= 1;
                    }
                }
            }
        }
        Effect::StateAsserted {
            entity_name,
            state_type,
            new_value,
            ..
        } => {
            // Find and update the NPC
            if let Some(npc) = world
                .npcs
                .values_mut()
                .find(|npc| npc.name.eq_ignore_ascii_case(entity_name))
            {
                match state_type {
                    StateType::Disposition => {
                        // Parse disposition string to enum
                        let new_disp = match new_value.to_lowercase().as_str() {
                            "hostile" => crate::world::Disposition::Hostile,
                            "unfriendly" => crate::world::Disposition::Unfriendly,
                            "neutral" => crate::world::Disposition::Neutral,
                            "friendly" => crate::world::Disposition::Friendly,
                            "helpful" => crate::world::Disposition::Helpful,
                            _ => return, // Invalid disposition, skip
                        };
                        npc.disposition = new_disp;
                    }
                    StateType::Location => {
                        // Find or create the location
                        // For now, just store as known information since location_id requires a proper Location
                        if !npc
                            .known_information
                            .iter()
                            .any(|i| i.contains(&format!("at {}", new_value)))
                        {
                            npc.known_information
                                .push(format!("Currently at {}", new_value));
                        }
                    }
                    StateType::Status => {
                        // NPC struct doesn't have a status field, store as known information
                        if !npc.known_information.iter().any(|i| i.contains("Status:")) {
                            npc.known_information.retain(|i| !i.starts_with("Status:"));
                        }
                        npc.known_information.push(format!("Status: {}", new_value));
                    }
                    StateType::Knowledge => {
                        // Add to known_information if not already present
                        if !npc.known_information.contains(new_value) {
                            npc.known_information.push(new_value.clone());
                        }
                    }
                    StateType::Relationship => {
                        // Relationships are tracked in story memory, not NPC struct directly
                        // The effect is recorded for story memory to process
                    }
                }
            }
        }
        Effect::KnowledgeShared {
            knowing_entity,
            content,
            ..
        } => {
            // Store in NPC known_information if the entity is an NPC
            if let Some(npc) = world
                .npcs
                .values_mut()
                .find(|npc| npc.name.eq_ignore_ascii_case(knowing_entity))
            {
                if !npc.known_information.contains(content) {
                    npc.known_information.push(content.clone());
                }
            }
            // Note: Full knowledge tracking (verification, source) is handled
            // in story_memory by the DM agent
        }

        // Scheduled event effects - these are primarily managed by StoryMemory,
        // not GameWorld. The actual scheduling is done by the DM agent.
        Effect::EventScheduled { .. }
        | Effect::EventCancelled { .. }
        | Effect::EventTriggered { .. } => {
            // No GameWorld state changes needed - StoryMemory handles these
        }
    }
}
