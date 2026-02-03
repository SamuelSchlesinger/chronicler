//! Class feature resolution methods.

use crate::rules::helpers::roll_with_fallback;
use crate::rules::types::{Effect, Resolution};
use crate::rules::RulesEngine;
use crate::world::{CharacterClass, CharacterId, GameWorld};

impl RulesEngine {
    pub(crate) fn resolve_use_rage(
        &self,
        world: &GameWorld,
        _character_id: CharacterId,
    ) -> Resolution {
        let character = &world.player_character;

        // Check if already raging
        if world.player_character.class_resources.rage_active {
            return Resolution::new(format!("{} is already raging!", character.name));
        }

        // Check for rage uses remaining
        let rage_feature = character.features.iter().find(|f| f.name == "Rage");
        if let Some(feature) = rage_feature {
            if let Some(ref uses) = feature.uses {
                if uses.current == 0 {
                    return Resolution::new(format!(
                        "{} has no rage uses remaining! (Recovers on long rest)",
                        character.name
                    ));
                }
            }
        }

        // Determine rage damage bonus based on level
        let barbarian_level = character
            .classes
            .iter()
            .find(|c| c.class == CharacterClass::Barbarian)
            .map(|c| c.level)
            .unwrap_or(1);

        let rage_damage = match barbarian_level {
            1..=8 => 2,
            9..=15 => 3,
            _ => 4,
        };

        Resolution::new(format!(
            "{} enters a RAGE! Gains: advantage on STR checks/saves, +{} rage damage to melee attacks, resistance to bludgeoning/piercing/slashing damage. Cannot cast spells or concentrate while raging.",
            character.name, rage_damage
        ))
        .with_effect(Effect::RageStarted {
            character_id: world.player_character.id,
            damage_bonus: rage_damage,
        })
        .with_effect(Effect::ClassResourceUsed {
            character_name: character.name.clone(),
            resource_name: "Rage".to_string(),
            description: format!("Entered rage (1 minute, +{rage_damage} damage)"),
        })
        .with_effect(Effect::FeatureUsed {
            feature_name: "Rage".to_string(),
            uses_remaining: 0,
        })
    }

    pub(crate) fn resolve_end_rage(
        &self,
        world: &GameWorld,
        _character_id: CharacterId,
        reason: &str,
    ) -> Resolution {
        let character = &world.player_character;

        if !world.player_character.class_resources.rage_active {
            return Resolution::new(format!("{} is not currently raging.", character.name));
        }

        let reason_text = match reason {
            "duration_expired" => "Rage ended (1 minute duration expired).",
            "unconscious" => "Rage ended (knocked unconscious).",
            "no_combat_action" => "Rage ended (turn ended without attacking or taking damage).",
            "voluntary" => "Rage ended voluntarily.",
            _ => "Rage ended.",
        };

        Resolution::new(format!("{}'s rage ends. {}", character.name, reason_text))
            .with_effect(Effect::RageEnded {
                character_id: world.player_character.id,
                reason: reason_text.to_string(),
            })
            .with_effect(Effect::ClassResourceUsed {
                character_name: character.name.clone(),
                resource_name: "Rage".to_string(),
                description: reason_text.to_string(),
            })
    }

    pub(crate) fn resolve_use_ki(
        &self,
        world: &GameWorld,
        _character_id: CharacterId,
        points: u8,
        ability: &str,
    ) -> Resolution {
        let character = &world.player_character;
        let resources = &world.player_character.class_resources;

        if resources.ki_points < points {
            return Resolution::new(format!(
                "{} doesn't have enough ki points! Has {} but needs {}.",
                character.name, resources.ki_points, points
            ));
        }

        let ability_description = match ability {
            "flurry_of_blows" => "Flurry of Blows: Make two unarmed strikes as a bonus action.",
            "patient_defense" => "Patient Defense: Take the Dodge action as a bonus action.",
            "step_of_the_wind" => {
                "Step of the Wind: Disengage or Dash as a bonus action, jump distance doubled."
            }
            "stunning_strike" => "Stunning Strike: Target must make a CON save or be Stunned until the end of your next turn.",
            _ => ability,
        };

        Resolution::new(format!(
            "{} spends {} ki point{}. {}",
            character.name,
            points,
            if points == 1 { "" } else { "s" },
            ability_description
        ))
        .with_effect(Effect::ClassResourceUsed {
            character_name: character.name.clone(),
            resource_name: "Ki Points".to_string(),
            description: format!("Spent {points} ki for {ability}"),
        })
    }

