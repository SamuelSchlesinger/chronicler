//! Hit points and health-related types.
//!
//! Contains types for tracking hit points, hit dice, and death saving throws.

use crate::dice::DieType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Hit Points and Health
// ============================================================================

/// Hit points tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HitPoints {
    pub current: i32,
    pub maximum: i32,
    pub temporary: i32,
}

impl HitPoints {
    pub fn new(maximum: i32) -> Self {
        Self {
            current: maximum,
            maximum,
            temporary: 0,
        }
    }

    pub fn take_damage(&mut self, amount: i32) -> DamageResult {
        let mut remaining = amount;

        if self.temporary > 0 {
            if self.temporary >= remaining {
                self.temporary -= remaining;
                return DamageResult {
                    damage_taken: amount,
                    dropped_to_zero: false,
                };
            } else {
                remaining -= self.temporary;
                self.temporary = 0;
            }
        }

        self.current -= remaining;
        DamageResult {
            damage_taken: amount,
            dropped_to_zero: self.current <= 0,
        }
    }

    pub fn heal(&mut self, amount: i32) -> i32 {
        let old = self.current;
        self.current = (self.current + amount).min(self.maximum);
        self.current - old
    }

    pub fn add_temp_hp(&mut self, amount: i32) {
        self.temporary = self.temporary.max(amount);
    }

    pub fn is_unconscious(&self) -> bool {
        self.current <= 0
    }

    pub fn ratio(&self) -> f32 {
        (self.current as f32 / self.maximum as f32).max(0.0)
    }
}

/// Result of taking damage.
#[derive(Debug, Clone)]
pub struct DamageResult {
    pub damage_taken: i32,
    pub dropped_to_zero: bool,
}

/// Hit dice tracking.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HitDice {
    pub total: HashMap<DieType, u8>,
    pub remaining: HashMap<DieType, u8>,
}

impl HitDice {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, die_type: DieType, count: u8) {
        *self.total.entry(die_type).or_insert(0) += count;
        *self.remaining.entry(die_type).or_insert(0) += count;
    }

    pub fn spend(&mut self, die_type: DieType) -> bool {
        if let Some(remaining) = self.remaining.get_mut(&die_type) {
            if *remaining > 0 {
                *remaining -= 1;
                return true;
            }
        }
        false
    }

    pub fn recover_half(&mut self) {
        for (die_type, total) in &self.total {
            let to_recover = (*total as f32 / 2.0).ceil() as u8;
            if let Some(remaining) = self.remaining.get_mut(die_type) {
                *remaining = (*remaining + to_recover).min(*total);
            }
        }
    }
}

/// Death saving throws.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeathSaves {
    pub successes: u8,
    pub failures: u8,
}

impl DeathSaves {
    pub fn add_success(&mut self) -> bool {
        self.successes += 1;
        self.successes >= 3
    }

    pub fn add_failure(&mut self) -> bool {
        self.failures += 1;
        self.failures >= 3
    }

