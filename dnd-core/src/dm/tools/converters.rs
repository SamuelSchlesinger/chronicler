//! String-to-enum converters for parsing tool inputs.
//!
//! These pure functions convert string representations from JSON tool inputs
//! into their corresponding D&D enum types.

use crate::dice::Advantage;
use crate::rules::DamageType;
use crate::world::{Ability, Condition, Skill};

/// Parse a skill name string into a Skill enum.
pub fn parse_skill(s: &str) -> Option<Skill> {
    match s.to_lowercase().replace('_', "").as_str() {
        "athletics" => Some(Skill::Athletics),
        "acrobatics" => Some(Skill::Acrobatics),
        "sleightofhand" => Some(Skill::SleightOfHand),
        "stealth" => Some(Skill::Stealth),
        "arcana" => Some(Skill::Arcana),
        "history" => Some(Skill::History),
        "investigation" => Some(Skill::Investigation),
        "nature" => Some(Skill::Nature),
        "religion" => Some(Skill::Religion),
        "animalhandling" => Some(Skill::AnimalHandling),
        "insight" => Some(Skill::Insight),
        "medicine" => Some(Skill::Medicine),
        "perception" => Some(Skill::Perception),
        "survival" => Some(Skill::Survival),
        "deception" => Some(Skill::Deception),
        "intimidation" => Some(Skill::Intimidation),
        "performance" => Some(Skill::Performance),
        "persuasion" => Some(Skill::Persuasion),
        _ => None,
    }
}

/// Parse an ability name string into an Ability enum.
pub fn parse_ability(s: &str) -> Option<Ability> {
    match s.to_lowercase().as_str() {
        "strength" | "str" => Some(Ability::Strength),
        "dexterity" | "dex" => Some(Ability::Dexterity),
        "constitution" | "con" => Some(Ability::Constitution),
        "intelligence" | "int" => Some(Ability::Intelligence),
        "wisdom" | "wis" => Some(Ability::Wisdom),
        "charisma" | "cha" => Some(Ability::Charisma),
        _ => None,
    }
}

/// Parse an advantage string into an Advantage enum.
pub fn parse_advantage(s: Option<&str>) -> Advantage {
    match s {
        Some("advantage") => Advantage::Advantage,
        Some("disadvantage") => Advantage::Disadvantage,
        _ => Advantage::Normal,
    }
}

/// Parse a damage type string into a DamageType enum.
pub fn parse_damage_type(s: &str) -> Option<DamageType> {
    match s.to_lowercase().as_str() {
        "slashing" => Some(DamageType::Slashing),
        "piercing" => Some(DamageType::Piercing),
        "bludgeoning" => Some(DamageType::Bludgeoning),
        "fire" => Some(DamageType::Fire),
        "cold" => Some(DamageType::Cold),
        "lightning" => Some(DamageType::Lightning),
        "thunder" => Some(DamageType::Thunder),
        "acid" => Some(DamageType::Acid),
        "poison" => Some(DamageType::Poison),
        "necrotic" => Some(DamageType::Necrotic),
        "radiant" => Some(DamageType::Radiant),
        "force" => Some(DamageType::Force),
        "psychic" => Some(DamageType::Psychic),
        _ => None,
    }
}