    pub(crate) fn resolve_use_lay_on_hands(
        &self,
        world: &GameWorld,
        _character_id: CharacterId,
        target_name: &str,
        hp_amount: u32,
        cure_disease: bool,
        neutralize_poison: bool,
    ) -> Resolution {
        let character = &world.player_character;
        let pool = world.player_character.class_resources.lay_on_hands_pool;

        let total_cost =
            hp_amount + if cure_disease { 5 } else { 0 } + if neutralize_poison { 5 } else { 0 };

        if pool < total_cost {
            return Resolution::new(format!(
                "{} doesn't have enough in their Lay on Hands pool! Has {} HP but needs {}.",
                character.name, pool, total_cost
            ));
        }

        let mut effects_text = Vec::new();
        if hp_amount > 0 {
            effects_text.push(format!("restores {hp_amount} HP"));
        }
        if cure_disease {
            effects_text.push("cures one disease".to_string());
        }
        if neutralize_poison {
            effects_text.push("neutralizes one poison".to_string());
        }

        Resolution::new(format!(
            "{} uses Lay on Hands on {}: {}. ({} HP remaining in pool)",
            character.name,
            target_name,
            effects_text.join(", "),
            pool - total_cost
        ))
        .with_effect(Effect::ClassResourceUsed {
            character_name: character.name.clone(),
            resource_name: "Lay on Hands".to_string(),
            description: format!("Used {total_cost} points on {target_name}"),
        })
    }

    pub(crate) fn resolve_use_divine_smite(
        &self,
        world: &GameWorld,
        _character_id: CharacterId,
        spell_slot_level: u8,
        target_is_undead_or_fiend: bool,
    ) -> Resolution {
        let character = &world.player_character;

        // Check if they have spell slots available
        if let Some(ref spellcasting) = character.spellcasting {
            let slot_idx = spell_slot_level.saturating_sub(1) as usize;
            if slot_idx < 9 {
                let slot = &spellcasting.spell_slots.slots[slot_idx];
                if slot.available() == 0 {
                    return Resolution::new(format!(
                        "{} has no level {} spell slots remaining!",
                        character.name, spell_slot_level
                    ));
                }
            }
        }

        // Calculate damage dice
        // Base: 2d8, +1d8 per slot level above 1st, max 5d8
        // Extra 1d8 vs undead/fiends
        let base_dice = 2 + (spell_slot_level.saturating_sub(1)).min(3);
        let total_dice = if target_is_undead_or_fiend {
            (base_dice + 1).min(6)
        } else {
            base_dice.min(5)
        };

        let damage_roll = roll_with_fallback(&format!("{total_dice}d8"), "2d8");

        let extra_text = if target_is_undead_or_fiend {
            " (extra damage vs undead/fiend)"
        } else {
            ""
        };

        Resolution::new(format!(
            "{} channels divine power into their strike! Divine Smite deals {}d8 = {} radiant damage{}. (Level {} slot expended)",
            character.name, total_dice, damage_roll.total, extra_text, spell_slot_level
        ))
        .with_effect(Effect::DiceRolled {
            roll: damage_roll,
            purpose: "Divine Smite damage".to_string(),
        })
        .with_effect(Effect::ClassResourceUsed {
            character_name: character.name.clone(),
            resource_name: "Divine Smite".to_string(),
            description: format!("Used level {spell_slot_level} slot for smite"),
        })
    }

