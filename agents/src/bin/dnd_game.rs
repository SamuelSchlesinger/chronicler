//! D&D Dungeon Master Game - Entry Point
//!
//! A single-player D&D 5e experience with an AI Dungeon Master.

use std::io::{self, stdout};
use std::sync::Arc;
use std::time::Duration;

use crossterm::{
    event::{self},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use tokio::sync::mpsc;

use agentic::llm::anthropic::AnthropicProvider;

use agents::dnd::app::{AiRequest, AiResponse, AppState};
use agents::dnd::events::{handle_event, EventResult};
use agents::dnd::ui::render::render;

#[tokio::main]
async fn main() -> io::Result<()> {
    // Load environment variables from .env file
    if dotenvy::from_path("../.env").is_err() {
        let _ = dotenvy::dotenv();
    }

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create application state
    let mut app = AppState::new();

    // Try to initialize AI
    let ai_initialized = if let Ok(api_key) = std::env::var("ANTHROPIC_API_KEY") {
        let llm = Arc::new(AnthropicProvider::new(api_key));
        app.init_ai(llm.clone());

        // Set up communication channels
        let (request_tx, mut request_rx) = mpsc::channel::<AiRequest>(8);
        let (response_tx, response_rx) = mpsc::channel::<AiResponse>(32);

        app.setup_ai_channels(request_tx, response_rx);

        // Get reference to DM agent
        let dm_agent = app.dm_agent.clone().unwrap();

        // Spawn the AI processing task
        tokio::spawn(async move {
            while let Some(request) = request_rx.recv().await {
                let tx = response_tx.clone();
                let agent = dm_agent.clone();

                // Process the AI request
                match agent
                    .process_action(
                        &request.player_input,
                        &request.game_snapshot,
                        &request.conversation_history,
                    )
                    .await
                {
                    Ok(dm_response) => {
                        // Send tool results first
                        for tool_result in &dm_response.tool_results {
                            let _ = tx
                                .send(AiResponse::ToolResult {
                                    tool_name: tool_result.tool_name.clone(),
                                    output: tool_result.output.clone(),
                                })
                                .await;
                        }

                        // Send narrative response
                        if !dm_response.narrative.is_empty() {
                            let _ = tx.send(AiResponse::Narrative(dm_response.narrative)).await;
                        }

                        // Signal completion
                        let _ = tx.send(AiResponse::Complete).await;
                    }
                    Err(e) => {
                        let _ = tx.send(AiResponse::Error(e.to_string())).await;
                    }
                }
            }
        });

        true
    } else {
        false
    };

    if !ai_initialized {
        app.add_narrative(
            "[No ANTHROPIC_API_KEY found - running in demo mode]".to_string(),
            agents::dnd::game::state::NarrativeType::System,
        );
    }

    // Main loop
    let result = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(e) = result {
        eprintln!("Error: {e}");
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut AppState) -> io::Result<()> {
    let tick_rate = Duration::from_millis(100);

    loop {
        // Poll for AI responses
        app.poll_ai_responses();

        // Draw
        terminal.draw(|frame| render(frame, app))?;

        // Handle events with timeout for animations
        if event::poll(tick_rate)? {
            let event = event::read()?;

            match handle_event(app, event) {
                EventResult::Quit => break,
                EventResult::Continue | EventResult::NeedsRedraw => {}
            }
        }

        // Tick for animations
        app.tick();

        // Check for quit flag
        if app.should_quit {
            break;
        }
    }

    Ok(())
}
