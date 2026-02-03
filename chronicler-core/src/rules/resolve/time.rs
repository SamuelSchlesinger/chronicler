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