    pub(crate) fn resolve_use_wild_shape(
        &self,
        world: &GameWorld,
        _character_id: CharacterId,
        beast_form: &str,
        beast_hp: i32,
        _beast_ac: Option<u8>,
    ) -> Resolution {
        let character = &world.player_character;

        // Check if already in Wild Shape
        if world
            .player_character
            .class_resources
            .wild_shape_form
            .is_some()
        {
            return Resolution::new(format!("{} is already in Wild Shape form!", character.name));
        }

        // Find Wild Shape feature uses
        let wild_shape_feature = character.features.iter().find(|f| f.name == "Wild Shape");
        if let Some(feature) = wild_shape_feature {
            if let Some(ref uses) = feature.uses {
                if uses.current == 0 {
                    return Resolution::new(format!(
                        "{} has no Wild Shape uses remaining! (Recovers on short/long rest)",
                        character.name
                    ));
                }
            }
        }

        // Calculate duration based on Druid level
        let druid_level = character
            .classes
            .iter()
            .find(|c| c.class == CharacterClass::Druid)
            .map(|c| c.level)
            .unwrap_or(2);
        let duration_hours = druid_level / 2;

        Resolution::new(format!(
            "{} transforms into a {}! Beast form has {} HP. Duration: {} hour{}. Mental stats, proficiencies, and features retained. Cannot cast spells but can maintain concentration.",
            character.name, beast_form, beast_hp, duration_hours,
            if duration_hours == 1 { "" } else { "s" }
        ))
        .with_effect(Effect::ClassResourceUsed {
            character_name: character.name.clone(),
            resource_name: "Wild Shape".to_string(),
            description: format!("Transformed into {beast_form} ({beast_hp} HP)"),
        })
        .with_effect(Effect::FeatureUsed {
            feature_name: "Wild Shape".to_string(),
            uses_remaining: 0,
        })
    }

    pub(crate) fn resolve_end_wild_shape(
        &self,
        world: &GameWorld,
        _character_id: CharacterId,
        reason: &str,
        excess_damage: i32,
    ) -> Resolution {
        let character = &world.player_character;

        if world
            .player_character
            .class_resources
            .wild_shape_form
            .is_none()
        {
            return Resolution::new(format!(
                "{} is not currently in Wild Shape form.",
                character.name
            ));
        }

        let reason_text = match reason {
            "duration_expired" => "Wild Shape ended (duration expired).",
            "hp_zero" => {
                if excess_damage > 0 {
                    &format!(
                        "Wild Shape ended (beast HP dropped to 0). {} excess damage carries over to normal form!",
                        excess_damage
                    )
                } else {
                    "Wild Shape ended (beast HP dropped to 0)."
                }
            }
            "voluntary" => "Wild Shape ended voluntarily as a bonus action.",
            "incapacitated" => "Wild Shape ended (druid became incapacitated).",
            _ => "Wild Shape ended.",
        };

        let mut resolution = Resolution::new(format!(
            "{} reverts to their normal form. {}",
            character.name, reason_text
        ))
        .with_effect(Effect::ClassResourceUsed {
            character_name: character.name.clone(),
            resource_name: "Wild Shape".to_string(),
            description: reason_text.to_string(),
        });

        // Apply excess damage if any
        if excess_damage > 0 {
            resolution = resolution.with_effect(Effect::HpChanged {
                target_id: world.player_character.id,
                amount: -excess_damage,
                new_current: (character.hit_points.current - excess_damage).max(0),
                new_max: character.hit_points.maximum,
                dropped_to_zero: character.hit_points.current - excess_damage <= 0,
            });
        }

        resolution
    }

