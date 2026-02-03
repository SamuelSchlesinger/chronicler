//! Combat-related resolution methods.

use crate::dice::{self, Advantage, DiceExpression};
use crate::rules::helpers::{roll_with_fallback, sneak_attack_dice};
use crate::rules::types::{CombatantInit, DamageType, Effect, Resolution};
use crate::rules::RulesEngine;
use crate::world::{Ability, CharacterClass, CharacterId, Condition, GameWorld};

impl RulesEngine {
    pub(crate) fn resolve_attack(
        &self,
        world: &GameWorld,
        _attacker_id: CharacterId,
        target_id: CharacterId,
        weapon_name: &str,
        advantage: Advantage,
    ) -> Resolution {
        let attacker = &world.player_character;

        // Unconscious characters cannot attack
        if attacker.has_condition(Condition::Unconscious) {
            return Resolution::new(format!(
                "{} is unconscious and cannot attack!",
                attacker.name
            ));
        }

        // Get target AC from combat state, or use player AC if targeting self
        let target_ac = if target_id == world.player_character.id {
            world.player_character.current_ac()
        } else if let Some(ref combat) = world.combat {
            combat
                .combatants
                .iter()
                .find(|c| c.id == target_id)
                .map(|c| c.armor_class)
                .unwrap_or(10) // Fallback for unknown targets
        } else {
            10 // Default AC outside combat
        };

        // Look up weapon from database or equipped weapon
        let weapon = crate::items::get_weapon(weapon_name);
        let equipped_weapon = attacker.equipment.main_hand.as_ref();

        // Determine the weapon properties
        let (damage_dice, is_finesse, is_ranged) = if let Some(w) = &weapon {
            (w.damage_dice.clone(), w.is_finesse(), w.is_ranged())
        } else if let Some(w) = equipped_weapon {
            (w.damage_dice.clone(), w.is_finesse(), w.is_ranged())
        } else {
            // Default to unarmed strike
            ("1".to_string(), false, false)
        };

        // Determine which ability modifier to use
        // Ranged: DEX only
        // Finesse: higher of STR or DEX
        // Melee: STR only
        let str_mod = attacker.ability_scores.modifier(Ability::Strength);
        let dex_mod = attacker.ability_scores.modifier(Ability::Dexterity);

        // Track if this is a strength-based melee attack (for rage bonus)
        let is_strength_melee = if is_ranged {
            false
        } else if is_finesse {
            str_mod >= dex_mod // Using STR for finesse weapon
        } else {
            true
        };

        let ability_mod = if is_ranged {
            dex_mod
        } else if is_finesse {
            str_mod.max(dex_mod)
        } else {
            str_mod
        };

        let attack_mod = ability_mod + attacker.proficiency_bonus();
        let attack_expr = DiceExpression::parse(&format!("1d20+{attack_mod}")).unwrap();
        let attack_roll = attack_expr.roll_with_advantage(advantage);

        let mut resolution = Resolution::new(format!(
            "{} attacks with {} (roll: {} vs AC {})",
            attacker.name, weapon_name, attack_roll.total, target_ac
        ));

        resolution = resolution.with_effect(Effect::DiceRolled {
            roll: attack_roll.clone(),
            purpose: format!("Attack with {weapon_name}"),
        });

        // Natural 1 always misses, natural 20 always hits (and crits)
        let hits = !attack_roll.is_fumble()
            && (attack_roll.total >= target_ac as i32 || attack_roll.is_critical());

        if hits {
            resolution = resolution.with_effect(Effect::AttackHit {
                attacker_name: attacker.name.clone(),
                target_name: "target".to_string(),
                attack_roll: attack_roll.total,
                target_ac,
                is_critical: attack_roll.is_critical(),
            });

            // Roll damage with ability modifier and rage bonus (if applicable)
            let rage_bonus = if is_strength_melee && attacker.class_resources.rage_active {
                attacker.class_resources.rage_damage_bonus as i32
            } else {
                0
            };
            let total_mod = ability_mod as i32 + rage_bonus;

            let damage_expr = if attack_roll.is_critical() {
                // Critical hit: double the number of dice
                // Parse "XdY" and produce "2XdY"
                let doubled_dice = if let Some(d_pos) = damage_dice.find('d') {
                    let num_dice: i32 = damage_dice[..d_pos].parse().unwrap_or(1);
                    let die_type = &damage_dice[d_pos..];
                    format!("{}{}", num_dice * 2, die_type)
                } else {
                    // Not a dice expression, just double the flat value
                    let flat: i32 = damage_dice.parse().unwrap_or(1);
                    format!("{}", flat * 2)
                };
                format!("{doubled_dice}+{total_mod}")
            } else {
                format!("{damage_dice}+{total_mod}")
            };
            let damage_roll = roll_with_fallback(&damage_expr, "1d4");
            resolution = resolution.with_effect(Effect::DiceRolled {
                roll: damage_roll.clone(),
                purpose: "Damage".to_string(),
            });

            // Check for Sneak Attack (Rogue feature)
            let rogue_level = attacker
                .classes
                .iter()
                .find(|c| c.class == CharacterClass::Rogue)
                .map(|c| c.level)
                .unwrap_or(0);

            if rogue_level > 0 && (is_finesse || is_ranged) {
                // Check if sneak attack conditions are met:
                // 1. Has advantage, OR
                // 2. An ally is engaged with the target (in melee range)
                let has_advantage = matches!(advantage, Advantage::Advantage);

                // Check for ally adjacent to target (any non-player ally in combat)
                let has_ally_adjacent = if let Some(ref combat) = world.combat {
                    combat
                        .combatants
                        .iter()
                        .any(|c| c.is_ally && !c.is_player && c.current_hp > 0 && c.id != target_id)
                } else {
                    false
                };

                // Check if sneak attack hasn't been used this turn
                let sneak_attack_available = if let Some(ref combat) = world.combat {
                    !combat.sneak_attack_used.contains(&attacker.id)
                } else {
                    true // Outside combat, allow it
                };

                if sneak_attack_available && (has_advantage || has_ally_adjacent) {
                    let sneak_dice = sneak_attack_dice(rogue_level);
                    let sneak_expr = if attack_roll.is_critical() {
                        format!("{}d6", sneak_dice * 2) // Double dice on crit
                    } else {
                        format!("{}d6", sneak_dice)
                    };
                    let sneak_roll = roll_with_fallback(&sneak_expr, "1d6");
                    resolution = resolution.with_effect(Effect::DiceRolled {
                        roll: sneak_roll.clone(),
                        purpose: "Sneak Attack".to_string(),
                    });
                    resolution = resolution.with_effect(Effect::SneakAttackUsed {
                        character_id: attacker.id,
                        damage_dice: sneak_dice,
                    });
                }
            }
        } else {
            resolution = resolution.with_effect(Effect::AttackMissed {
                attacker_name: attacker.name.clone(),
                target_name: "target".to_string(),
                attack_roll: attack_roll.total,
                target_ac,
            });
        }

        resolution
    }

