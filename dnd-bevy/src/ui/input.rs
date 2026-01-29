//! Input panel for player commands.

use bevy_egui::egui;
use dnd_core::world::NarrativeType;

use crate::state::AppState;

/// Render the input panel at the bottom of the screen.
pub fn render_input_panel(ctx: &egui::Context, app_state: &mut AppState) {
    egui::TopBottomPanel::bottom("input_panel")
        .min_height(60.0)
        .show(ctx, |ui| {
            ui.add_space(4.0);

            ui.horizontal(|ui| {
                // Input label
                ui.label(">");

                // Check for key presses before creating the text edit
                let enter_pressed = ctx.input(|i| i.key_pressed(egui::Key::Enter));
                let up_pressed = ctx.input(|i| i.key_pressed(egui::Key::ArrowUp));
                let down_pressed = ctx.input(|i| i.key_pressed(egui::Key::ArrowDown));

                // Text input
                let response = ui.add_sized(
                    [ui.available_width() - 80.0, 30.0],
                    egui::TextEdit::singleline(&mut app_state.input_text)
                        .hint_text("What do you do? (Up/Down for history)")
                        .interactive(!app_state.is_processing),
                );

                // Handle history navigation when input has focus
                if response.has_focus() || response.lost_focus() {
                    if up_pressed {
                        app_state.history_up();
                    } else if down_pressed {
                        app_state.history_down();
                    }
                }

                // Submit on Enter when text field has focus (or just lost it due to Enter)
                let should_submit = enter_pressed
                    && (response.has_focus() || response.lost_focus())
                    && !app_state.input_text.trim().is_empty()
                    && !app_state.is_processing;

                if should_submit {
                    let action = std::mem::take(&mut app_state.input_text);
                    // Add to history
                    app_state.add_to_history(action.clone());
                    // Add player action to narrative
                    app_state.add_narrative(
                        action.clone(),
                        NarrativeType::PlayerAction,
                        0.0,
                    );
                    app_state.send_action(action);
                }

                // Auto-focus the input field (but not right after submitting)
                if !app_state.is_processing && !should_submit {
                    response.request_focus();
                }

                // Send button
                let send_enabled = !app_state.is_processing && !app_state.input_text.trim().is_empty();
                if ui
                    .add_enabled(send_enabled, egui::Button::new("Send"))
                    .clicked()
                {
                    let action = std::mem::take(&mut app_state.input_text);
                    app_state.add_to_history(action.clone());
                    app_state.add_narrative(action.clone(), NarrativeType::PlayerAction, 0.0);
                    app_state.send_action(action);
                }
            });

            // Quick action buttons (disabled while processing)
            ui.horizontal(|ui| {
                ui.add_space(16.0);
                ui.add_enabled_ui(!app_state.is_processing, |ui| {
                    // Combat actions (shown during combat)
                    if app_state.in_combat && app_state.is_player_turn {
                        if ui.small_button("Attack").clicked() {
                            app_state.input_text = "I attack ".to_string();
                        }
                        if ui.small_button("Dodge").clicked() {
                            let action = "I take the Dodge action".to_string();
                            app_state.add_narrative(action.clone(), NarrativeType::PlayerAction, 0.0);
                            app_state.send_action(action);
                        }
                        if ui.small_button("Disengage").clicked() {
                            let action = "I take the Disengage action".to_string();
                            app_state.add_narrative(action.clone(), NarrativeType::PlayerAction, 0.0);
                            app_state.send_action(action);
                        }
                        ui.separator();
                    }

                    // General actions
                    if ui.small_button("Look Around").clicked() {
                        let action = "I look around".to_string();
                        app_state.add_narrative(action.clone(), NarrativeType::PlayerAction, 0.0);
                        app_state.send_action(action);
                    }
                    if ui.small_button("Check Inventory").clicked() {
                        let action = "I check my inventory".to_string();
                        app_state.add_narrative(action.clone(), NarrativeType::PlayerAction, 0.0);
                        app_state.send_action(action);
                    }
                    if ui.small_button("Rest").clicked() {
                        app_state.input_text = "I want to take a ".to_string();
                    }
                });
            });
        });
}