/// Parse a condition string into a Condition enum.
pub fn parse_condition(s: &str) -> Option<Condition> {
    match s.to_lowercase().as_str() {
        "blinded" => Some(Condition::Blinded),
        "charmed" => Some(Condition::Charmed),
        "deafened" => Some(Condition::Deafened),
        "frightened" => Some(Condition::Frightened),
        "grappled" => Some(Condition::Grappled),
        "incapacitated" => Some(Condition::Incapacitated),
        "invisible" => Some(Condition::Invisible),
        "paralyzed" => Some(Condition::Paralyzed),
        "petrified" => Some(Condition::Petrified),
        "poisoned" => Some(Condition::Poisoned),
        "prone" => Some(Condition::Prone),
        "restrained" => Some(Condition::Restrained),
        "stunned" => Some(Condition::Stunned),
        "unconscious" => Some(Condition::Unconscious),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ability() {
        assert_eq!(parse_ability("strength"), Some(Ability::Strength));
        assert_eq!(parse_ability("str"), Some(Ability::Strength));
        assert_eq!(parse_ability("STR"), Some(Ability::Strength));
        assert_eq!(parse_ability("dexterity"), Some(Ability::Dexterity));
        assert_eq!(parse_ability("dex"), Some(Ability::Dexterity));
        assert_eq!(parse_ability("constitution"), Some(Ability::Constitution));
        assert_eq!(parse_ability("con"), Some(Ability::Constitution));
        assert_eq!(parse_ability("intelligence"), Some(Ability::Intelligence));
        assert_eq!(parse_ability("int"), Some(Ability::Intelligence));
        assert_eq!(parse_ability("wisdom"), Some(Ability::Wisdom));
        assert_eq!(parse_ability("wis"), Some(Ability::Wisdom));
        assert_eq!(parse_ability("charisma"), Some(Ability::Charisma));
        assert_eq!(parse_ability("cha"), Some(Ability::Charisma));
        assert_eq!(parse_ability("invalid"), None);
    }

    #[test]
    fn test_parse_skill() {
        assert_eq!(parse_skill("athletics"), Some(Skill::Athletics));
        assert_eq!(parse_skill("stealth"), Some(Skill::Stealth));
        assert_eq!(parse_skill("perception"), Some(Skill::Perception));
        assert_eq!(parse_skill("persuasion"), Some(Skill::Persuasion));
        assert_eq!(parse_skill("sleight_of_hand"), Some(Skill::SleightOfHand));
        assert_eq!(parse_skill("animal_handling"), Some(Skill::AnimalHandling));
        assert_eq!(parse_skill("invalid"), None);
    }

    #[test]
    fn test_parse_advantage() {
        assert_eq!(parse_advantage(Some("advantage")), Advantage::Advantage);
        assert_eq!(
            parse_advantage(Some("disadvantage")),
            Advantage::Disadvantage
        );
        assert_eq!(parse_advantage(Some("normal")), Advantage::Normal);
        assert_eq!(parse_advantage(None), Advantage::Normal);
    }

    #[test]
    fn test_parse_damage_type() {
        assert_eq!(parse_damage_type("slashing"), Some(DamageType::Slashing));
        assert_eq!(parse_damage_type("SLASHING"), Some(DamageType::Slashing));
        assert_eq!(parse_damage_type("piercing"), Some(DamageType::Piercing));
        assert_eq!(
            parse_damage_type("bludgeoning"),
            Some(DamageType::Bludgeoning)
        );
        assert_eq!(parse_damage_type("fire"), Some(DamageType::Fire));
        assert_eq!(parse_damage_type("cold"), Some(DamageType::Cold));
        assert_eq!(parse_damage_type("lightning"), Some(DamageType::Lightning));
        assert_eq!(parse_damage_type("psychic"), Some(DamageType::Psychic));
        assert_eq!(parse_damage_type("invalid"), None);
    }

    #[test]
    fn test_parse_condition() {
        assert_eq!(parse_condition("blinded"), Some(Condition::Blinded));
        assert_eq!(parse_condition("BLINDED"), Some(Condition::Blinded));
        assert_eq!(parse_condition("charmed"), Some(Condition::Charmed));
        assert_eq!(parse_condition("frightened"), Some(Condition::Frightened));
        assert_eq!(parse_condition("grappled"), Some(Condition::Grappled));
        assert_eq!(
            parse_condition("incapacitated"),
            Some(Condition::Incapacitated)
        );
        assert_eq!(parse_condition("invisible"), Some(Condition::Invisible));
        assert_eq!(parse_condition("paralyzed"), Some(Condition::Paralyzed));
        assert_eq!(parse_condition("poisoned"), Some(Condition::Poisoned));
        assert_eq!(parse_condition("prone"), Some(Condition::Prone));
        assert_eq!(parse_condition("stunned"), Some(Condition::Stunned));
        assert_eq!(parse_condition("unconscious"), Some(Condition::Unconscious));
        assert_eq!(parse_condition("invalid"), None);
    }
}
