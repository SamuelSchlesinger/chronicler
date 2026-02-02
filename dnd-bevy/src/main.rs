//! D&D Bevy GUI - A visual interface for D&D with AI Dungeon Master.
//!
//! This application provides a polished, cross-platform GUI built with Bevy
//! and egui. It features:
//! - Text-based narrative gameplay
//! - Dice rolling animations
//! - Floating damage numbers
//! - Combat effects
//! - Character creation wizard

mod animations;
mod character_creation;
mod effects;
mod runtime;
mod sound;
mod state;
mod ui;
mod window;

use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use serde::Deserialize;

/// Application configuration loaded from config.toml
#[derive(Deserialize)]
struct ConfigFile {
    paths: PathsConfig,
}

#[derive(Deserialize, Clone)]
struct PathsConfig {
    asset_path: String,
    saves_path: String,
}

/// Runtime configuration resource available throughout the app
#[derive(Resource, Clone)]
pub struct AppConfig {
    pub saves_path: String,
}

impl AppConfig {
    pub fn characters_path(&self) -> String {
        format!("{}/characters", self.saves_path)
    }
}

use crate::character_creation::{CharacterCreation, ReadyToStart};
use crate::state::{
    AppState, CharacterSaveList, GamePhase, GameSaveList, OnboardingState, PendingSession,
};
use dnd_core::{GameSession, SessionConfig};

fn main() {
    // Load .env file if present
    dotenvy::dotenv().ok();

    // Load configuration from config.toml
    let config: ConfigFile = std::fs::read_to_string("config.toml")
        .map_err(|e| format!("Failed to read config.toml: {e}"))
        .and_then(|s| toml::from_str(&s).map_err(|e| format!("Failed to parse config.toml: {e}")))
        .expect("config.toml must exist and be valid. Run from workspace root.");

    let asset_path = config.paths.asset_path;
    let saves_path = config.paths.saves_path.clone();

    // Create saves directories if they don't exist
    std::fs::create_dir_all(&saves_path).ok();
    std::fs::create_dir_all(format!("{}/characters", &saves_path)).ok();

    let app_config = AppConfig {
        saves_path: saves_path.clone(),
    };

    // Load settings from disk
    let window_settings = window::load_settings(&saves_path);
    let sound_settings = sound::load_settings(&saves_path);
    let onboarding_state = OnboardingState::load(&saves_path);

    // Always use windowed mode (fullscreen disabled due to macOS issues)
    let initial_window_mode = bevy::window::WindowMode::Windowed;

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "D&D: AI Dungeon Master".into(),
                        resolution: (window_settings.width, window_settings.height).into(),
                        resizable: true,
                        mode: initial_window_mode,
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    file_path: asset_path,
                    ..default()
                }),
        )
        .add_plugins(EguiPlugin)
        .add_plugins(sound::SoundPlugin)
        .add_plugins(window::WindowSettingsPlugin)
        .insert_resource(app_config)
        .insert_resource(window_settings)
        .insert_resource(sound_settings)
        .insert_resource(onboarding_state)
        // App state
        .init_state::<GamePhase>()
        .init_resource::<AppState>()
        .init_resource::<CharacterSaveList>()
        .init_resource::<GameSaveList>()
        // Startup systems
        .add_systems(Startup, setup)
        // State transition systems
        .add_systems(
            OnEnter(GamePhase::CharacterCreation),
            setup_character_creation,
        )
        .add_systems(
            OnExit(GamePhase::CharacterCreation),
            cleanup_character_creation,
        )
        // Update systems - UI
        .add_systems(Update, (ui::main_ui_system, ui::handle_keyboard_input))
        // Update systems - animations
        .add_systems(
            Update,
            (
                animations::animate_screen_shake,
                animations::cleanup_finished_animations,
            ),
        )
        // Update systems - AI worker and session management
        .add_systems(
            Update,
            (
                state::handle_worker_responses,
                state::process_pending_sounds,
                state::check_pending_session,
                state::check_pending_character_list,
                state::check_pending_game_list,
                state::check_pending_game_load,
                state::clear_old_status,
                handle_ready_to_start,
            ),
        )
        .run();
}

/// Initial setup system.
fn setup(mut commands: Commands) {
    // Spawn 2D camera for animations
    commands.spawn(Camera2d);
}

/// Setup character creation when entering that state.
fn setup_character_creation(mut commands: Commands) {
    commands.insert_resource(CharacterCreation::new());
}

/// Cleanup character creation when exiting that state.
fn cleanup_character_creation(mut commands: Commands) {
    commands.remove_resource::<CharacterCreation>();
}

/// Handle ReadyToStart - spawn async session creation.
fn handle_ready_to_start(
    mut commands: Commands,
    ready: Option<Res<ReadyToStart>>,
    mut app_state: ResMut<AppState>,
) {
    let Some(ready) = ready else { return };

    // Create a channel to receive the session
    let (tx, rx) = std::sync::mpsc::channel();

    let character = ready.character.clone();
    let campaign_name = ready.campaign_name.clone();

    // Spawn async session creation
    std::thread::spawn(move || {
        let result = crate::runtime::RUNTIME.block_on(async {
            let config = SessionConfig::new(&campaign_name).with_character_name(&character.name);

            GameSession::new_with_character(config, character)
                .await
                .map_err(|e| e.to_string())
        });
        let _ = tx.send(result);
    });

    // Store the pending session receiver
    commands.insert_resource(PendingSession {
        receiver: std::sync::Mutex::new(rx),
    });

    // Remove ReadyToStart
    commands.remove_resource::<ReadyToStart>();

    // Show loading status
    app_state.set_status_persistent("Creating adventure...");
}
