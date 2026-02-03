//! Ability scores for D&D 5e characters.
//!
//! This module defines the six ability scores (Strength, Dexterity, Constitution,
//! Intelligence, Wisdom, Charisma) and the container type for storing them.

use serde::{Deserialize, Serialize};
use std::fmt;

// ============================================================================
// Ability Scores
// ============================================================================

/// The six ability scores.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Ability {
    Strength,
    Dexterity,
    Constitution,
    Intelligence,
    Wisdom,
    Charisma,
}

impl Ability {
    pub fn abbreviation(&self) -> &'static str {
        match self {
            Ability::Strength => "STR",
            Ability::Dexterity => "DEX",
            Ability::Constitution => "CON",
            Ability::Intelligence => "INT",
            Ability::Wisdom => "WIS",
            Ability::Charisma => "CHA",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Ability::Strength => "Strength",
            Ability::Dexterity => "Dexterity",
            Ability::Constitution => "Constitution",
            Ability::Intelligence => "Intelligence",
            Ability::Wisdom => "Wisdom",
            Ability::Charisma => "Charisma",
        }
    }

    pub fn all() -> [Ability; 6] {
        [
            Ability::Strength,
            Ability::Dexterity,
            Ability::Constitution,
            Ability::Intelligence,
            Ability::Wisdom,
            Ability::Charisma,
        ]
    }
}

impl fmt::Display for Ability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.abbreviation())
    }
}

/// Ability scores container.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbilityScores {
    pub strength: u8,
    pub dexterity: u8,
    pub constitution: u8,
    pub intelligence: u8,
    pub wisdom: u8,
    pub charisma: u8,
}

impl AbilityScores {
    pub fn new(str: u8, dex: u8, con: u8, int: u8, wis: u8, cha: u8) -> Self {
        Self {
            strength: str,
            dexterity: dex,
            constitution: con,
            intelligence: int,
            wisdom: wis,
            charisma: cha,
        }
    }

    pub fn standard_array() -> Self {
        Self::new(15, 14, 13, 12, 10, 8)
    }

    pub fn get(&self, ability: Ability) -> u8 {
        match ability {
            Ability::Strength => self.strength,
            Ability::Dexterity => self.dexterity,
            Ability::Constitution => self.constitution,
            Ability::Intelligence => self.intelligence,
            Ability::Wisdom => self.wisdom,
            Ability::Charisma => self.charisma,
        }
    }

    pub fn set(&mut self, ability: Ability, value: u8) {
        match ability {
            Ability::Strength => self.strength = value,
            Ability::Dexterity => self.dexterity = value,
            Ability::Constitution => self.constitution = value,
            Ability::Intelligence => self.intelligence = value,
            Ability::Wisdom => self.wisdom = value,
            Ability::Charisma => self.charisma = value,
        }
    }

    pub fn modifier(&self, ability: Ability) -> i8 {
        let score = self.get(ability) as i8;
        // Use floor division to correctly handle negative numbers
        // D&D 5e: score 8-9 = -1, 10-11 = 0, 12-13 = +1, etc.
        (score - 10).div_euclid(2)
    }
}