    pub(crate) fn resolve_damage(
        &self,
        world: &GameWorld,
        target_id: CharacterId,
        amount: i32,
        damage_type: DamageType,
        source: &str,
    ) -> Resolution {
        let target = &world.player_character;

        // Special handling for damage while already at 0 HP
        if target.hit_points.current <= 0 {
            // Massive damage while at 0 HP = instant death
            if amount >= target.hit_points.maximum {
                return Resolution::new(format!(
                    "{} takes {} {} damage from {} while unconscious - INSTANT DEATH! (Damage {} >= max HP {})",
                    target.name, amount, damage_type.name(), source, amount, target.hit_points.maximum
                ))
                .with_effect(Effect::CharacterDied {
                    target_id,
                    cause: format!("Massive damage while unconscious from {source}"),
                });
            }

            // Damage while at 0 HP causes death save failures
            // (Critical hits cause 2 failures, but we don't know if this was a crit here)
            let new_failures = target.death_saves.failures + 1;
            let died = new_failures >= 3;

            if died {
                return Resolution::new(format!(
                    "{} takes {} {} damage from {} while unconscious - death save failure! Total failures: 3 - {} DIES!",
                    target.name, amount, damage_type.name(), source, target.name
                ))
                .with_effect(Effect::DeathSaveFailure {
                    target_id,
                    failures: 1,
                    total_failures: new_failures,
                    source: source.to_string(),
                })
                .with_effect(Effect::CharacterDied {
                    target_id,
                    cause: "Failed 3 death saving throws".to_string(),
                });
            }

            return Resolution::new(format!(
                "{} takes {} {} damage from {} while unconscious - death save failure! (Failures: {}/3)",
                target.name, amount, damage_type.name(), source, new_failures
            ))
            .with_effect(Effect::DeathSaveFailure {
                target_id,
                failures: 1,
                total_failures: new_failures,
                source: source.to_string(),
            });
        }

        let mut hp = target.hit_points.clone();
        let result = hp.take_damage(amount);

        // Check for massive damage (instant death)
        // If damage reduces you to 0 HP AND remaining damage >= max HP, instant death
        let overflow_damage = if result.dropped_to_zero {
            amount - (target.hit_points.current + target.hit_points.temporary)
        } else {
            0
        };
        let instant_death = result.dropped_to_zero && overflow_damage >= hp.maximum;

        // Build narrative with HP status so DM knows the character's state
        let hp_status = if instant_death {
            format!(
                " (INSTANT DEATH! Massive damage ({} overflow) exceeds max HP of {})",
                overflow_damage, hp.maximum
            )
        } else if result.dropped_to_zero {
            format!(
                " (HP: 0/{} - UNCONSCIOUS! Character falls and begins making death saving throws)",
                hp.maximum
            )
        } else if hp.current <= hp.maximum / 4 {
            format!(" (HP: {}/{} - critically wounded)", hp.current, hp.maximum)
        } else if hp.current <= hp.maximum / 2 {
            format!(" (HP: {}/{} - bloodied)", hp.current, hp.maximum)
        } else {
            format!(" (HP: {}/{})", hp.current, hp.maximum)
        };

        let mut resolution = Resolution::new(format!(
            "{} takes {} {} damage from {}{}",
            target.name,
            amount,
            damage_type.name(),
            source,
            hp_status
        ));

        resolution = resolution.with_effect(Effect::HpChanged {
            target_id,
            amount: -amount,
            new_current: hp.current,
            new_max: hp.maximum,
            dropped_to_zero: result.dropped_to_zero,
        });

        if instant_death {
            resolution = resolution.with_effect(Effect::CharacterDied {
                target_id,
                cause: format!("Massive damage from {source}"),
            });
        }

        resolution
    }