    pub(crate) fn resolve_use_channel_divinity(
        &self,
        world: &GameWorld,
        _character_id: CharacterId,
        option: &str,
        targets: &[String],
    ) -> Resolution {
        let character = &world.player_character;

        // Check for Channel Divinity uses
        let cd_feature = character
            .features
            .iter()
            .find(|f| f.name == "Channel Divinity");
        if let Some(feature) = cd_feature {
            if let Some(ref uses) = feature.uses {
                if uses.current == 0 {
                    return Resolution::new(format!(
                        "{} has no Channel Divinity uses remaining! (Recovers on short/long rest)",
                        character.name
                    ));
                }
            }
        }

        let option_description = match option.to_lowercase().as_str() {
            "turn undead" => {
                "Turn Undead: Each undead within 30 feet must make a WIS save. On failure, they must spend their turns moving away and cannot take reactions for 1 minute."
            }
            "divine spark" => {
                "Divine Spark: Either deal 1d8 radiant damage to one creature within 30 feet (DEX save for half), or restore 1d8 HP to one creature within 30 feet."
            }
            "sacred weapon" => {
                "Sacred Weapon: Your weapon becomes magical for 1 minute, +CHA to attack rolls, and sheds bright light."
            }
            _ => option,
        };

        let targets_text = if targets.is_empty() {
            String::new()
        } else {
            format!(" Targets: {}.", targets.join(", "))
        };

        Resolution::new(format!(
            "{} uses Channel Divinity: {}.{}",
            character.name, option_description, targets_text
        ))
        .with_effect(Effect::ClassResourceUsed {
            character_name: character.name.clone(),
            resource_name: "Channel Divinity".to_string(),
            description: option.to_string(),
        })
        .with_effect(Effect::FeatureUsed {
            feature_name: "Channel Divinity".to_string(),
            uses_remaining: 0,
        })
    }

    pub(crate) fn resolve_use_bardic_inspiration(
        &self,
        world: &GameWorld,
        _character_id: CharacterId,
        target_name: &str,
        die_size: &str,
    ) -> Resolution {
        let character = &world.player_character;

        // Check for Bardic Inspiration uses
        let bi_feature = character
            .features
            .iter()
            .find(|f| f.name == "Bardic Inspiration");
        if let Some(feature) = bi_feature {
            if let Some(ref uses) = feature.uses {
                if uses.current == 0 {
                    return Resolution::new(format!(
                        "{} has no Bardic Inspiration uses remaining! (Recovers on long rest, or short rest at level 5+)",
                        character.name
                    ));
                }
            }
        }

        Resolution::new(format!(
            "{} inspires {} with a rousing performance! {} gains a {} Bardic Inspiration die they can add to one ability check, attack roll, or saving throw within the next 10 minutes.",
            character.name, target_name, target_name, die_size
        ))
        .with_effect(Effect::ClassResourceUsed {
            character_name: character.name.clone(),
            resource_name: "Bardic Inspiration".to_string(),
            description: format!("Inspired {target_name} with a {die_size}"),
        })
        .with_effect(Effect::FeatureUsed {
            feature_name: "Bardic Inspiration".to_string(),
            uses_remaining: 0,
        })
    }

    pub(crate) fn resolve_use_action_surge(
        &self,
        world: &GameWorld,
        _character_id: CharacterId,
        action_taken: &str,
    ) -> Resolution {
        let character = &world.player_character;

        if world.player_character.class_resources.action_surge_used {
            return Resolution::new(format!(
                "{} has already used Action Surge! (Recovers on short/long rest)",
                character.name
            ));
        }

        Resolution::new(format!(
            "{} surges with renewed vigor! Takes an additional action this turn: {}",
            character.name, action_taken
        ))
        .with_effect(Effect::ClassResourceUsed {
            character_name: character.name.clone(),
            resource_name: "Action Surge".to_string(),
            description: action_taken.to_string(),
        })
        .with_effect(Effect::FeatureUsed {
            feature_name: "Action Surge".to_string(),
            uses_remaining: 0,
        })
    }

