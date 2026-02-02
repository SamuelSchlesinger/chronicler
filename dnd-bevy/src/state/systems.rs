//! Bevy systems for state management.

use bevy::prelude::*;
use dnd_core::world::NarrativeType;

use super::{
    spawn_worker, ActiveOverlay, AppState, CharacterSaveList, GamePhase, GameSaveList,
    PendingCharacterList, PendingGameList, PendingGameLoad, PendingSession, WorkerResponse,
};

/// System to process pending UI sounds.
pub fn process_pending_sounds(
    mut app_state: ResMut<AppState>,
    mut sound_writer: EventWriter<crate::sound::SoundEffect>,
) {
    for sound in app_state.pending_sounds.drain(..) {
        sound_writer.send(sound);
    }
}

/// System to clear old status messages after 3 seconds.
pub fn clear_old_status(mut app_state: ResMut<AppState>, time: Res<Time>) {
    if let Some(set_time) = app_state.status_set_time {
        let elapsed = time.elapsed_secs_f64() - set_time;
        if elapsed > 3.0 {
            app_state.clear_status();
        }
    }
}

/// System to handle responses from the AI worker.
pub fn handle_worker_responses(
    mut app_state: ResMut<AppState>,
    time: Res<Time>,
    mut commands: Commands,
    mut sound_writer: EventWriter<crate::sound::SoundEffect>,
) {
    // Take the receiver temporarily to check for messages
    let response = if let Some(rx) = &mut app_state.response_rx {
        rx.try_recv().ok()
    } else {
        None
    };

    if let Some(response) = response {
        match response {
            WorkerResponse::StreamChunk(text) => {
                app_state.streaming_text.push_str(&text);
            }
            WorkerResponse::Effect(effect) => {
                crate::effects::process_effect(
                    &mut app_state,
                    &effect,
                    &mut commands,
                    time.elapsed_secs_f64(),
                    &mut sound_writer,
                );
            }
            WorkerResponse::Complete {
                narrative,
                effects: _,
                world_update,
                in_combat,
                is_player_turn,
            } => {
                // Add the complete narrative
                if !narrative.is_empty() {
                    app_state.add_narrative(
                        narrative,
                        NarrativeType::DmNarration,
                        time.elapsed_secs_f64(),
                    );
                }
                app_state.streaming_text.clear();
                app_state.world = world_update;
                app_state.in_combat = in_combat;
                app_state.is_player_turn = is_player_turn;
                app_state.is_processing = false;
            }
            WorkerResponse::Cancelled => {
                app_state.is_processing = false;
                app_state.streaming_text.clear();
            }
            WorkerResponse::Error(err) => {
                app_state.error_message = Some(err);
                app_state.is_processing = false;
            }
            WorkerResponse::SaveComplete(result) => {
                app_state.is_saving = false;
                match result {
                    Ok(path) => {
                        app_state.set_status(format!("Saved to {path:?}"), time.elapsed_secs_f64());
                    }
                    Err(e) => {
                        app_state.error_message = Some(format!("Save failed: {e}"));
                    }
                }
            }
            WorkerResponse::LoadComplete(result) => {
                app_state.is_loading = false;
                match result {
                    Ok(world_update) => {
                        app_state.world = world_update;
                        app_state.set_status("Game loaded", time.elapsed_secs_f64());
                    }
                    Err(e) => {
                        app_state.error_message = Some(format!("Load failed: {e}"));
                    }
                }
            }
        }
    }
}

/// System to check for pending game list load.
pub fn check_pending_game_list(
    mut commands: Commands,
    pending: Option<Res<PendingGameList>>,
    mut save_list: Option<ResMut<GameSaveList>>,
) {
    let Some(pending) = pending else { return };
    let Some(ref mut list) = save_list else {
        return;
    };

    let result = {
        let receiver = pending
            .receiver
            .lock()
            .expect("game list receiver mutex poisoned");
        receiver.try_recv()
    };

    match result {
        Ok(Ok(saves)) => {
            list.saves = saves;
            list.loading = false;
            list.loaded = true;
            commands.remove_resource::<PendingGameList>();
        }
        Ok(Err(e)) => {
            list.error = Some(e);
            list.loading = false;
            list.loaded = true;
            commands.remove_resource::<PendingGameList>();
        }
        Err(std::sync::mpsc::TryRecvError::Empty) => {}
        Err(std::sync::mpsc::TryRecvError::Disconnected) => {
            list.error = Some("Game list load failed unexpectedly".to_string());
            list.loading = false;
            list.loaded = true;
            commands.remove_resource::<PendingGameList>();
        }
    }
}

