//! Skill checks, ability checks, saving throws, and dice rolls.

use crate::dice::{self, Advantage, DiceExpression};
use crate::rules::types::{Effect, Resolution};
use crate::rules::RulesEngine;
use crate::world::{Ability, CharacterId, Condition, GameWorld, Skill};

impl RulesEngine {
    pub(crate) fn resolve_skill_check(
        &self,
        world: &GameWorld,
        _character_id: CharacterId,
        skill: Skill,
        dc: i32,
        advantage: Advantage,
        description: &str,
    ) -> Resolution {
        let character = &world.player_character;

        // Unconscious characters automatically fail Strength and Dexterity checks
        if character.has_condition(Condition::Unconscious) {
            let ability = skill.ability();
            if matches!(ability, Ability::Strength | Ability::Dexterity) {
                return Resolution::new(format!(
                    "{} is unconscious and automatically fails the {} check!",
                    character.name,
                    skill.name()
                ))
                .with_effect(Effect::CheckFailed {
                    check_type: skill.name().to_string(),
                    roll: 0,
                    dc,
                });
            }
        }

        let modifier = character.skill_modifier(skill);

        // Check for armor-imposed stealth disadvantage
        let effective_advantage = if skill == Skill::Stealth {
            if let Some(ref armor) = character.equipment.armor {
                if armor.stealth_disadvantage {
                    // Armor imposes disadvantage on Stealth
                    advantage.combine(Advantage::Disadvantage)
                } else {
                    advantage
                }
            } else {
                advantage
            }
        } else {
            advantage
        };

        let expr = DiceExpression::parse(&format!("1d20+{modifier}")).unwrap();
        let roll = expr.roll_with_advantage(effective_advantage);

        let success = roll.total >= dc;
        let result_str = if success { "succeeds" } else { "fails" };

        // Note if stealth disadvantage was applied
        let disadvantage_note = if skill == Skill::Stealth
            && effective_advantage != advantage
            && matches!(effective_advantage, Advantage::Disadvantage)
        {
            " [armor disadvantage]"
        } else {
            ""
        };

        let mut resolution = Resolution::new(format!(
            "{} {} ({} check: {} vs DC {}){}",
            character.name,
            result_str,
            skill.name(),
            roll.total,
            dc,
            disadvantage_note
        ));

        resolution = resolution.with_effect(Effect::DiceRolled {
            roll: roll.clone(),
            purpose: format!("{} check - {}", skill.name(), description),
        });

        if success {
            resolution = resolution.with_effect(Effect::CheckSucceeded {
                check_type: skill.name().to_string(),
                roll: roll.total,
                dc,
            });
        } else {
            resolution = resolution.with_effect(Effect::CheckFailed {
                check_type: skill.name().to_string(),
                roll: roll.total,
                dc,
            });
        }

        resolution
    }

    pub(crate) fn resolve_ability_check(
        &self,
        world: &GameWorld,
        _character_id: CharacterId,
        ability: Ability,
        dc: i32,
        advantage: Advantage,
        description: &str,
    ) -> Resolution {
        let character = &world.player_character;

        // Unconscious characters automatically fail Strength and Dexterity checks
        if character.has_condition(Condition::Unconscious)
            && matches!(ability, Ability::Strength | Ability::Dexterity)
        {
            return Resolution::new(format!(
                "{} is unconscious and automatically fails the {} check!",
                character.name,
                ability.abbreviation()
            ))
            .with_effect(Effect::CheckFailed {
                check_type: format!("{} check", ability.abbreviation()),
                roll: 0,
                dc,
            });
        }

        let modifier = character.ability_scores.modifier(ability);

        let expr = DiceExpression::parse(&format!("1d20+{modifier}")).unwrap();
        let roll = expr.roll_with_advantage(advantage);

        let success = roll.total >= dc;
        let result_str = if success { "succeeds" } else { "fails" };

        let mut resolution = Resolution::new(format!(
            "{} {} ({} check: {} vs DC {})",
            character.name,
            result_str,
            ability.abbreviation(),
            roll.total,
            dc
        ));

        resolution = resolution.with_effect(Effect::DiceRolled {
            roll: roll.clone(),
            purpose: format!("{} check - {}", ability.abbreviation(), description),
        });

        if success {
            resolution.with_effect(Effect::CheckSucceeded {
                check_type: ability.abbreviation().to_string(),
                roll: roll.total,
                dc,
            })
        } else {
            resolution.with_effect(Effect::CheckFailed {
                check_type: ability.abbreviation().to_string(),
                roll: roll.total,
                dc,
            })
        }
    }

    pub(crate) fn resolve_saving_throw(
        &self,
        world: &GameWorld,
        _character_id: CharacterId,
        ability: Ability,
        dc: i32,
        advantage: Advantage,
        source: &str,
    ) -> Resolution {
        let character = &world.player_character;

        // Unconscious characters automatically fail Strength and Dexterity saving throws
        if character.has_condition(Condition::Unconscious)
            && matches!(ability, Ability::Strength | Ability::Dexterity)
        {
            return Resolution::new(format!(
                "{} is unconscious and automatically fails the {} saving throw!",
                character.name,
                ability.abbreviation()
            ))
            .with_effect(Effect::CheckFailed {
                check_type: format!("{} save", ability.abbreviation()),
                roll: 0,
                dc,
            });
        }

        let modifier = character.saving_throw_modifier(ability);

        let expr = DiceExpression::parse(&format!("1d20+{modifier}")).unwrap();
        let roll = expr.roll_with_advantage(advantage);

        let success = roll.total >= dc;
        let result_str = if success { "succeeds" } else { "fails" };

        let mut resolution = Resolution::new(format!(
            "{} {} on {} saving throw ({} vs DC {})",
            character.name,
            result_str,
            ability.abbreviation(),
            roll.total,
            dc
        ));

        resolution = resolution.with_effect(Effect::DiceRolled {
            roll: roll.clone(),
            purpose: format!("{} save vs {}", ability.abbreviation(), source),
        });

        if success {
            resolution.with_effect(Effect::CheckSucceeded {
                check_type: format!("{} save", ability.abbreviation()),
                roll: roll.total,
                dc,
            })
        } else {
            resolution.with_effect(Effect::CheckFailed {
                check_type: format!("{} save", ability.abbreviation()),
                roll: roll.total,
                dc,
            })
        }
    }

    pub(crate) fn resolve_roll_dice(&self, notation: &str, purpose: &str) -> Resolution {
        match dice::roll(notation) {
            Ok(roll) => Resolution::new(format!("Rolling {notation} for {purpose}: {roll}"))
                .with_effect(Effect::DiceRolled {
                    roll,
                    purpose: purpose.to_string(),
                }),
            Err(e) => Resolution::new(format!("Failed to roll {notation}: {e}")),
        }
    }
}