impl Default for AbilityScores {
    fn default() -> Self {
        Self::new(10, 10, 10, 10, 10, 10)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ability_abbreviations() {
        assert_eq!(Ability::Strength.abbreviation(), "STR");
        assert_eq!(Ability::Dexterity.abbreviation(), "DEX");
        assert_eq!(Ability::Constitution.abbreviation(), "CON");
        assert_eq!(Ability::Intelligence.abbreviation(), "INT");
        assert_eq!(Ability::Wisdom.abbreviation(), "WIS");
        assert_eq!(Ability::Charisma.abbreviation(), "CHA");
    }

    #[test]
    fn test_ability_names() {
        assert_eq!(Ability::Strength.name(), "Strength");
        assert_eq!(Ability::Dexterity.name(), "Dexterity");
        assert_eq!(Ability::Constitution.name(), "Constitution");
        assert_eq!(Ability::Intelligence.name(), "Intelligence");
        assert_eq!(Ability::Wisdom.name(), "Wisdom");
        assert_eq!(Ability::Charisma.name(), "Charisma");
    }

    #[test]
    fn test_ability_all() {
        let all = Ability::all();
        assert_eq!(all.len(), 6);
        assert!(all.contains(&Ability::Strength));
        assert!(all.contains(&Ability::Charisma));
    }

    #[test]
    fn test_ability_display() {
        assert_eq!(format!("{}", Ability::Strength), "STR");
        assert_eq!(format!("{}", Ability::Charisma), "CHA");
    }

    #[test]
    fn test_ability_scores_new() {
        let scores = AbilityScores::new(15, 14, 13, 12, 10, 8);
        assert_eq!(scores.strength, 15);
        assert_eq!(scores.dexterity, 14);
        assert_eq!(scores.constitution, 13);
        assert_eq!(scores.intelligence, 12);
        assert_eq!(scores.wisdom, 10);
        assert_eq!(scores.charisma, 8);
    }

    #[test]
    fn test_ability_scores_standard_array() {
        let scores = AbilityScores::standard_array();
        assert_eq!(scores.strength, 15);
        assert_eq!(scores.dexterity, 14);
        assert_eq!(scores.constitution, 13);
        assert_eq!(scores.intelligence, 12);
        assert_eq!(scores.wisdom, 10);
        assert_eq!(scores.charisma, 8);
    }

    #[test]
    fn test_ability_scores_default() {
        let scores = AbilityScores::default();
        assert_eq!(scores.strength, 10);
        assert_eq!(scores.dexterity, 10);
        assert_eq!(scores.constitution, 10);
        assert_eq!(scores.intelligence, 10);
        assert_eq!(scores.wisdom, 10);
        assert_eq!(scores.charisma, 10);
    }

    #[test]
    fn test_ability_scores_get() {
        let scores = AbilityScores::new(18, 16, 14, 12, 10, 8);
        assert_eq!(scores.get(Ability::Strength), 18);
        assert_eq!(scores.get(Ability::Dexterity), 16);
        assert_eq!(scores.get(Ability::Constitution), 14);
        assert_eq!(scores.get(Ability::Intelligence), 12);
        assert_eq!(scores.get(Ability::Wisdom), 10);
        assert_eq!(scores.get(Ability::Charisma), 8);
    }

    #[test]
    fn test_ability_scores_set() {
        let mut scores = AbilityScores::default();
        scores.set(Ability::Strength, 20);
        scores.set(Ability::Charisma, 5);
        assert_eq!(scores.get(Ability::Strength), 20);
        assert_eq!(scores.get(Ability::Charisma), 5);
    }

    #[test]
    fn test_modifier_positive() {
        let scores = AbilityScores::new(18, 16, 14, 12, 11, 10);
        assert_eq!(scores.modifier(Ability::Strength), 4); // 18 -> +4
        assert_eq!(scores.modifier(Ability::Dexterity), 3); // 16 -> +3
        assert_eq!(scores.modifier(Ability::Constitution), 2); // 14 -> +2
        assert_eq!(scores.modifier(Ability::Intelligence), 1); // 12 -> +1
        assert_eq!(scores.modifier(Ability::Wisdom), 0); // 11 -> +0
        assert_eq!(scores.modifier(Ability::Charisma), 0); // 10 -> +0
    }

    #[test]
    fn test_modifier_negative() {
        let scores = AbilityScores::new(9, 8, 7, 6, 4, 1);
        assert_eq!(scores.modifier(Ability::Strength), -1); // 9 -> -1
        assert_eq!(scores.modifier(Ability::Dexterity), -1); // 8 -> -1
        assert_eq!(scores.modifier(Ability::Constitution), -2); // 7 -> -2
        assert_eq!(scores.modifier(Ability::Intelligence), -2); // 6 -> -2
        assert_eq!(scores.modifier(Ability::Wisdom), -3); // 4 -> -3
        assert_eq!(scores.modifier(Ability::Charisma), -5); // 1 -> -5
    }

    #[test]
    fn test_modifier_boundary_values() {
        // Test important D&D boundaries
        let mut scores = AbilityScores::default();

        // Score of 1 (minimum for living creatures)
        scores.set(Ability::Strength, 1);
        assert_eq!(scores.modifier(Ability::Strength), -5);

        // Score of 10-11 (average, +0 modifier)
        scores.set(Ability::Strength, 10);
        assert_eq!(scores.modifier(Ability::Strength), 0);
        scores.set(Ability::Strength, 11);
        assert_eq!(scores.modifier(Ability::Strength), 0);

        // Score of 20 (typical maximum for PCs)
        scores.set(Ability::Strength, 20);
        assert_eq!(scores.modifier(Ability::Strength), 5);

        // Score of 30 (deity-level)
        scores.set(Ability::Strength, 30);
        assert_eq!(scores.modifier(Ability::Strength), 10);
    }

    #[test]
    fn test_modifier_formula_edge_cases() {
        // Verify the floor division formula: (score - 10) / 2 rounded down
        let mut scores = AbilityScores::default();

        // Even scores
        scores.set(Ability::Strength, 2);
        assert_eq!(scores.modifier(Ability::Strength), -4);
        scores.set(Ability::Strength, 4);
        assert_eq!(scores.modifier(Ability::Strength), -3);

        // Odd scores (should round down)
        scores.set(Ability::Strength, 3);
        assert_eq!(scores.modifier(Ability::Strength), -4); // (3-10)/2 = -3.5 -> -4
        scores.set(Ability::Strength, 5);
        assert_eq!(scores.modifier(Ability::Strength), -3); // (5-10)/2 = -2.5 -> -3
    }
}
