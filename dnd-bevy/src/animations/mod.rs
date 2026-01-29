//! Animation systems for visual feedback.
//!
//! This module provides:
//! - Dice rolling animations
//! - Floating damage numbers
//! - Combat effects (screen shake, flashes)

pub mod damage;
pub mod dice;
pub mod effects;

pub use damage::{animate_damage_numbers, spawn_damage_number};
pub use dice::{animate_dice, spawn_dice_animation};
pub use effects::{animate_combat_effects, spawn_combat_effect};

use bevy::prelude::*;

/// Marker component for animations that should be cleaned up when finished.
#[derive(Component)]
pub struct AnimationLifetime {
    pub remaining: f32,
}

/// System to clean up finished animations.
pub fn cleanup_finished_animations(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut AnimationLifetime)>,
) {
    for (entity, mut lifetime) in query.iter_mut() {
        lifetime.remaining -= time.delta_secs();
        if lifetime.remaining <= 0.0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}
