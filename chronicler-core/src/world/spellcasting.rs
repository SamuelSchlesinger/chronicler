//! Spellcasting data structures and mechanics.
//!
//! This module contains types for tracking spellcasting abilities,
//! spell slots, and related functionality for D&D 5e characters.

use serde::{Deserialize, Serialize};

use super::{Ability, AbilityScores};

/// Spellcasting data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpellcastingData {
    pub ability: Ability,
    pub spells_known: Vec<String>,
    pub spells_prepared: Vec<String>,
    pub cantrips_known: Vec<String>,
    pub spell_slots: SpellSlots,
}

impl SpellcastingData {
    pub fn spell_save_dc(&self, ability_scores: &AbilityScores, proficiency: i8) -> u8 {
        let ability_mod = ability_scores.modifier(self.ability);
        (8 + proficiency + ability_mod).max(0) as u8
    }

    pub fn spell_attack_bonus(&self, ability_scores: &AbilityScores, proficiency: i8) -> i8 {
        ability_scores.modifier(self.ability) + proficiency
    }
}

/// Spell slot tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpellSlots {
    pub slots: [SlotInfo; 9],
}

impl SpellSlots {
    pub fn new() -> Self {
        Self {
            slots: std::array::from_fn(|_| SlotInfo { total: 0, used: 0 }),
        }
    }

    pub fn use_slot(&mut self, level: u8) -> bool {
        if (1..=9).contains(&level) {
            let slot = &mut self.slots[level as usize - 1];
            if slot.available() > 0 {
                slot.used += 1;
                return true;
            }
        }
        false
    }

    pub fn recover_all(&mut self) {
        for slot in &mut self.slots {
            slot.used = 0;
        }
    }
}

