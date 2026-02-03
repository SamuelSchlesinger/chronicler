//! Spell casting resolution.

use crate::dice;
use crate::rules::helpers::roll_with_fallback;
use crate::rules::types::{Effect, Resolution};
use crate::rules::RulesEngine;
use crate::world::{CharacterId, GameWorld};

impl RulesEngine {
    pub(crate) fn resolve_cast_spell(
        &self,
        world: &GameWorld,
        _caster_id: CharacterId,
        spell_name: &str,
        slot_level: u8,
        target_names: &[String],
    ) -> Resolution {
        use crate::spells::{get_spell, SpellAttackType};

        let caster = &world.player_character;

        // Look up the spell
        let spell = match get_spell(spell_name) {
            Some(s) => s,
            None => {
                return Resolution::new(format!(
                    "Unknown spell: '{}'. The spell is not in the database.",
                    spell_name
                ));
            }
        };

        // Determine the effective slot level
        let effective_slot = if spell.level == 0 {
            0 // Cantrips don't use slots
        } else if slot_level == 0 {
            spell.level // Use base spell level if not specified
        } else if slot_level < spell.level {
            return Resolution::new(format!(
                "Cannot cast {} using a level {} slot - requires at least level {}.",
                spell.name, slot_level, spell.level
            ));
        } else {
            slot_level
        };

        // Check and consume spell slot (if not a cantrip)
        if spell.level > 0 {
            if let Some(ref spellcasting) = caster.spellcasting {
                let slot_idx = (effective_slot - 1) as usize;
                if slot_idx >= 9 {
                    return Resolution::new("Invalid spell slot level.");
                }
                let available = spellcasting.spell_slots.slots[slot_idx].available();
                if available == 0 {
                    return Resolution::new(format!(
                        "{} has no level {} spell slots remaining!",
                        caster.name, effective_slot
                    ));
                }
            } else {
                return Resolution::new(format!(
                    "{} doesn't have spellcasting ability!",
                    caster.name
                ));
            }
        }

        // Get spellcasting ability modifier
        let spell_mod = caster
            .spellcasting
            .as_ref()
            .map(|sc| caster.ability_scores.modifier(sc.ability))
            .unwrap_or(0);
        let spell_attack_bonus = spell_mod + caster.proficiency_bonus();
        // Minimum DC of 8 as a sanity floor (though in practice, no valid build would go lower)
        let spell_save_dc = (8 + spell_mod + caster.proficiency_bonus()).max(8);

        // Build the resolution
        let mut resolution = Resolution::new(String::new());
        let mut narrative_parts = Vec::new();

        // Casting announcement
        let slot_text = if spell.level == 0 {
            String::new()
        } else if effective_slot > spell.level {
            format!(" (upcast at level {})", effective_slot)
        } else {
            format!(" (level {} slot)", effective_slot)
        };
        narrative_parts.push(format!(
            "{} casts {}{}!",
            caster.name, spell.name, slot_text
        ));

        // Handle concentration
        if spell.concentration {
            narrative_parts.push("(Concentration)".to_string());
        }

        // Determine damage dice (accounting for cantrip scaling and upcasting)
        let caster_level = caster.level;
        let damage_dice = spell.effective_damage_dice(caster_level, effective_slot);

        // Handle spell attack (if applicable)
        if let Some(attack_type) = &spell.attack_type {
            let attack_type_name = match attack_type {
                SpellAttackType::Melee => "melee",
                SpellAttackType::Ranged => "ranged",
            };

            // Roll spell attack
            let attack_roll = roll_with_fallback(&format!("1d20+{}", spell_attack_bonus), "1d20");

            resolution = resolution.with_effect(Effect::DiceRolled {
                roll: attack_roll.clone(),
                purpose: format!("{} spell attack", attack_type_name),
            });

            let target_name = target_names.first().map(|s| s.as_str()).unwrap_or("target");

            // Look up target AC from combat state by name
            let target_ac = if let Some(ref combat) = world.combat {
                combat
                    .combatants
                    .iter()
                    .find(|c| c.name.eq_ignore_ascii_case(target_name))
                    .map(|c| c.armor_class)
                    .unwrap_or(10)
            } else {
                10 // Default AC outside combat
            };

            narrative_parts.push(format!(
                "Makes a {} spell attack against {}: {} vs AC {}.",
                attack_type_name, target_name, attack_roll.total, target_ac
            ));

            let hits = !attack_roll.is_fumble()
                && (attack_roll.total >= target_ac as i32 || attack_roll.is_critical());

            if hits {
                narrative_parts.push("Hit!".to_string());
                resolution = resolution.with_effect(Effect::AttackHit {
                    attacker_name: caster.name.clone(),
                    target_name: target_name.to_string(),
                    attack_roll: attack_roll.total,
                    target_ac,
                    is_critical: attack_roll.is_critical(),
                });

                // Roll damage
                if let Some(ref dice_str) = damage_dice {
                    let damage_formula = if attack_roll.is_critical() {
                        // Double dice on crit
                        if let Some(d_pos) = dice_str.find('d') {
                            let num: i32 = dice_str[..d_pos].parse().unwrap_or(1);
                            format!("{}d{}", num * 2, &dice_str[d_pos + 1..])
                        } else {
                            dice_str.clone()
                        }
                    } else {
                        dice_str.clone()
                    };

                    if let Ok(damage_roll) = dice::roll(&damage_formula) {
                        let damage_type_name =
                            spell.damage_type.map(|dt| dt.name()).unwrap_or("magical");

                        narrative_parts.push(format!(
                            "Deals {} {} damage.",
                            damage_roll.total, damage_type_name
                        ));

                        resolution = resolution.with_effect(Effect::DiceRolled {
                            roll: damage_roll,
                            purpose: format!("{} damage", spell.name),
                        });
                    }
                }
            } else {
                narrative_parts.push("Miss!".to_string());
                resolution = resolution.with_effect(Effect::AttackMissed {
                    attacker_name: caster.name.clone(),
                    target_name: target_name.to_string(),
                    attack_roll: attack_roll.total,
                    target_ac,
                });
            }
        }
        // Handle saving throw spells
        else if let Some(save_ability) = spell.save_type {
            let save_effect = spell.save_effect.as_deref().unwrap_or("negates effect");

            narrative_parts.push(format!(
                "Targets must make a DC {} {} saving throw ({} on success).",
                spell_save_dc,
                save_ability.name(),
                save_effect
            ));

            // Roll damage (before save resolution)
            if let Some(ref dice_str) = damage_dice {
                if let Ok(damage_roll) = dice::roll(dice_str) {
                    let damage_type_name =
                        spell.damage_type.map(|dt| dt.name()).unwrap_or("magical");

                    narrative_parts.push(format!(
                        "On a failed save: {} {} damage.",
                        damage_roll.total, damage_type_name
                    ));

                    resolution = resolution.with_effect(Effect::DiceRolled {
                        roll: damage_roll,
                        purpose: format!("{} damage", spell.name),
                    });
                }
            }
        }
        // Handle healing spells
        else if let Some(ref healing_dice) = spell.healing_dice {
            let healing_formula = format!("{}+{}", healing_dice, spell_mod);
            if let Ok(healing_roll) = dice::roll(&healing_formula) {
                let target_name = target_names.first().map(|s| s.as_str()).unwrap_or("target");
                narrative_parts.push(format!(
                    "{} heals {} for {} HP.",
                    caster.name, target_name, healing_roll.total
                ));

                resolution = resolution.with_effect(Effect::DiceRolled {
                    roll: healing_roll,
                    purpose: format!("{} healing", spell.name),
                });
            }
        }
        // Utility spells (no attack/save/healing)
        else {
            narrative_parts.push(spell.description.clone());
        }

        // Add spell slot consumption effect (for leveled spells)
        if spell.level > 0 {
            resolution = resolution.with_effect(Effect::SpellSlotUsed {
                level: effective_slot,
                remaining: 0, // Will be calculated by effect application
            });
        }

        resolution.narrative = narrative_parts.join(" ");
        resolution
    }

