//! Background AI worker for non-blocking DM processing.
//!
//! This module provides a channel-based communication system between
//! the UI thread and the AI worker. The worker owns the GameSession
//! and processes player actions asynchronously, sending responses
//! back through channels.

use dnd_core::rules::Effect;
use dnd_core::world::{
    AbilityScores, CombatState, Condition, DeathSaves, GameMode, GameTime, HitPoints, Item, Quest,
    Skill,
};
use dnd_core::GameSession;
use std::collections::HashMap;
use tokio::sync::mpsc;

/// Request sent from the UI to the AI worker.
#[derive(Debug)]
#[allow(dead_code)] // Shutdown is for future graceful cleanup
pub enum WorkerRequest {
    /// Process a player action.
    PlayerAction(String),
    /// Cancel the current processing.
    Cancel,
    /// Save the game to a file.
    Save(std::path::PathBuf),
    /// Load a game from a file.
    Load(std::path::PathBuf),
    /// Shutdown the worker.
    Shutdown,
}

/// Response sent from the AI worker to the UI.
#[derive(Debug)]
#[allow(dead_code)]
pub enum WorkerResponse {
    /// A chunk of streaming text as it arrives.
    StreamChunk(String),
    /// A game effect to process.
    Effect(Effect),
    /// Processing completed successfully.
    Complete {
        /// The full narrative response.
        narrative: String,
        /// All effects that were applied.
        effects: Vec<Effect>,
        /// Updated world state for rendering.
        world_update: WorldUpdate,
        /// Whether combat is currently active.
        in_combat: bool,
        /// Whether it's the player's turn.
        is_player_turn: bool,
    },
    /// Processing was cancelled.
    Cancelled,
    /// An error occurred.
    Error(String),
    /// Save operation completed.
    SaveComplete(Result<std::path::PathBuf, String>),
    /// Load operation completed with new world state.
    LoadComplete(Result<WorldUpdate, String>),
}

/// World state snapshot for UI rendering.
/// Contains the mutable fields needed by the UI.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct WorldUpdate {
    /// Player hit points.
    pub player_hp: HitPoints,
    /// Current combat state if any.
    pub combat: Option<CombatState>,
    /// Current game mode.
    pub mode: GameMode,
    /// Current game time.
    pub game_time: GameTime,
    /// Player name.
    pub player_name: String,
    /// Player class.
    pub player_class: Option<String>,
    /// Player level.
    pub player_level: u8,
    /// Player AC.
    pub player_ac: u8,
    /// Player initiative modifier.
    pub player_initiative: i8,
    /// Player speed.
    pub player_speed: u32,
    /// Current location name.
    pub current_location: String,
    /// Campaign name.
    pub campaign_name: String,
    /// Active conditions affecting the player.
    pub conditions: Vec<Condition>,
    /// Death save progress (when at 0 HP).
    pub death_saves: DeathSaves,
    /// Player's gold.
    pub gold: f32,
    /// Equipped weapon name (if any).
    pub equipped_weapon: Option<String>,
    /// Equipped armor name (if any).
    pub equipped_armor: Option<String>,
    /// Inventory items.
    pub inventory_items: Vec<Item>,
    /// Ability scores.
    pub ability_scores: AbilityScores,
    /// Skill proficiencies (skill -> proficiency level string).
    pub skill_proficiencies: HashMap<Skill, String>,
    /// Proficiency bonus.
    pub proficiency_bonus: i8,
    /// Active and completed quests.
    pub quests: Vec<Quest>,
}

impl WorldUpdate {
    /// Create a WorldUpdate snapshot from a GameSession.
    pub fn from_session(session: &GameSession) -> Self {
        let world = session.world();
        let character = &world.player_character;
        Self {
            player_hp: character.hit_points.clone(),
            combat: world.combat.clone(),
            mode: world.mode,
            game_time: world.game_time.clone(),
            player_name: character.name.clone(),
            player_class: character.classes.first().map(|c| c.class.name().to_string()),
            player_level: character.level,
            player_ac: character.current_ac(),
            player_initiative: character.initiative_modifier(),
            player_speed: character.speed.walk,
            current_location: world.current_location.name.clone(),
            campaign_name: world.campaign_name.clone(),
            conditions: character.conditions.iter().map(|c| c.condition).collect(),
            death_saves: character.death_saves.clone(),
            gold: character.inventory.gold,
            equipped_weapon: character
                .equipment
                .main_hand
                .as_ref()
                .map(|w| w.base.name.clone()),
            equipped_armor: character
                .equipment
                .armor
                .as_ref()
                .map(|a| a.base.name.clone()),
            inventory_items: character.inventory.items.clone(),
            ability_scores: character.ability_scores.clone(),
            skill_proficiencies: character
                .skill_proficiencies
                .iter()
                .map(|(skill, level)| (*skill, format!("{level:?}")))
                .collect(),
            proficiency_bonus: character.proficiency_bonus(),
            quests: world.quests.clone(),
        }
    }
}

