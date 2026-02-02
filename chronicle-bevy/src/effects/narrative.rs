//! Formats game effects into narrative text for display.
//!
//! This module converts game effects into human-readable narrative strings.

use chronicle_core::rules::Effect;
use chronicle_core::world::NarrativeType;

/// Represents narrative output from an effect.
pub struct NarrativeOutput {
    /// The narrative text to display.
    pub text: String,
    /// The type of narrative (combat, system, etc.).
    pub narrative_type: NarrativeType,
    /// Optional status message to display.
    pub status: Option<String>,
}

/// Generates narrative output for a game effect.
///
/// Returns `None` for internal effects that should not produce visible narrative.
pub fn narrative_for_effect(effect: &Effect) -> Option<NarrativeOutput> {
    match effect {
        Effect::DiceRolled { roll, purpose } => Some(NarrativeOutput {
            text: format!("{}: {} = {}", purpose, roll.expression, roll.total),
            narrative_type: NarrativeType::System,
            status: None,
        }),

        Effect::AttackHit {
            attacker_name,
            target_name,
            attack_roll,
            target_ac,
            is_critical,
        } => {
            if *is_critical {
                Some(NarrativeOutput {
                    text: format!(
                        "CRITICAL HIT! {attacker_name} rolls {attack_roll} vs AC {target_ac} and strikes {target_name}!"
                    ),
                    narrative_type: NarrativeType::Combat,
                    status: None,
                })
            } else {
                Some(NarrativeOutput {
                    text: format!(
                        "{attacker_name} rolls {attack_roll} vs AC {target_ac} and hits {target_name}!"
                    ),
                    narrative_type: NarrativeType::Combat,
                    status: None,
                })
            }
        }

        Effect::AttackMissed {
            attacker_name,
            target_name,
            attack_roll,
            target_ac,
        } => Some(NarrativeOutput {
            text: format!(
                "{attacker_name} rolls {attack_roll} vs AC {target_ac} and misses {target_name}!"
            ),
            narrative_type: NarrativeType::Combat,
            status: None,
        }),

        Effect::HpChanged {
            amount,
            new_current,
            dropped_to_zero,
            ..
        } => {
            let text = if *amount < 0 {
                format!("Takes {} damage! (HP: {})", -amount, new_current)
            } else if *amount > 0 {
                format!("Heals {amount} HP! (HP: {new_current})")
            } else {
                return None;
            };

            let narrative_type = if *amount < 0 {
                NarrativeType::Combat
            } else {
                NarrativeType::System
            };

            let status = if *dropped_to_zero {
                Some("You fall unconscious!".to_string())
            } else {
                None
            };

            Some(NarrativeOutput {
                text,
                narrative_type,
                status,
            })
        }

        Effect::ConditionApplied {
            condition, source, ..
        } => Some(NarrativeOutput {
            text: format!("Now {condition} from {source}!"),
            narrative_type: NarrativeType::Combat,
            status: None,
        }),

        Effect::ConditionRemoved { condition, .. } => Some(NarrativeOutput {
            text: format!("No longer {condition}."),
            narrative_type: NarrativeType::System,
            status: None,
        }),

        Effect::CombatStarted => Some(NarrativeOutput {
            text: "Combat begins!".to_string(),
            narrative_type: NarrativeType::Combat,
            status: Some("Roll for initiative!".to_string()),
        }),

        Effect::CombatEnded => Some(NarrativeOutput {
            text: "Combat ends.".to_string(),
            narrative_type: NarrativeType::System,
            status: None,
        }),

        Effect::TurnAdvanced {
            round,
            current_combatant,
        } => Some(NarrativeOutput {
            text: format!("Round {round} - {current_combatant}'s turn."),
            narrative_type: NarrativeType::Combat,
            status: None,
        }),

        Effect::InitiativeRolled {
            name, roll, total, ..
        } => Some(NarrativeOutput {
            text: format!("{name} rolls {roll} for initiative (total: {total})"),
            narrative_type: NarrativeType::System,
            status: None,
        }),

        Effect::CombatantAdded {
            name, initiative, ..
        } => Some(NarrativeOutput {
            text: format!("{name} enters combat with initiative {initiative}."),
            narrative_type: NarrativeType::Combat,
            status: None,
        }),

        Effect::TimeAdvanced { minutes } => {
            let text = if *minutes >= 60 {
                let hours = minutes / 60;
                let mins = minutes % 60;
                if mins > 0 {
                    format!("{hours} hours and {mins} minutes pass.")
                } else {
                    format!("{hours} hours pass.")
                }
            } else {
                format!("{minutes} minutes pass.")
            };
            Some(NarrativeOutput {
                text,
                narrative_type: NarrativeType::System,
                status: None,
            })
        }

        Effect::ExperienceGained { amount, new_total } => Some(NarrativeOutput {
            text: format!("Gained {amount} XP! (Total: {new_total} XP)"),
            narrative_type: NarrativeType::System,
            status: None,
        }),

        Effect::LevelUp { new_level } => Some(NarrativeOutput {
            text: format!("LEVEL UP! You are now level {new_level}!"),
            narrative_type: NarrativeType::System,
            status: Some(format!("Level up! Now level {new_level}!")),
        }),

        Effect::FeatureUsed {
            feature_name,
            uses_remaining,
        } => Some(NarrativeOutput {
            text: format!("Used {feature_name}. ({uses_remaining} uses remaining)"),
            narrative_type: NarrativeType::System,
            status: None,
        }),

        Effect::SpellSlotUsed { level, remaining } => Some(NarrativeOutput {
            text: format!("Used a level {level} spell slot. ({remaining} remaining)"),
            narrative_type: NarrativeType::System,
            status: None,
        }),

        Effect::RestCompleted { rest_type } => {
            let rest_name = match rest_type {
                chronicle_core::rules::RestType::Short => "short",
                chronicle_core::rules::RestType::Long => "long",
            };
            Some(NarrativeOutput {
                text: format!("Completed a {rest_name} rest."),
                narrative_type: NarrativeType::System,
                status: None,
            })
        }

        Effect::CheckSucceeded {
            check_type,
            roll,
            dc,
        } => Some(NarrativeOutput {
            text: format!("{check_type} check succeeded! ({roll} vs DC {dc})"),
            narrative_type: NarrativeType::System,
            status: None,
        }),

        Effect::CheckFailed {
            check_type,
            roll,
            dc,
        } => Some(NarrativeOutput {
            text: format!("{check_type} check failed. ({roll} vs DC {dc})"),
            narrative_type: NarrativeType::System,
            status: None,
        }),

        Effect::FactRemembered { .. } | Effect::ConsequenceRegistered { .. } => {
            // Internal effects - no UI output
            None
        }

        Effect::ConsequenceTriggered {
            consequence_description,
            ..
        } => Some(NarrativeOutput {
            text: format!("CONSEQUENCE: {consequence_description}"),
            narrative_type: NarrativeType::System,
            status: None,
        }),

        Effect::ItemAdded {
            item_name,
            quantity,
            new_total,
        } => {
            let qty_str = if *quantity > 1 {
                format!("{quantity} x ")
            } else {
                String::new()
            };
            Some(NarrativeOutput {
                text: format!("Received {qty_str}{item_name}! (now have {new_total})"),
                narrative_type: NarrativeType::System,
                status: None,
            })
        }

        Effect::ItemRemoved {
            item_name,
            quantity,
            remaining,
        } => {
            let qty_str = if *quantity > 1 {
                format!("{quantity} x ")
            } else {
                String::new()
            };
            let text = if *remaining > 0 {
                format!("Lost {qty_str}{item_name}. ({remaining} remaining)")
            } else {
                format!("Lost {qty_str}{item_name}.")
            };
            Some(NarrativeOutput {
                text,
                narrative_type: NarrativeType::System,
                status: None,
            })
        }

        Effect::ItemEquipped { item_name, slot } => Some(NarrativeOutput {
            text: format!("Equipped {item_name} in {slot} slot."),
            narrative_type: NarrativeType::System,
            status: None,
        }),

        Effect::ItemUnequipped { item_name, slot } => Some(NarrativeOutput {
            text: format!("Unequipped {item_name} from {slot} slot."),
            narrative_type: NarrativeType::System,
            status: None,
        }),

        Effect::ItemUsed { item_name, result } => Some(NarrativeOutput {
            text: format!("Used {item_name}. {result}"),
            narrative_type: NarrativeType::System,
            status: None,
        }),

        Effect::GoldChanged {
            amount,
            new_total,
            reason,
        } => {
            let action = if *amount >= 0 { "Gained" } else { "Spent" };
            Some(NarrativeOutput {
                text: format!(
                    "{} {} gp ({}). Total: {} gp",
                    action,
                    amount.abs(),
                    reason,
                    new_total
                ),
                narrative_type: NarrativeType::System,
                status: None,
            })
        }

        Effect::SilverChanged {
            amount,
            new_total,
            reason,
        } => {
            let action = if *amount >= 0 { "Gained" } else { "Spent" };
            Some(NarrativeOutput {
                text: format!(
                    "{} {} sp ({}). Total: {} sp",
                    action,
                    amount.abs(),
                    reason,
                    new_total
                ),
                narrative_type: NarrativeType::System,
                status: None,
            })
        }

        Effect::AcChanged { new_ac, source } => Some(NarrativeOutput {
            text: format!("AC changed to {new_ac} ({source})"),
            narrative_type: NarrativeType::System,
            status: None,
        }),

        Effect::DeathSaveFailure {
            total_failures,
            source,
            ..
        } => {
            let status = if *total_failures >= 3 {
                Some("You have died!".to_string())
            } else {
                Some(format!("Death saves: {total_failures}/3 failures"))
            };
            Some(NarrativeOutput {
                text: format!("DEATH SAVE FAILURE from {source}! ({total_failures}/3 failures)"),
                narrative_type: NarrativeType::Combat,
                status,
            })
        }

        Effect::DeathSavesReset { .. } => Some(NarrativeOutput {
            text: "Death saves reset - you're stable!".to_string(),
            narrative_type: NarrativeType::System,
            status: None,
        }),

        Effect::CharacterDied { cause, .. } => Some(NarrativeOutput {
            text: format!("YOU HAVE DIED! Cause: {cause}"),
            narrative_type: NarrativeType::Combat,
            status: Some("GAME OVER - Your character has died.".to_string()),
        }),

        Effect::DeathSaveSuccess {
            roll,
            total_successes,
            ..
        } => Some(NarrativeOutput {
            text: format!("Death save SUCCESS! Rolled {roll} ({total_successes}/3 successes)"),
            narrative_type: NarrativeType::Combat,
            status: Some(format!("Death saves: {total_successes}/3 successes")),
        }),

        Effect::Stabilized { .. } => Some(NarrativeOutput {
            text: "You have stabilized! No longer dying.".to_string(),
            narrative_type: NarrativeType::Combat,
            status: Some("Stabilized - unconscious but stable".to_string()),
        }),

        Effect::ConcentrationBroken {
            spell_name,
            damage_taken,
            roll,
            dc,
            ..
        } => Some(NarrativeOutput {
            text: format!(
                "CONCENTRATION BROKEN! Took {damage_taken} damage, rolled {roll} vs DC {dc} - {spell_name} ends!"
            ),
            narrative_type: NarrativeType::Combat,
            status: Some(format!("Lost concentration on {spell_name}!")),
        }),

        Effect::ConcentrationMaintained {
            spell_name,
            roll,
            dc,
            ..
        } => Some(NarrativeOutput {
            text: format!(
                "Concentration maintained! Rolled {roll} vs DC {dc} - {spell_name} continues."
            ),
            narrative_type: NarrativeType::System,
            status: None,
        }),

        Effect::LocationChanged {
            previous_location,
            new_location,
        } => Some(NarrativeOutput {
            text: format!("You travel from {previous_location} to {new_location}."),
            narrative_type: NarrativeType::System,
            status: Some(format!("Now at: {new_location}")),
        }),

        Effect::ClassResourceUsed {
            character_name,
            resource_name,
            description,
        } => Some(NarrativeOutput {
            text: format!("{character_name} uses {resource_name}: {description}"),
            narrative_type: NarrativeType::System,
            status: None,
        }),

        Effect::RageStarted { damage_bonus, .. } => Some(NarrativeOutput {
            text: format!(
                "RAGE! +{damage_bonus} damage to melee attacks, resistance to physical damage"
            ),
            narrative_type: NarrativeType::System,
            status: Some("Raging!".to_string()),
        }),

        Effect::RageEnded { reason, .. } => Some(NarrativeOutput {
            text: format!("Rage ended: {reason}"),
            narrative_type: NarrativeType::System,
            status: None,
        }),

        // Quest effects
        Effect::QuestCreated { name, giver, .. } => {
            let giver_text = giver
                .as_ref()
                .map(|g| format!(" from {g}"))
                .unwrap_or_default();
            Some(NarrativeOutput {
                text: format!("NEW QUEST{giver_text}: {name}"),
                narrative_type: NarrativeType::System,
                status: Some(format!("Quest started: {name}")),
            })
        }

        Effect::QuestObjectiveAdded {
            quest_name,
            objective,
            ..
        } => Some(NarrativeOutput {
            text: format!("New objective for \"{quest_name}\": {objective}"),
            narrative_type: NarrativeType::System,
            status: None,
        }),

        Effect::QuestObjectiveCompleted {
            quest_name,
            objective_description,
        } => Some(NarrativeOutput {
            text: format!("Objective completed: {objective_description}"),
            narrative_type: NarrativeType::System,
            status: Some(format!("Progress on \"{quest_name}\"")),
        }),

        Effect::QuestCompleted { quest_name, .. } => Some(NarrativeOutput {
            text: format!("QUEST COMPLETED: {quest_name}!"),
            narrative_type: NarrativeType::System,
            status: Some(format!("Completed: {quest_name}")),
        }),

        Effect::QuestFailed {
            quest_name,
            failure_reason,
        } => Some(NarrativeOutput {
            text: format!("QUEST FAILED: {quest_name} - {failure_reason}"),
            narrative_type: NarrativeType::System,
            status: Some(format!("Failed: {quest_name}")),
        }),

        Effect::QuestUpdated { quest_name, .. } => Some(NarrativeOutput {
            text: format!("Quest \"{quest_name}\" updated."),
            narrative_type: NarrativeType::System,
            status: None,
        }),

        Effect::SneakAttackUsed { damage_dice, .. } => Some(NarrativeOutput {
            text: format!("SNEAK ATTACK! ({damage_dice}d6 extra damage)"),
            narrative_type: NarrativeType::Combat,
            status: None,
        }),

        // World-building effects
        Effect::NpcCreated { name, location } => {
            let loc_text = location
                .as_ref()
                .map(|l| format!(" at {l}"))
                .unwrap_or_default();
            Some(NarrativeOutput {
                text: format!("{name} enters the story{loc_text}."),
                narrative_type: NarrativeType::System,
                status: None,
            })
        }

        Effect::NpcUpdated { npc_name, changes } => Some(NarrativeOutput {
            text: format!("{npc_name}: {changes}"),
            narrative_type: NarrativeType::System,
            status: None,
        }),

        Effect::NpcMoved {
            npc_name,
            to_location,
            ..
        } => Some(NarrativeOutput {
            text: format!("{npc_name} moves to {to_location}."),
            narrative_type: NarrativeType::System,
            status: None,
        }),

        Effect::NpcRemoved { npc_name, reason } => Some(NarrativeOutput {
            text: format!("{npc_name} leaves the story: {reason}"),
            narrative_type: NarrativeType::System,
            status: None,
        }),

        Effect::LocationCreated {
            name,
            location_type,
        } => Some(NarrativeOutput {
            text: format!("Discovered new {location_type}: {name}"),
            narrative_type: NarrativeType::System,
            status: Some(format!("New location: {name}")),
        }),

        Effect::LocationsConnected {
            from,
            to,
            direction,
        } => {
            let dir_text = direction
                .as_ref()
                .map(|d| format!(" ({d})"))
                .unwrap_or_default();
            Some(NarrativeOutput {
                text: format!("Path discovered: {from} ↔ {to}{dir_text}"),
                narrative_type: NarrativeType::System,
                status: None,
            })
        }

        Effect::LocationUpdated {
            location_name,
            changes,
        } => Some(NarrativeOutput {
            text: format!("{location_name}: {changes}"),
            narrative_type: NarrativeType::System,
            status: None,
        }),

        Effect::AbilityScoreModified {
            ability,
            modifier,
            source,
        } => {
            let sign = if *modifier >= 0 { "+" } else { "" };
            Some(NarrativeOutput {
                text: format!("{ability:?} {sign}{modifier} ({source})"),
                narrative_type: NarrativeType::System,
                status: Some(format!("{ability:?} modified!")),
            })
        }

        Effect::SpellSlotRestored {
            level,
            new_remaining,
        } => Some(NarrativeOutput {
            text: format!("Level {level} spell slot restored! ({new_remaining} available)"),
            narrative_type: NarrativeType::System,
            status: None,
        }),

        Effect::StateAsserted {
            entity_name,
            state_type,
            new_value,
            reason,
            ..
        } => Some(NarrativeOutput {
            text: format!(
                "[{} {} is now {} - {}]",
                entity_name,
                state_type.name(),
                new_value,
                reason
            ),
            narrative_type: NarrativeType::System,
            status: None,
        }),

        Effect::KnowledgeShared {
            knowing_entity,
            content,
            source,
            verification,
            ..
        } => {
            let verification_note = match verification.as_str() {
                "true" => " (verified)",
                "false" => " (false information)",
                "partial" => " (partially true)",
                "outdated" => " (outdated)",
                _ => "",
            };
            Some(NarrativeOutput {
                text: format!(
                    "[{} learns from {}: \"{}\"{}]",
                    knowing_entity, source, content, verification_note
                ),
                narrative_type: NarrativeType::System,
                status: None,
            })
        }

        // Scheduled event effects
        Effect::EventScheduled {
            description,
            trigger_description,
            location,
            ..
        } => {
            let loc_text = location
                .as_ref()
                .map(|l| format!(" at {}", l))
                .unwrap_or_default();
            Some(NarrativeOutput {
                text: format!(
                    "[Event scheduled: \"{}\" {}{}]",
                    description, trigger_description, loc_text
                ),
                narrative_type: NarrativeType::System,
                status: None,
            })
        }

        Effect::EventCancelled {
            description,
            reason,
        } => Some(NarrativeOutput {
            text: format!("[Event cancelled: \"{}\" - {}]", description, reason),
            narrative_type: NarrativeType::System,
            status: None,
        }),

        Effect::EventTriggered {
            description,
            location,
        } => {
            let loc_text = location
                .as_ref()
                .map(|l| format!(" at {}", l))
                .unwrap_or_default();
            Some(NarrativeOutput {
                text: format!("[EVENT: {}{}]", description, loc_text),
                narrative_type: NarrativeType::System,
                status: Some(description.clone()),
            })
        }
    }
}