    pub(crate) fn resolve_restore_spell_slot(
        &self,
        world: &GameWorld,
        slot_level: u8,
        source: &str,
    ) -> Resolution {
        if slot_level == 0 || slot_level > 9 {
            return Resolution::new(format!(
                "Invalid spell slot level: {}. Must be between 1 and 9.",
                slot_level
            ));
        }

        let new_remaining = world
            .player_character
            .spellcasting
            .as_ref()
            .map(|sc| {
                let slot_idx = (slot_level - 1) as usize;
                sc.spell_slots.slots[slot_idx].available() + 1
            })
            .unwrap_or(0);

        Resolution::new(format!(
            "Level {} spell slot restored by {}",
            slot_level, source
        ))
        .with_effect(Effect::SpellSlotRestored {
            level: slot_level,
            new_remaining,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::types::Effect;
    use crate::world::{create_sample_cleric, create_sample_fighter, GameWorld};

    // ========== Cast Spell Tests ==========

    #[test]
    fn test_cast_spell_unknown_spell() {
        let character = create_sample_cleric("Sera");
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution = engine.resolve_cast_spell(
            &world,
            world.player_character.id,
            "Nonexistent Spell",
            1,
            &[],
        );

        assert!(resolution.narrative.contains("Unknown spell"));
    }

    #[test]
    fn test_cast_cantrip_no_slot_required() {
        let character = create_sample_cleric("Sera");
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution = engine.resolve_cast_spell(
            &world,
            world.player_character.id,
            "Sacred Flame",
            0,
            &["Goblin".to_string()],
        );

        assert!(resolution.narrative.contains("casts Sacred Flame"));
        // Cantrips don't produce SpellSlotUsed effect
        assert!(!resolution
            .effects
            .iter()
            .any(|e| matches!(e, Effect::SpellSlotUsed { .. })));
    }

    #[test]
    fn test_cast_spell_slot_too_low() {
        let character = create_sample_cleric("Sera");
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        // Try to cast a level 2 spell with a level 1 slot
        let resolution = engine.resolve_cast_spell(
            &world,
            world.player_character.id,
            "Spiritual Weapon",
            1,
            &[],
        );

        assert!(resolution.narrative.contains("Cannot cast"));
        assert!(resolution.narrative.contains("requires at least level"));
    }

    #[test]
    fn test_cast_spell_no_spellcasting_ability() {
        let character = create_sample_fighter("Roland");
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution = engine.resolve_cast_spell(
            &world,
            world.player_character.id,
            "Cure Wounds",
            1,
            &["Roland".to_string()],
        );

        assert!(resolution.narrative.contains("doesn't have spellcasting"));
    }

    #[test]
    fn test_cast_spell_consumes_slot() {
        let character = create_sample_cleric("Sera");
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution = engine.resolve_cast_spell(
            &world,
            world.player_character.id,
            "Cure Wounds",
            1,
            &["Ally".to_string()],
        );

        assert!(resolution.narrative.contains("casts Cure Wounds"));
        assert!(resolution
            .effects
            .iter()
            .any(|e| matches!(e, Effect::SpellSlotUsed { level: 1, .. })));
    }

    #[test]
    fn test_cast_spell_upcast() {
        let mut character = create_sample_cleric("Sera");
        // Add a level 2 slot
        if let Some(ref mut spellcasting) = character.spellcasting {
            spellcasting.spell_slots.slots[1] = crate::world::SlotInfo { total: 2, used: 0 };
        }
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution = engine.resolve_cast_spell(
            &world,
            world.player_character.id,
            "Cure Wounds",
            2,
            &["Ally".to_string()],
        );

        assert!(resolution.narrative.contains("upcast at level 2"));
        assert!(resolution
            .effects
            .iter()
            .any(|e| matches!(e, Effect::SpellSlotUsed { level: 2, .. })));
    }

    #[test]
    fn test_cast_spell_concentration() {
        let character = create_sample_cleric("Sera");
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        // Bless is a concentration spell
        let resolution = engine.resolve_cast_spell(
            &world,
            world.player_character.id,
            "Bless",
            1,
            &["Ally".to_string()],
        );

        assert!(resolution.narrative.contains("Concentration"));
    }

    #[test]
    fn test_cast_healing_spell() {
        let character = create_sample_cleric("Sera");
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution = engine.resolve_cast_spell(
            &world,
            world.player_character.id,
            "Cure Wounds",
            1,
            &["Roland".to_string()],
        );

        assert!(resolution.narrative.contains("heals"));
        assert!(resolution.narrative.contains("Roland"));
        assert!(resolution.effects.iter().any(
            |e| matches!(e, Effect::DiceRolled { purpose, .. } if purpose.contains("healing"))
        ));
    }

    #[test]
    fn test_cast_spell_saving_throw() {
        let character = create_sample_cleric("Sera");
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        // Sacred Flame requires a Dex save
        let resolution = engine.resolve_cast_spell(
            &world,
            world.player_character.id,
            "Sacred Flame",
            0,
            &["Goblin".to_string()],
        );

        assert!(resolution.narrative.contains("saving throw"));
        assert!(resolution.narrative.contains("DC"));
    }

    // ========== Restore Spell Slot Tests ==========

    #[test]
    fn test_restore_spell_slot_valid() {
        let character = create_sample_cleric("Sera");
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution = engine.resolve_restore_spell_slot(&world, 1, "Arcane Recovery");

        assert!(resolution.narrative.contains("Level 1 spell slot restored"));
        assert!(resolution.narrative.contains("Arcane Recovery"));
        assert!(resolution
            .effects
            .iter()
            .any(|e| matches!(e, Effect::SpellSlotRestored { level: 1, .. })));
    }

    #[test]
    fn test_restore_spell_slot_invalid_level_zero() {
        let character = create_sample_cleric("Sera");
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution = engine.resolve_restore_spell_slot(&world, 0, "test");

        assert!(resolution.narrative.contains("Invalid spell slot level"));
        assert!(resolution.effects.is_empty());
    }

    #[test]
    fn test_restore_spell_slot_invalid_level_too_high() {
        let character = create_sample_cleric("Sera");
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution = engine.resolve_restore_spell_slot(&world, 10, "test");

        assert!(resolution.narrative.contains("Invalid spell slot level"));
        assert!(resolution.effects.is_empty());
    }
}