/// System to check for pending game load and start the session.
pub fn check_pending_game_load(
    mut commands: Commands,
    pending: Option<Res<PendingGameLoad>>,
    mut app_state: ResMut<AppState>,
    mut next_phase: ResMut<NextState<GamePhase>>,
) {
    let Some(pending) = pending else { return };

    let result = {
        let receiver = pending
            .receiver
            .lock()
            .expect("game load receiver mutex poisoned");
        receiver.try_recv()
    };

    match result {
        Ok(Ok(session)) => {
            // Session loaded successfully - spawn the worker
            let (request_tx, response_rx, initial_world) = spawn_worker(session);
            app_state.request_tx = Some(request_tx);
            app_state.response_rx = Some(response_rx);
            app_state.world = initial_world;
            app_state.set_status_persistent("Game loaded!");
            app_state.overlay = ActiveOverlay::None;

            // Transition to playing
            next_phase.set(GamePhase::Playing);
            commands.remove_resource::<PendingGameLoad>();
        }
        Ok(Err(e)) => {
            app_state.error_message = Some(format!("Failed to load game: {e}"));
            commands.remove_resource::<PendingGameLoad>();
        }
        Err(std::sync::mpsc::TryRecvError::Empty) => {}
        Err(std::sync::mpsc::TryRecvError::Disconnected) => {
            app_state.error_message = Some("Game load failed unexpectedly".to_string());
            commands.remove_resource::<PendingGameLoad>();
        }
    }
}

/// System to check for pending character list load.
pub fn check_pending_character_list(
    mut commands: Commands,
    pending: Option<Res<PendingCharacterList>>,
    mut save_list: Option<ResMut<CharacterSaveList>>,
) {
    let Some(pending) = pending else { return };
    let Some(ref mut list) = save_list else {
        return;
    };

    let result = {
        let receiver = pending
            .receiver
            .lock()
            .expect("character list receiver mutex poisoned");
        receiver.try_recv()
    };

    match result {
        Ok(Ok(saves)) => {
            list.saves = saves;
            list.loading = false;
            list.loaded = true;
            commands.remove_resource::<PendingCharacterList>();
        }
        Ok(Err(e)) => {
            list.error = Some(format!("Failed to load saves: {e}"));
            list.loading = false;
            list.loaded = true;
            commands.remove_resource::<PendingCharacterList>();
        }
        Err(std::sync::mpsc::TryRecvError::Empty) => {
            // Still loading
        }
        Err(std::sync::mpsc::TryRecvError::Disconnected) => {
            list.error = Some("Character list load failed unexpectedly".to_string());
            list.loading = false;
            list.loaded = true;
            commands.remove_resource::<PendingCharacterList>();
        }
    }
}

/// System to check for pending session creation and connect it when ready.
pub fn check_pending_session(
    mut commands: Commands,
    pending: Option<Res<PendingSession>>,
    mut app_state: ResMut<AppState>,
) {
    let Some(pending) = pending else { return };

    // Try to receive without blocking
    let result = {
        let receiver = pending
            .receiver
            .lock()
            .expect("session receiver mutex poisoned");
        receiver.try_recv()
    };

    match result {
        Ok(Ok(session)) => {
            // Session created successfully - spawn the worker
            let (request_tx, response_rx, initial_world) = spawn_worker(session);
            app_state.request_tx = Some(request_tx);
            app_state.response_rx = Some(response_rx);
            app_state.world = initial_world;
            app_state.set_status_persistent("Adventure begins!");

            // Send initial action to get the DM's opening narration
            app_state.send_action(
                "I begin my adventure. Set the scene and describe where I am.".to_string(),
            );

            // Remove the pending resource
            commands.remove_resource::<PendingSession>();
        }
        Ok(Err(e)) => {
            app_state.error_message = Some(format!("Failed to create session: {e}"));
            commands.remove_resource::<PendingSession>();
        }
        Err(std::sync::mpsc::TryRecvError::Empty) => {
            // Still waiting
        }
        Err(std::sync::mpsc::TryRecvError::Disconnected) => {
            app_state.error_message = Some("Session creation failed unexpectedly".to_string());
            commands.remove_resource::<PendingSession>();
        }
    }
}