    pub fn reset(&mut self) {
        self.successes = 0;
        self.failures = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========== HitPoints Tests ==========

    #[test]
    fn test_hit_points_new() {
        let hp = HitPoints::new(20);
        assert_eq!(hp.current, 20);
        assert_eq!(hp.maximum, 20);
        assert_eq!(hp.temporary, 0);
    }

    #[test]
    fn test_take_damage_basic() {
        let mut hp = HitPoints::new(20);
        let result = hp.take_damage(5);

        assert_eq!(hp.current, 15);
        assert_eq!(result.damage_taken, 5);
        assert!(!result.dropped_to_zero);
    }

    #[test]
    fn test_take_damage_to_zero() {
        let mut hp = HitPoints::new(10);
        let result = hp.take_damage(10);

        assert_eq!(hp.current, 0);
        assert!(result.dropped_to_zero);
    }

    #[test]
    fn test_take_damage_below_zero() {
        let mut hp = HitPoints::new(10);
        let result = hp.take_damage(15);

        assert_eq!(hp.current, -5);
        assert!(result.dropped_to_zero);
    }

    #[test]
    fn test_take_damage_with_temp_hp_absorbs_all() {
        let mut hp = HitPoints::new(20);
        hp.add_temp_hp(10);

        let result = hp.take_damage(5);

        assert_eq!(hp.temporary, 5); // 10 - 5 = 5 temp HP remaining
        assert_eq!(hp.current, 20); // No damage to real HP
        assert_eq!(result.damage_taken, 5);
        assert!(!result.dropped_to_zero);
    }

    #[test]
    fn test_take_damage_with_temp_hp_partial() {
        let mut hp = HitPoints::new(20);
        hp.add_temp_hp(5);

        let result = hp.take_damage(8);

        assert_eq!(hp.temporary, 0); // All temp HP used
        assert_eq!(hp.current, 17); // 20 - (8 - 5) = 17
        assert_eq!(result.damage_taken, 8);
        assert!(!result.dropped_to_zero);
    }

    #[test]
    fn test_take_damage_with_temp_hp_exactly() {
        let mut hp = HitPoints::new(20);
        hp.add_temp_hp(10);

        let result = hp.take_damage(10);

        assert_eq!(hp.temporary, 0);
        assert_eq!(hp.current, 20);
        assert!(!result.dropped_to_zero);
    }

    #[test]
    fn test_heal_basic() {
        let mut hp = HitPoints::new(20);
        hp.current = 10;

        let healed = hp.heal(5);

        assert_eq!(hp.current, 15);
        assert_eq!(healed, 5);
    }

    #[test]
    fn test_heal_capped_at_max() {
        let mut hp = HitPoints::new(20);
        hp.current = 18;

        let healed = hp.heal(10);

        assert_eq!(hp.current, 20); // Capped at maximum
        assert_eq!(healed, 2); // Only actually healed 2
    }

    #[test]
    fn test_heal_at_full() {
        let mut hp = HitPoints::new(20);

        let healed = hp.heal(10);

        assert_eq!(hp.current, 20);
        assert_eq!(healed, 0); // No healing applied
    }

    #[test]
    fn test_heal_from_negative() {
        let mut hp = HitPoints::new(20);
        hp.current = -5;

        let healed = hp.heal(10);

        assert_eq!(hp.current, 5);
        assert_eq!(healed, 10);
    }

    #[test]
    fn test_add_temp_hp_replaces_lower() {
        let mut hp = HitPoints::new(20);
        hp.add_temp_hp(5);
        assert_eq!(hp.temporary, 5);

        hp.add_temp_hp(10);
        assert_eq!(hp.temporary, 10); // Higher value replaces
    }

    #[test]
    fn test_add_temp_hp_keeps_higher() {
        let mut hp = HitPoints::new(20);
        hp.add_temp_hp(10);
        assert_eq!(hp.temporary, 10);

        hp.add_temp_hp(5);
        assert_eq!(hp.temporary, 10); // Keep the higher value
    }

    #[test]
    fn test_is_unconscious() {
        let mut hp = HitPoints::new(20);
        assert!(!hp.is_unconscious());

        hp.current = 0;
        assert!(hp.is_unconscious());

        hp.current = -5;
        assert!(hp.is_unconscious());
    }

    #[test]
    fn test_ratio() {
        let mut hp = HitPoints::new(20);
        assert!((hp.ratio() - 1.0).abs() < f32::EPSILON);

        hp.current = 10;
        assert!((hp.ratio() - 0.5).abs() < f32::EPSILON);

        hp.current = 0;
        assert!((hp.ratio() - 0.0).abs() < f32::EPSILON);

        hp.current = -5;
        assert!((hp.ratio() - 0.0).abs() < f32::EPSILON); // Clamped to 0
    }

    // ========== HitDice Tests ==========

    #[test]
    fn test_hit_dice_new() {
        let hd = HitDice::new();
        assert!(hd.total.is_empty());
        assert!(hd.remaining.is_empty());
    }

    #[test]
    fn test_hit_dice_add() {
        let mut hd = HitDice::new();
        hd.add(DieType::D10, 5);

        assert_eq!(hd.total.get(&DieType::D10), Some(&5));
        assert_eq!(hd.remaining.get(&DieType::D10), Some(&5));
    }

    #[test]
    fn test_hit_dice_add_multiple() {
        let mut hd = HitDice::new();
        hd.add(DieType::D10, 3);
        hd.add(DieType::D10, 2);

        assert_eq!(hd.total.get(&DieType::D10), Some(&5));
        assert_eq!(hd.remaining.get(&DieType::D10), Some(&5));
    }

    #[test]
    fn test_hit_dice_add_different_types() {
        let mut hd = HitDice::new();
        hd.add(DieType::D8, 3);
        hd.add(DieType::D10, 2);

        assert_eq!(hd.total.get(&DieType::D8), Some(&3));
        assert_eq!(hd.total.get(&DieType::D10), Some(&2));
    }

    #[test]
    fn test_hit_dice_spend() {
        let mut hd = HitDice::new();
        hd.add(DieType::D10, 5);

        assert!(hd.spend(DieType::D10));
        assert_eq!(hd.remaining.get(&DieType::D10), Some(&4));
        assert_eq!(hd.total.get(&DieType::D10), Some(&5)); // Total unchanged
    }

    #[test]
    fn test_hit_dice_spend_all() {
        let mut hd = HitDice::new();
        hd.add(DieType::D10, 2);

        assert!(hd.spend(DieType::D10));
        assert!(hd.spend(DieType::D10));
        assert!(!hd.spend(DieType::D10)); // Can't spend when none remaining

        assert_eq!(hd.remaining.get(&DieType::D10), Some(&0));
    }

    #[test]
    fn test_hit_dice_spend_wrong_type() {
        let mut hd = HitDice::new();
        hd.add(DieType::D10, 5);

        assert!(!hd.spend(DieType::D8)); // Don't have D8s
    }

    #[test]
    fn test_hit_dice_recover_half() {
        let mut hd = HitDice::new();
        hd.add(DieType::D10, 10);

        // Spend 8 dice
        for _ in 0..8 {
            hd.spend(DieType::D10);
        }
        assert_eq!(hd.remaining.get(&DieType::D10), Some(&2));

        // Recover half (5 dice, rounded up)
        hd.recover_half();
        assert_eq!(hd.remaining.get(&DieType::D10), Some(&7)); // 2 + 5 = 7
    }

    #[test]
    fn test_hit_dice_recover_half_rounds_up() {
        let mut hd = HitDice::new();
        hd.add(DieType::D10, 5); // 5 total, recover ceil(5/2) = 3

        for _ in 0..5 {
            hd.spend(DieType::D10);
        }
        assert_eq!(hd.remaining.get(&DieType::D10), Some(&0));

        hd.recover_half();
        assert_eq!(hd.remaining.get(&DieType::D10), Some(&3)); // Rounded up
    }

    #[test]
    fn test_hit_dice_recover_capped_at_total() {
        let mut hd = HitDice::new();
        hd.add(DieType::D10, 5);
        hd.spend(DieType::D10); // Only spent 1

        hd.recover_half();
        assert_eq!(hd.remaining.get(&DieType::D10), Some(&5)); // Can't exceed total
    }

    // ========== DeathSaves Tests ==========

    #[test]
    fn test_death_saves_default() {
        let ds = DeathSaves::default();
        assert_eq!(ds.successes, 0);
        assert_eq!(ds.failures, 0);
    }

    #[test]
    fn test_death_saves_add_success() {
        let mut ds = DeathSaves::default();

        assert!(!ds.add_success()); // 1 success
        assert!(!ds.add_success()); // 2 successes
        assert!(ds.add_success()); // 3 successes - stabilized!

        assert_eq!(ds.successes, 3);
    }

    #[test]
    fn test_death_saves_add_failure() {
        let mut ds = DeathSaves::default();

        assert!(!ds.add_failure()); // 1 failure
        assert!(!ds.add_failure()); // 2 failures
        assert!(ds.add_failure()); // 3 failures - dead!

        assert_eq!(ds.failures, 3);
    }

    #[test]
    fn test_death_saves_reset() {
        let mut ds = DeathSaves::default();
        ds.add_success();
        ds.add_success();
        ds.add_failure();

        ds.reset();

        assert_eq!(ds.successes, 0);
        assert_eq!(ds.failures, 0);
    }

    #[test]
    fn test_death_saves_mixed() {
        let mut ds = DeathSaves::default();

        ds.add_success();
        ds.add_failure();
        ds.add_success();
        ds.add_failure();

        assert_eq!(ds.successes, 2);
        assert_eq!(ds.failures, 2);

        // Adding 3rd success stabilizes
        assert!(ds.add_success());
        assert_eq!(ds.successes, 3);
    }
}
