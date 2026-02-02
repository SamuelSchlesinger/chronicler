//! Application state and AI worker integration.
//!
//! This module provides the GameState resource that holds all mutable
//! application state, and integrates with the async AI worker for
//! processing player actions.

mod app_state;
mod game_phase;
mod onboarding;
mod save_lists;
mod systems;
mod worker;
mod world_update;

// Re-export all public types
pub use app_state::{ActiveOverlay, AppState};
pub use game_phase::GamePhase;
pub use onboarding::OnboardingState;
pub use save_lists::{
    CharacterSaveList, GameSaveInfo, GameSaveList, PendingCharacterList, PendingGameList,
    PendingGameLoad, PendingSession,
};
pub use systems::{
    check_pending_character_list, check_pending_game_list, check_pending_game_load,
    check_pending_session, clear_old_status, handle_worker_responses, process_pending_sounds,
};
pub use worker::{spawn_worker, WorkerRequest, WorkerResponse};
pub use world_update::WorldUpdate;
