//! D&D 5e conditions system.
//!
//! This module defines the standard conditions from the SRD 5.2,
//! including effects like Blinded, Charmed, Frightened, etc.

use std::fmt;

use serde::{Deserialize, Serialize};

// ============================================================================
// Conditions
// ============================================================================

/// D&D 5e conditions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Condition {
    Blinded,
    Charmed,
    Deafened,
    Frightened,
    Grappled,
    Incapacitated,
    Invisible,
    Paralyzed,
    Petrified,
    Poisoned,
    Prone,
    Restrained,
    Stunned,
    Unconscious,
    Exhaustion(u8),
}

impl Condition {
    pub fn name(&self) -> &'static str {
        match self {
            Condition::Blinded => "Blinded",
            Condition::Charmed => "Charmed",
            Condition::Deafened => "Deafened",
            Condition::Frightened => "Frightened",
            Condition::Grappled => "Grappled",
            Condition::Incapacitated => "Incapacitated",
            Condition::Invisible => "Invisible",
            Condition::Paralyzed => "Paralyzed",
            Condition::Petrified => "Petrified",
            Condition::Poisoned => "Poisoned",
            Condition::Prone => "Prone",
            Condition::Restrained => "Restrained",
            Condition::Stunned => "Stunned",
            Condition::Unconscious => "Unconscious",
            Condition::Exhaustion(_) => "Exhaustion",
        }
    }

    pub fn is_incapacitating(&self) -> bool {
        matches!(
            self,
            Condition::Incapacitated
                | Condition::Paralyzed
                | Condition::Petrified
                | Condition::Stunned
                | Condition::Unconscious
        )
    }
}

impl fmt::Display for Condition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Condition::Exhaustion(level) => write!(f, "Exhaustion ({level})"),
            _ => write!(f, "{}", self.name()),
        }
    }
}

/// A condition applied to a creature with tracking info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveCondition {
    pub condition: Condition,
    pub source: String,
    pub duration_rounds: Option<u32>,
}

impl ActiveCondition {
    pub fn new(condition: Condition, source: impl Into<String>) -> Self {
        Self {
            condition,
            source: source.into(),
            duration_rounds: None,
        }
    }

    pub fn with_duration(mut self, rounds: u32) -> Self {
        self.duration_rounds = Some(rounds);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========== Condition Tests ==========

    #[test]
    fn test_condition_names() {
        assert_eq!(Condition::Blinded.name(), "Blinded");
        assert_eq!(Condition::Charmed.name(), "Charmed");
        assert_eq!(Condition::Deafened.name(), "Deafened");
        assert_eq!(Condition::Frightened.name(), "Frightened");
        assert_eq!(Condition::Grappled.name(), "Grappled");
        assert_eq!(Condition::Incapacitated.name(), "Incapacitated");
        assert_eq!(Condition::Invisible.name(), "Invisible");
        assert_eq!(Condition::Paralyzed.name(), "Paralyzed");
        assert_eq!(Condition::Petrified.name(), "Petrified");
        assert_eq!(Condition::Poisoned.name(), "Poisoned");
        assert_eq!(Condition::Prone.name(), "Prone");
        assert_eq!(Condition::Restrained.name(), "Restrained");
        assert_eq!(Condition::Stunned.name(), "Stunned");
        assert_eq!(Condition::Unconscious.name(), "Unconscious");
        assert_eq!(Condition::Exhaustion(3).name(), "Exhaustion");
    }

    #[test]
    fn test_condition_is_incapacitating() {
        // Incapacitating conditions
        assert!(Condition::Incapacitated.is_incapacitating());
        assert!(Condition::Paralyzed.is_incapacitating());
        assert!(Condition::Petrified.is_incapacitating());
        assert!(Condition::Stunned.is_incapacitating());
        assert!(Condition::Unconscious.is_incapacitating());

        // Non-incapacitating conditions
        assert!(!Condition::Blinded.is_incapacitating());
        assert!(!Condition::Charmed.is_incapacitating());
        assert!(!Condition::Deafened.is_incapacitating());
        assert!(!Condition::Frightened.is_incapacitating());
        assert!(!Condition::Grappled.is_incapacitating());
        assert!(!Condition::Invisible.is_incapacitating());
        assert!(!Condition::Poisoned.is_incapacitating());
        assert!(!Condition::Prone.is_incapacitating());
        assert!(!Condition::Restrained.is_incapacitating());
        assert!(!Condition::Exhaustion(1).is_incapacitating());
        assert!(!Condition::Exhaustion(5).is_incapacitating());
    }

    #[test]
    fn test_condition_display_basic() {
        assert_eq!(format!("{}", Condition::Blinded), "Blinded");
        assert_eq!(format!("{}", Condition::Poisoned), "Poisoned");
        assert_eq!(format!("{}", Condition::Unconscious), "Unconscious");
    }

    #[test]
    fn test_condition_display_exhaustion() {
        assert_eq!(format!("{}", Condition::Exhaustion(1)), "Exhaustion (1)");
        assert_eq!(format!("{}", Condition::Exhaustion(3)), "Exhaustion (3)");
        assert_eq!(format!("{}", Condition::Exhaustion(6)), "Exhaustion (6)");
    }

    #[test]
    fn test_condition_equality() {
        assert_eq!(Condition::Blinded, Condition::Blinded);
        assert_ne!(Condition::Blinded, Condition::Deafened);

        // Exhaustion levels matter for equality
        assert_eq!(Condition::Exhaustion(2), Condition::Exhaustion(2));
        assert_ne!(Condition::Exhaustion(2), Condition::Exhaustion(3));
    }

    // ========== ActiveCondition Tests ==========

    #[test]
    fn test_active_condition_new() {
        let ac = ActiveCondition::new(Condition::Poisoned, "Giant spider bite");

        assert_eq!(ac.condition, Condition::Poisoned);
        assert_eq!(ac.source, "Giant spider bite");
        assert!(ac.duration_rounds.is_none());
    }

    #[test]
    fn test_active_condition_with_duration() {
        let ac = ActiveCondition::new(Condition::Frightened, "Dragon's presence").with_duration(10);

        assert_eq!(ac.condition, Condition::Frightened);
        assert_eq!(ac.source, "Dragon's presence");
        assert_eq!(ac.duration_rounds, Some(10));
    }

    #[test]
    fn test_active_condition_chained() {
        let ac =
            ActiveCondition::new(Condition::Stunned, "Mindflayer's mind blast").with_duration(3);

        assert_eq!(ac.condition, Condition::Stunned);
        assert_eq!(ac.duration_rounds, Some(3));
    }

    #[test]
    fn test_active_condition_zero_duration() {
        // Edge case: duration of 0 (immediate effect)
        let ac = ActiveCondition::new(Condition::Prone, "Knocked down").with_duration(0);

        assert_eq!(ac.duration_rounds, Some(0));
    }

    #[test]
    fn test_active_condition_source_types() {
        // Test that source accepts different Into<String> types
        let ac1 = ActiveCondition::new(Condition::Blinded, "Spell");
        let ac2 = ActiveCondition::new(Condition::Blinded, String::from("Spell"));

        assert_eq!(ac1.source, ac2.source);
    }
}
