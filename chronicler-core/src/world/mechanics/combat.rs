//! Combat mechanics for D&D 5e.
//!
//! Implements combat state transitions including:
//! - Starting combat (entering combat mode)
//! - Ending combat (returning to exploration mode)
//! - Turn management

use crate::world::{CombatState, GameMode, GameWorld};

/// Start combat, transitioning the game to combat mode.
///
/// Returns a mutable reference to the newly created combat state.
pub fn start_combat(world: &mut GameWorld) -> &mut CombatState {
    world.mode = GameMode::Combat;
    world.combat.insert(CombatState::new())
}

/// End combat, transitioning the game back to exploration mode.
///
/// Clears the combat state.
pub fn end_combat(world: &mut GameWorld) {
    world.combat = None;
    world.mode = GameMode::Exploration;
}

/// Advance to the next turn in combat.
///
/// Returns the index of the new current combatant, or None if combat is not active.
pub fn next_turn(world: &mut GameWorld) -> Option<usize> {
    if let Some(ref mut combat) = world.combat {
        combat.next_turn();
        Some(combat.turn_index)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::create_sample_fighter;

    #[test]
    fn test_start_combat() {
        let character = create_sample_fighter("Test");
        let mut world = GameWorld::new("Test Campaign", character);

        assert!(matches!(world.mode, GameMode::Exploration));
        assert!(world.combat.is_none());

        start_combat(&mut world);

        assert!(matches!(world.mode, GameMode::Combat));
        assert!(world.combat.is_some());
    }

    #[test]
    fn test_end_combat() {
        let character = create_sample_fighter("Test");
        let mut world = GameWorld::new("Test Campaign", character);

        start_combat(&mut world);
        assert!(matches!(world.mode, GameMode::Combat));

        end_combat(&mut world);

        assert!(matches!(world.mode, GameMode::Exploration));
        assert!(world.combat.is_none());
    }

    #[test]
    fn test_next_turn() {
        let character = create_sample_fighter("Test");
        let mut world = GameWorld::new("Test Campaign", character);

        // No combat active
        assert!(next_turn(&mut world).is_none());

        // Start combat
        start_combat(&mut world);

        // Now next_turn should work
        let turn = next_turn(&mut world);
        assert!(turn.is_some());
    }
}