    pub(crate) fn resolve_heal(
        &self,
        world: &GameWorld,
        target_id: CharacterId,
        amount: i32,
        source: &str,
    ) -> Resolution {
        let target = &world.player_character;
        let mut hp = target.hit_points.clone();
        let was_unconscious = hp.current <= 0;
        let healed = hp.heal(amount);

        // Build narrative with HP status
        let hp_status = if was_unconscious && hp.current > 0 {
            format!(
                " (HP: {}/{} - regains consciousness!)",
                hp.current, hp.maximum
            )
        } else if hp.current == hp.maximum {
            format!(" (HP: {}/{} - fully healed)", hp.current, hp.maximum)
        } else {
            format!(" (HP: {}/{})", hp.current, hp.maximum)
        };

        let resolution = Resolution::new(format!(
            "{} heals {} hit points from {}{}",
            target.name, healed, source, hp_status
        ));

        resolution.with_effect(Effect::HpChanged {
            target_id,
            amount: healed,
            new_current: hp.current,
            new_max: hp.maximum,
            dropped_to_zero: false,
        })
    }

    pub(crate) fn resolve_apply_condition(
        &self,
        world: &GameWorld,
        target_id: CharacterId,
        condition: Condition,
        source: &str,
        duration_rounds: Option<u32>,
    ) -> Resolution {
        let target = &world.player_character;

        let duration_text = duration_rounds
            .map(|d| format!(" for {} rounds", d))
            .unwrap_or_default();

        let resolution = Resolution::new(format!(
            "{} is now {} ({}){}",
            target.name,
            condition.name(),
            source,
            duration_text
        ));

        resolution.with_effect(Effect::ConditionApplied {
            target_id,
            condition,
            source: source.to_string(),
            duration_rounds,
        })
    }

    pub(crate) fn resolve_remove_condition(
        &self,
        world: &GameWorld,
        target_id: CharacterId,
        condition: Condition,
    ) -> Resolution {
        let target = &world.player_character;

        let resolution =
            Resolution::new(format!("{} is no longer {}", target.name, condition.name()));

        resolution.with_effect(Effect::ConditionRemoved {
            target_id,
            condition,
        })
    }