impl Default for SpellSlots {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SlotInfo {
    pub total: u8,
    pub used: u8,
}

impl SlotInfo {
    pub fn available(&self) -> u8 {
        self.total.saturating_sub(self.used)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========== SlotInfo Tests ==========

    #[test]
    fn test_slot_info_available_basic() {
        let slot = SlotInfo { total: 4, used: 1 };
        assert_eq!(slot.available(), 3);
    }

    #[test]
    fn test_slot_info_available_none_used() {
        let slot = SlotInfo { total: 4, used: 0 };
        assert_eq!(slot.available(), 4);
    }

    #[test]
    fn test_slot_info_available_all_used() {
        let slot = SlotInfo { total: 4, used: 4 };
        assert_eq!(slot.available(), 0);
    }

    #[test]
    fn test_slot_info_available_saturating() {
        // Edge case: used > total (shouldn't happen, but handle gracefully)
        let slot = SlotInfo { total: 2, used: 5 };
        assert_eq!(slot.available(), 0); // Saturating prevents underflow
    }

    // ========== SpellSlots Tests ==========

    #[test]
    fn test_spell_slots_new() {
        let slots = SpellSlots::new();
        for i in 0..9 {
            assert_eq!(slots.slots[i].total, 0);
            assert_eq!(slots.slots[i].used, 0);
        }
    }

    #[test]
    fn test_spell_slots_default() {
        let slots = SpellSlots::default();
        for i in 0..9 {
            assert_eq!(slots.slots[i].total, 0);
        }
    }

    #[test]
    fn test_spell_slots_use_slot_success() {
        let mut slots = SpellSlots::new();
        slots.slots[0] = SlotInfo { total: 4, used: 0 }; // Level 1

        assert!(slots.use_slot(1));
        assert_eq!(slots.slots[0].used, 1);
        assert_eq!(slots.slots[0].available(), 3);
    }

    #[test]
    fn test_spell_slots_use_slot_failure_no_slots() {
        let mut slots = SpellSlots::new();
        // Level 1 has 0 total slots

        assert!(!slots.use_slot(1));
    }

    #[test]
    fn test_spell_slots_use_slot_failure_all_used() {
        let mut slots = SpellSlots::new();
        slots.slots[0] = SlotInfo { total: 2, used: 2 }; // Level 1, all used

        assert!(!slots.use_slot(1));
    }

    #[test]
    fn test_spell_slots_use_slot_invalid_level() {
        let mut slots = SpellSlots::new();

        assert!(!slots.use_slot(0)); // Level 0 is invalid
        assert!(!slots.use_slot(10)); // Level 10 is invalid
    }

    #[test]
    fn test_spell_slots_use_slot_all_levels() {
        let mut slots = SpellSlots::new();

        // Set up slots for all levels
        for i in 0..9 {
            slots.slots[i] = SlotInfo {
                total: (i + 1) as u8,
                used: 0,
            };
        }

        // Use a slot at each level
        for level in 1..=9 {
            assert!(slots.use_slot(level));
        }

        // Verify usage
        for i in 0..9 {
            assert_eq!(slots.slots[i].used, 1);
        }
    }

    #[test]
    fn test_spell_slots_recover_all() {
        let mut slots = SpellSlots::new();

        // Set up typical wizard spell slots (level 5)
        slots.slots[0] = SlotInfo { total: 4, used: 4 }; // 1st level
        slots.slots[1] = SlotInfo { total: 3, used: 3 }; // 2nd level
        slots.slots[2] = SlotInfo { total: 2, used: 2 }; // 3rd level

        slots.recover_all();

        assert_eq!(slots.slots[0].used, 0);
        assert_eq!(slots.slots[0].available(), 4);
        assert_eq!(slots.slots[1].used, 0);
        assert_eq!(slots.slots[1].available(), 3);
        assert_eq!(slots.slots[2].used, 0);
        assert_eq!(slots.slots[2].available(), 2);
    }

    // ========== SpellcastingData Tests ==========

    #[test]
    fn test_spell_save_dc_wizard() {
        let data = SpellcastingData {
            ability: Ability::Intelligence,
            spells_known: vec![],
            spells_prepared: vec!["Fireball".to_string()],
            cantrips_known: vec!["Fire Bolt".to_string()],
            spell_slots: SpellSlots::new(),
        };

        let scores = AbilityScores::new(10, 10, 10, 18, 10, 10); // INT 18 (+4)
        let proficiency = 3; // Level 5-8

        let dc = data.spell_save_dc(&scores, proficiency);
        assert_eq!(dc, 15); // 8 + 3 + 4 = 15
    }

    #[test]
    fn test_spell_save_dc_cleric() {
        let data = SpellcastingData {
            ability: Ability::Wisdom,
            spells_known: vec![],
            spells_prepared: vec!["Guiding Bolt".to_string()],
            cantrips_known: vec!["Sacred Flame".to_string()],
            spell_slots: SpellSlots::new(),
        };

        let scores = AbilityScores::new(10, 10, 10, 10, 16, 10); // WIS 16 (+3)
        let proficiency = 2; // Level 1-4

        let dc = data.spell_save_dc(&scores, proficiency);
        assert_eq!(dc, 13); // 8 + 2 + 3 = 13
    }

    #[test]
    fn test_spell_save_dc_minimum() {
        let data = SpellcastingData {
            ability: Ability::Charisma,
            spells_known: vec![],
            spells_prepared: vec![],
            cantrips_known: vec![],
            spell_slots: SpellSlots::new(),
        };

        // Extremely low stats (shouldn't happen in practice but test bounds)
        let scores = AbilityScores::new(1, 1, 1, 1, 1, 1); // All stats 1 (-5)
        let proficiency = 2;

        let dc = data.spell_save_dc(&scores, proficiency);
        assert_eq!(dc, 5); // 8 + 2 + (-5) = 5, but clamped to at least 0
    }

    #[test]
    fn test_spell_attack_bonus_basic() {
        let data = SpellcastingData {
            ability: Ability::Intelligence,
            spells_known: vec![],
            spells_prepared: vec![],
            cantrips_known: vec![],
            spell_slots: SpellSlots::new(),
        };

        let scores = AbilityScores::new(10, 10, 10, 16, 10, 10); // INT 16 (+3)
        let proficiency = 2;

        let bonus = data.spell_attack_bonus(&scores, proficiency);
        assert_eq!(bonus, 5); // 3 + 2 = 5
    }

    #[test]
    fn test_spell_attack_bonus_high_level() {
        let data = SpellcastingData {
            ability: Ability::Wisdom,
            spells_known: vec![],
            spells_prepared: vec![],
            cantrips_known: vec![],
            spell_slots: SpellSlots::new(),
        };

        let scores = AbilityScores::new(10, 10, 10, 10, 20, 10); // WIS 20 (+5)
        let proficiency = 6; // Level 17+

        let bonus = data.spell_attack_bonus(&scores, proficiency);
        assert_eq!(bonus, 11); // 5 + 6 = 11
    }

    #[test]
    fn test_spell_attack_bonus_negative_modifier() {
        let data = SpellcastingData {
            ability: Ability::Charisma,
            spells_known: vec![],
            spells_prepared: vec![],
            cantrips_known: vec![],
            spell_slots: SpellSlots::new(),
        };

        let scores = AbilityScores::new(10, 10, 10, 10, 10, 8); // CHA 8 (-1)
        let proficiency = 2;

        let bonus = data.spell_attack_bonus(&scores, proficiency);
        assert_eq!(bonus, 1); // -1 + 2 = 1
    }
}