/// Channel buffer sizes.
const REQUEST_BUFFER: usize = 8;
const RESPONSE_BUFFER: usize = 64; // Larger for streaming chunks

/// Spawn the AI worker and return channel endpoints.
///
/// The worker runs in a background task and owns the GameSession.
/// Communication happens through the returned channels.
pub fn spawn_worker(
    session: GameSession,
) -> (
    mpsc::Sender<WorkerRequest>,
    mpsc::Receiver<WorkerResponse>,
    WorldUpdate,
) {
    let (request_tx, request_rx) = mpsc::channel(REQUEST_BUFFER);
    let (response_tx, response_rx) = mpsc::channel(RESPONSE_BUFFER);

    // Get initial world state before spawning
    let initial_world = WorldUpdate::from_session(&session);

    // Spawn the worker task
    tokio::spawn(worker_loop(session, request_rx, response_tx));

    (request_tx, response_rx, initial_world)
}

/// The main worker loop that processes requests.
async fn worker_loop(
    mut session: GameSession,
    mut request_rx: mpsc::Receiver<WorkerRequest>,
    response_tx: mpsc::Sender<WorkerResponse>,
) {
    loop {
        match request_rx.recv().await {
            Some(WorkerRequest::PlayerAction(input)) => {
                process_player_action(&mut session, &input, &response_tx).await;
            }
            Some(WorkerRequest::Cancel) => {
                // Currently processing is not interruptible at the API level,
                // but we acknowledge the cancel request
                let _ = response_tx.send(WorkerResponse::Cancelled).await;
            }
            Some(WorkerRequest::Save(path)) => {
                let result = session.save(&path).await;
                let response = match result {
                    Ok(()) => WorkerResponse::SaveComplete(Ok(path)),
                    Err(e) => WorkerResponse::SaveComplete(Err(e.to_string())),
                };
                let _ = response_tx.send(response).await;
            }
            Some(WorkerRequest::Load(path)) => {
                match GameSession::load(&path).await {
                    Ok(new_session) => {
                        session = new_session;
                        let world_update = WorldUpdate::from_session(&session);
                        let _ = response_tx
                            .send(WorkerResponse::LoadComplete(Ok(world_update)))
                            .await;
                    }
                    Err(e) => {
                        let _ = response_tx
                            .send(WorkerResponse::LoadComplete(Err(e.to_string())))
                            .await;
                    }
                }
            }
            Some(WorkerRequest::Shutdown) | None => {
                // Graceful shutdown
                break;
            }
        }
    }
}

/// Process a player action and send responses with streaming.
async fn process_player_action(
    session: &mut GameSession,
    input: &str,
    response_tx: &mpsc::Sender<WorkerResponse>,
) {
    let input = input.trim();
    if input.is_empty() {
        return;
    }

    // Clone the sender for use in the streaming callback
    let stream_tx = response_tx.clone();

    // Process the player action with streaming
    let result = session
        .player_action_streaming(input, |text| {
            // Send each text chunk as it arrives
            // Use try_send since we're in a sync callback context
            let _ = stream_tx.try_send(WorkerResponse::StreamChunk(text.to_string()));
        })
        .await;

    match result {
        Ok(response) => {
            // Send individual effects for immediate UI updates
            for effect in &response.effects {
                let _ = response_tx
                    .send(WorkerResponse::Effect(effect.clone()))
                    .await;
            }

            // Build world update
            let world_update = WorldUpdate::from_session(session);

            // Send complete response
            let _ = response_tx
                .send(WorkerResponse::Complete {
                    narrative: response.narrative,
                    effects: response.effects,
                    world_update,
                    in_combat: response.in_combat,
                    is_player_turn: response.is_player_turn,
                })
                .await;
        }
        Err(e) => {
            let _ = response_tx
                .send(WorkerResponse::Error(e.to_string()))
                .await;
        }
    }
}