    pub(crate) fn resolve_start_combat(
        &self,
        world: &GameWorld,
        combatants: Vec<CombatantInit>,
    ) -> Resolution {
        let mut resolution = Resolution::new("Combat begins! Roll for initiative.")
            .with_effect(Effect::CombatStarted);

        // Roll initiative for each combatant
        for init in combatants {
            let modifier = if init.is_player {
                world.player_character.initiative_modifier()
            } else {
                init.initiative_modifier
            };

            let roll = dice::roll("1d20").unwrap();
            let total = roll.total + modifier as i32;

            resolution = resolution.with_effect(Effect::InitiativeRolled {
                character_id: init.id,
                name: init.name.clone(),
                roll: roll.total,
                total,
            });

            resolution = resolution.with_effect(Effect::CombatantAdded {
                id: init.id,
                name: init.name,
                initiative: total,
                is_ally: init.is_ally,
                current_hp: init.current_hp,
                max_hp: init.max_hp,
                armor_class: init.armor_class,
            });
        }

        resolution
    }

    pub(crate) fn resolve_end_combat(&self, _world: &GameWorld) -> Resolution {
        Resolution::new("Combat ends.").with_effect(Effect::CombatEnded)
    }

    pub(crate) fn resolve_next_turn(&self, world: &GameWorld) -> Resolution {
        if let Some(ref combat) = world.combat {
            let mut combat_clone = combat.clone();
            combat_clone.next_turn();

            let current = combat_clone
                .current_combatant()
                .map(|c| c.name.clone())
                .unwrap_or_else(|| "Unknown".to_string());

            Resolution::new(format!(
                "Next turn: {} (Round {})",
                current, combat_clone.round
            ))
            .with_effect(Effect::TurnAdvanced {
                round: combat_clone.round,
                current_combatant: current,
            })
        } else {
            Resolution::new("No combat in progress")
        }
    }

    pub(crate) fn resolve_roll_initiative(
        &self,
        character_id: CharacterId,
        name: &str,
        modifier: i8,
        _is_player: bool,
    ) -> Resolution {
        let roll = dice::roll("1d20").unwrap();
        let total = roll.total + modifier as i32;

        Resolution::new(format!(
            "{} rolls initiative: {} + {} = {}",
            name, roll.total, modifier, total
        ))
        .with_effect(Effect::DiceRolled {
            roll: roll.clone(),
            purpose: "Initiative".to_string(),
        })
        .with_effect(Effect::InitiativeRolled {
            character_id,
            name: name.to_string(),
            roll: roll.total,
            total,
        })
    }

