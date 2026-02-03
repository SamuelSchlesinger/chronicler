//! Time-related resolution methods (rests, time advancement).

use crate::rules::types::{Effect, Resolution, RestType};
use crate::rules::RulesEngine;
use crate::world::GameWorld;

impl RulesEngine {
    pub(crate) fn resolve_short_rest(&self, world: &GameWorld) -> Resolution {
        // Can't rest during combat
        if world.combat.is_some() {
            return Resolution::new("Cannot take a short rest while in combat!");
        }

        Resolution::new("The party takes a short rest, spending 1 hour resting.")
            .with_effect(Effect::TimeAdvanced { minutes: 60 })
            .with_effect(Effect::RestCompleted {
                rest_type: RestType::Short,
            })
    }

    pub(crate) fn resolve_long_rest(&self, world: &GameWorld) -> Resolution {
        // Can't rest during combat
        if world.combat.is_some() {
            return Resolution::new("Cannot take a long rest while in combat!");
        }

        Resolution::new("The party takes a long rest, spending 8 hours resting.")
            .with_effect(Effect::TimeAdvanced { minutes: 480 })
            .with_effect(Effect::RestCompleted {
                rest_type: RestType::Long,
            })
    }

    pub(crate) fn resolve_advance_time(&self, minutes: u32) -> Resolution {
        let hours = minutes / 60;
        let mins = minutes % 60;

        let time_str = if hours > 0 && mins > 0 {
            format!("{hours} hours and {mins} minutes")
        } else if hours > 0 {
            format!("{hours} hours")
        } else {
            format!("{mins} minutes")
        };

        Resolution::new(format!("{time_str} pass.")).with_effect(Effect::TimeAdvanced { minutes })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::types::Effect;
    use crate::world::{create_sample_fighter, CharacterId, CombatState, Combatant, GameWorld};

    // ========== Short Rest Tests ==========

    #[test]
    fn test_short_rest_success() {
        let character = create_sample_fighter("Roland");
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution = engine.resolve_short_rest(&world);

        assert!(resolution.narrative.contains("short rest"));
        assert!(resolution.narrative.contains("1 hour"));
        assert!(resolution
            .effects
            .iter()
            .any(|e| matches!(e, Effect::TimeAdvanced { minutes: 60 })));
        assert!(resolution.effects.iter().any(|e| matches!(
            e,
            Effect::RestCompleted {
                rest_type: RestType::Short
            }
        )));
    }

    #[test]
    fn test_short_rest_during_combat() {
        let character = create_sample_fighter("Roland");
        let mut world = GameWorld::new("Test", character);
        world.combat = Some(CombatState {
            active: true,
            round: 1,
            turn_index: 0,
            combatants: vec![Combatant {
                id: CharacterId::new(),
                name: "Roland".to_string(),
                initiative: 15,
                is_player: true,
                is_ally: true,
                current_hp: 20,
                max_hp: 20,
                armor_class: 16,
            }],
            sneak_attack_used: std::collections::HashSet::new(),
            attacks_this_turn: std::collections::HashMap::new(),
        });
        let engine = RulesEngine::new();

        let resolution = engine.resolve_short_rest(&world);

        assert!(resolution.narrative.contains("Cannot take a short rest"));
        assert!(resolution.narrative.contains("combat"));
        assert!(resolution.effects.is_empty());
    }

    // ========== Long Rest Tests ==========

    #[test]
    fn test_long_rest_success() {
        let character = create_sample_fighter("Roland");
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let resolution = engine.resolve_long_rest(&world);

        assert!(resolution.narrative.contains("long rest"));
        assert!(resolution.narrative.contains("8 hours"));
        assert!(resolution
            .effects
            .iter()
            .any(|e| matches!(e, Effect::TimeAdvanced { minutes: 480 })));
        assert!(resolution.effects.iter().any(|e| matches!(
            e,
            Effect::RestCompleted {
                rest_type: RestType::Long
            }
        )));
    }

    #[test]
    fn test_long_rest_during_combat() {
        let character = create_sample_fighter("Roland");
        let mut world = GameWorld::new("Test", character);
        world.combat = Some(CombatState {
            active: true,
            round: 1,
            turn_index: 0,
            combatants: vec![Combatant {
                id: CharacterId::new(),
                name: "Roland".to_string(),
                initiative: 15,
                is_player: true,
                is_ally: true,
                current_hp: 20,
                max_hp: 20,
                armor_class: 16,
            }],
            sneak_attack_used: std::collections::HashSet::new(),
            attacks_this_turn: std::collections::HashMap::new(),
        });
        let engine = RulesEngine::new();

        let resolution = engine.resolve_long_rest(&world);

        assert!(resolution.narrative.contains("Cannot take a long rest"));
        assert!(resolution.narrative.contains("combat"));
        assert!(resolution.effects.is_empty());
    }

    // ========== Advance Time Tests ==========

    #[test]
    fn test_advance_time_minutes_only() {
        let engine = RulesEngine::new();

        let resolution = engine.resolve_advance_time(30);

        assert!(resolution.narrative.contains("30 minutes"));
        assert!(resolution
            .effects
            .iter()
            .any(|e| matches!(e, Effect::TimeAdvanced { minutes: 30 })));
    }

    #[test]
    fn test_advance_time_hours_only() {
        let engine = RulesEngine::new();

        let resolution = engine.resolve_advance_time(120);

        assert!(resolution.narrative.contains("2 hours"));
        assert!(!resolution.narrative.contains("minutes"));
        assert!(resolution
            .effects
            .iter()
            .any(|e| matches!(e, Effect::TimeAdvanced { minutes: 120 })));
    }

    #[test]
    fn test_advance_time_hours_and_minutes() {
        let engine = RulesEngine::new();

        let resolution = engine.resolve_advance_time(90);

        assert!(resolution.narrative.contains("1 hours"));
        assert!(resolution.narrative.contains("30 minutes"));
        assert!(resolution
            .effects
            .iter()
            .any(|e| matches!(e, Effect::TimeAdvanced { minutes: 90 })));
    }
}