    pub(crate) fn resolve_use_second_wind(
        &self,
        world: &GameWorld,
        _character_id: CharacterId,
    ) -> Resolution {
        let character = &world.player_character;

        if world.player_character.class_resources.second_wind_used {
            return Resolution::new(format!(
                "{} has already used Second Wind! (Recovers on short/long rest)",
                character.name
            ));
        }

        // Calculate healing: 1d10 + fighter level
        let fighter_level = character
            .classes
            .iter()
            .find(|c| c.class == CharacterClass::Fighter)
            .map(|c| c.level)
            .unwrap_or(1);

        let healing_roll = roll_with_fallback(&format!("1d10+{fighter_level}"), "1d10+1");
        let healing = healing_roll.total;

        let new_hp = (character.hit_points.current + healing).min(character.hit_points.maximum);

        Resolution::new(format!(
            "{} catches their breath with Second Wind! Regains 1d10+{} = {} HP. (Now at {}/{})",
            character.name, fighter_level, healing, new_hp, character.hit_points.maximum
        ))
        .with_effect(Effect::DiceRolled {
            roll: healing_roll,
            purpose: "Second Wind healing".to_string(),
        })
        .with_effect(Effect::HpChanged {
            target_id: world.player_character.id,
            amount: healing,
            new_current: new_hp,
            new_max: character.hit_points.maximum,
            dropped_to_zero: false,
        })
        .with_effect(Effect::ClassResourceUsed {
            character_name: character.name.clone(),
            resource_name: "Second Wind".to_string(),
            description: format!("Healed {healing} HP"),
        })
        .with_effect(Effect::FeatureUsed {
            feature_name: "Second Wind".to_string(),
            uses_remaining: 0,
        })
    }

    pub(crate) fn resolve_use_sorcery_points(
        &self,
        world: &GameWorld,
        _character_id: CharacterId,
        points: u8,
        metamagic: &str,
        spell_name: Option<&str>,
        slot_level: Option<u8>,
    ) -> Resolution {
        let character = &world.player_character;
        let resources = &world.player_character.class_resources;

        // Handle slot conversion separately
        if metamagic == "convert_to_slot" {
            if let Some(level) = slot_level {
                let cost = level; // Costs spell level points to create a slot
                if resources.sorcery_points < cost {
                    return Resolution::new(format!(
                        "{} doesn't have enough sorcery points! Has {} but needs {} to create a level {} slot.",
                        character.name, resources.sorcery_points, cost, level
                    ));
                }
                return Resolution::new(format!(
                    "{} converts {} sorcery points into a level {} spell slot.",
                    character.name, cost, level
                ))
                .with_effect(Effect::ClassResourceUsed {
                    character_name: character.name.clone(),
                    resource_name: "Sorcery Points".to_string(),
                    description: format!("Created level {level} spell slot"),
                });
            }
        }

        if metamagic == "convert_from_slot" {
            if let Some(level) = slot_level {
                return Resolution::new(format!(
                    "{} converts a level {} spell slot into {} sorcery points.",
                    character.name, level, level
                ))
                .with_effect(Effect::ClassResourceUsed {
                    character_name: character.name.clone(),
                    resource_name: "Sorcery Points".to_string(),
                    description: format!("Gained {level} points from slot"),
                });
            }
        }

        // Regular Metamagic usage
        if resources.sorcery_points < points {
            return Resolution::new(format!(
                "{} doesn't have enough sorcery points! Has {} but needs {}.",
                character.name, resources.sorcery_points, points
            ));
        }

        let metamagic_description = match metamagic.to_lowercase().as_str() {
            "careful" => "Careful Spell: Protect allies from your spell's area effect.",
            "distant" => "Distant Spell: Double the spell's range (or 30 ft if touch).",
            "empowered" => "Empowered Spell: Reroll up to CHA mod damage dice.",
            "extended" => "Extended Spell: Double the spell's duration (max 24 hours).",
            "heightened" => "Heightened Spell: Target has disadvantage on first save.",
            "quickened" => "Quickened Spell: Cast as a bonus action instead of an action.",
            "subtle" => "Subtle Spell: Cast without verbal or somatic components.",
            "twinned" => "Twinned Spell: Target a second creature with a single-target spell.",
            _ => metamagic,
        };

        let spell_text = spell_name.map_or(String::new(), |s| format!(" on {}", s));

        Resolution::new(format!(
            "{} uses {}{} ({} sorcery point{}).",
            character.name,
            metamagic_description,
            spell_text,
            points,
            if points == 1 { "" } else { "s" }
        ))
        .with_effect(Effect::ClassResourceUsed {
            character_name: character.name.clone(),
            resource_name: "Sorcery Points".to_string(),
            description: format!("Used {points} for {metamagic}"),
        })
    }
}