    pub(crate) fn resolve_death_save(
        &self,
        world: &GameWorld,
        character_id: CharacterId,
    ) -> Resolution {
        let character = &world.player_character;

        // Must be at 0 HP to make death saves
        if character.hit_points.current > 0 {
            return Resolution::new(format!(
                "{} is not dying and doesn't need to make a death save.",
                character.name
            ));
        }

        // Roll d20
        let roll = dice::roll("1d20").unwrap();
        let roll_value = roll.total;

        // Check for natural 20 - regain 1 HP
        if roll.is_critical() {
            return Resolution::new(format!(
                "{} rolls a NATURAL 20 on their death save! They regain 1 HP and become conscious!",
                character.name
            ))
            .with_effect(Effect::DeathSavesReset {
                target_id: character_id,
            })
            .with_effect(Effect::HpChanged {
                target_id: character_id,
                amount: 1,
                new_current: 1,
                new_max: character.hit_points.maximum,
                dropped_to_zero: false,
            })
            .with_effect(Effect::ConditionRemoved {
                target_id: character_id,
                condition: Condition::Unconscious,
            });
        }

        // Check for natural 1 - counts as 2 failures
        if roll.is_fumble() {
            let new_failures = character.death_saves.failures + 2;
            if new_failures >= 3 {
                return Resolution::new(format!(
                    "{} rolls a NATURAL 1 on their death save! Two failures! {} has died!",
                    character.name, character.name
                ))
                .with_effect(Effect::DeathSaveFailure {
                    target_id: character_id,
                    failures: 2,
                    total_failures: new_failures.min(3),
                    source: "Natural 1 on death save".to_string(),
                })
                .with_effect(Effect::CharacterDied {
                    target_id: character_id,
                    cause: "Failed death saves".to_string(),
                });
            } else {
                return Resolution::new(format!(
                    "{} rolls a NATURAL 1 on their death save! That counts as TWO failures! ({}/3)",
                    character.name, new_failures
                ))
                .with_effect(Effect::DeathSaveFailure {
                    target_id: character_id,
                    failures: 2,
                    total_failures: new_failures,
                    source: "Natural 1 on death save".to_string(),
                });
            }
        }

        // Normal roll - 10+ is success, <10 is failure
        if roll_value >= 10 {
            let new_successes = character.death_saves.successes + 1;
            if new_successes >= 3 {
                Resolution::new(format!(
                    "{} rolls {} on their death save - SUCCESS! With 3 successes, {} is now STABLE!",
                    character.name, roll_value, character.name
                ))
                .with_effect(Effect::DeathSaveSuccess {
                    target_id: character_id,
                    roll: roll_value,
                    total_successes: 3,
                })
                .with_effect(Effect::Stabilized {
                    target_id: character_id,
                })
            } else {
                Resolution::new(format!(
                    "{} rolls {} on their death save - SUCCESS! ({}/3 successes)",
                    character.name, roll_value, new_successes
                ))
                .with_effect(Effect::DeathSaveSuccess {
                    target_id: character_id,
                    roll: roll_value,
                    total_successes: new_successes,
                })
            }
        } else {
            let new_failures = character.death_saves.failures + 1;
            if new_failures >= 3 {
                Resolution::new(format!(
                    "{} rolls {} on their death save - FAILURE! With 3 failures, {} has DIED!",
                    character.name, roll_value, character.name
                ))
                .with_effect(Effect::DeathSaveFailure {
                    target_id: character_id,
                    failures: 1,
                    total_failures: 3,
                    source: "Death save".to_string(),
                })
                .with_effect(Effect::CharacterDied {
                    target_id: character_id,
                    cause: "Failed death saves".to_string(),
                })
            } else {
                Resolution::new(format!(
                    "{} rolls {} on their death save - FAILURE! ({}/3 failures)",
                    character.name, roll_value, new_failures
                ))
                .with_effect(Effect::DeathSaveFailure {
                    target_id: character_id,
                    failures: 1,
                    total_failures: new_failures,
                    source: "Death save".to_string(),
                })
            }
        }
    }

    pub(crate) fn resolve_concentration_check(
        &self,
        world: &GameWorld,
        character_id: CharacterId,
        damage_taken: i32,
        spell_name: &str,
    ) -> Resolution {
        let character = &world.player_character;

        // Calculate DC: max(10, damage / 2)
        let dc = (damage_taken / 2).max(10);

        // Get CON modifier
        let con_mod = character.ability_scores.modifier(Ability::Constitution);
        let proficiency = character.proficiency_bonus();

        // Check if proficient in CON saves (some classes like Sorcerer, Wizard with War Caster)
        // For now, assume base CON save without proficiency unless they have the save proficiency
        let save_mod = if character
            .saving_throw_proficiencies
            .contains(&Ability::Constitution)
        {
            con_mod + proficiency
        } else {
            con_mod
        };

        // Roll the save
        let roll = dice::roll(&format!("1d20+{save_mod}")).unwrap();
        let roll_total = roll.total;

        if roll_total >= dc {
            Resolution::new(format!(
                "{} makes a DC {} Constitution save to maintain concentration on {}. Rolls {} - SUCCESS! Concentration maintained.",
                character.name, dc, spell_name, roll_total
            ))
            .with_effect(Effect::ConcentrationMaintained {
                character_id,
                spell_name: spell_name.to_string(),
                roll: roll_total,
                dc,
            })
        } else {
            Resolution::new(format!(
                "{} makes a DC {} Constitution save to maintain concentration on {}. Rolls {} - FAILED! Concentration is broken!",
                character.name, dc, spell_name, roll_total
            ))
            .with_effect(Effect::ConcentrationBroken {
                character_id,
                spell_name: spell_name.to_string(),
                damage_taken,
                roll: roll_total,
                dc,
            })
        }
    }
}
