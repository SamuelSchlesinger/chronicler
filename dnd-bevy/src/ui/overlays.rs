//! Overlay windows for inventory, character sheet, etc.

use bevy_egui::egui;
use dnd_core::world::{Ability, QuestStatus};

use crate::state::AppState;

/// Render the inventory overlay.
pub fn render_inventory(ctx: &egui::Context, app_state: &AppState) {
    egui::Window::new("Inventory")
        .collapsible(false)
        .resizable(true)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .default_size([400.0, 500.0])
        .show(ctx, |ui| {
            // Gold
            ui.horizontal(|ui| {
                ui.label("Gold:");
                ui.label(
                    egui::RichText::new(format!("{:.0} gp", app_state.world.gold))
                        .color(egui::Color32::from_rgb(218, 165, 32))
                        .strong(),
                );
            });

            ui.separator();

            // Equipped items
            ui.heading("Equipped");
            ui.indent("equipped", |ui| {
                if let Some(ref weapon) = app_state.world.equipped_weapon {
                    ui.label(format!("Main Hand: {weapon}"));
                } else {
                    ui.label("Main Hand: (empty)");
                }
                if let Some(ref armor) = app_state.world.equipped_armor {
                    ui.label(format!("Armor: {armor}"));
                } else {
                    ui.label("Armor: (none)");
                }
            });

            ui.separator();

            // Inventory items
            ui.heading("Items");

            if app_state.world.inventory_items.is_empty() {
                ui.label(egui::RichText::new("Your pack is empty.").italics());
            } else {
                egui::ScrollArea::vertical()
                    .max_height(300.0)
                    .show(ui, |ui| {
                        for item in &app_state.world.inventory_items {
                            ui.horizontal(|ui| {
                                let name = if item.quantity > 1 {
                                    format!("{} x{}", item.name, item.quantity)
                                } else {
                                    item.name.clone()
                                };

                                let color = if item.magical {
                                    egui::Color32::from_rgb(138, 43, 226) // Purple for magical
                                } else {
                                    egui::Color32::WHITE
                                };

                                ui.label(egui::RichText::new(name).color(color));

                                if item.weight > 0.0 {
                                    ui.label(
                                        egui::RichText::new(format!("({:.1} lb)", item.weight))
                                            .color(egui::Color32::GRAY)
                                            .small(),
                                    );
                                }
                            });

                            if let Some(ref desc) = item.description {
                                ui.indent("item_desc", |ui| {
                                    ui.label(
                                        egui::RichText::new(desc)
                                            .color(egui::Color32::GRAY)
                                            .small()
                                            .italics(),
                                    );
                                });
                            }
                        }
                    });
            }

            ui.separator();
            ui.label(
                egui::RichText::new("Press I or Escape to close")
                    .small()
                    .color(egui::Color32::GRAY),
            );
        });
}

/// Render the character sheet overlay.
pub fn render_character_sheet(ctx: &egui::Context, app_state: &AppState) {
    egui::Window::new("Character Sheet")
        .collapsible(false)
        .resizable(true)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .default_size([500.0, 600.0])
        .show(ctx, |ui| {
            // Header
            ui.horizontal(|ui| {
                ui.heading(
                    egui::RichText::new(&app_state.world.player_name)
                        .color(egui::Color32::from_rgb(218, 165, 32)),
                );
                if let Some(ref class) = app_state.world.player_class {
                    ui.label(format!("Level {} {}", app_state.world.player_level, class));
                }
            });

            ui.separator();

            // Two-column layout
            ui.columns(2, |columns| {
                // Left column: Ability scores
                columns[0].heading("Ability Scores");
                columns[0].separator();

                let abilities = [
                    (Ability::Strength, "STR"),
                    (Ability::Dexterity, "DEX"),
                    (Ability::Constitution, "CON"),
                    (Ability::Intelligence, "INT"),
                    (Ability::Wisdom, "WIS"),
                    (Ability::Charisma, "CHA"),
                ];

                for (ability, abbr) in abilities {
                    let score = app_state.world.ability_scores.get(ability);
                    let modifier = app_state.world.ability_scores.modifier(ability);
                    let mod_str = if modifier >= 0 {
                        format!("+{modifier}")
                    } else {
                        format!("{modifier}")
                    };

                    columns[0].horizontal(|ui| {
                        ui.label(egui::RichText::new(abbr).strong());
                        ui.label(format!("{score:2}"));
                        ui.label(
                            egui::RichText::new(format!("({mod_str})"))
                                .color(egui::Color32::from_rgb(100, 180, 255)),
                        );
                    });
                }

                columns[0].add_space(10.0);
                columns[0].separator();
                columns[0].heading("Combat Stats");
                columns[0].separator();

                columns[0].horizontal(|ui| {
                    ui.label("Armor Class:");
                    ui.label(egui::RichText::new(format!("{}", app_state.world.player_ac)).strong());
                });

                columns[0].horizontal(|ui| {
                    ui.label("Initiative:");
                    let init = app_state.world.player_initiative;
                    ui.label(
                        egui::RichText::new(if init >= 0 {
                            format!("+{init}")
                        } else {
                            format!("{init}")
                        })
                        .strong(),
                    );
                });

                columns[0].horizontal(|ui| {
                    ui.label("Speed:");
                    ui.label(format!("{} ft", app_state.world.player_speed));
                });

                columns[0].horizontal(|ui| {
                    ui.label("Proficiency Bonus:");
                    ui.label(
                        egui::RichText::new(format!("+{}", app_state.world.proficiency_bonus))
                            .strong(),
                    );
                });

                // Right column: Skills
                columns[1].heading("Skills");
                columns[1].separator();

                let mut skills: Vec<_> = app_state.world.skill_proficiencies.iter().collect();
                skills.sort_by_key(|(skill, _)| skill.name());

                for (skill, proficiency) in skills {
                    columns[1].horizontal(|ui| {
                        let is_proficient = proficiency != "NotProficient";
                        let marker = if is_proficient { "[*]" } else { "[ ]" };
                        let color = if is_proficient {
                            egui::Color32::GREEN
                        } else {
                            egui::Color32::DARK_GRAY
                        };

                        ui.label(egui::RichText::new(marker).color(color));
                        ui.label(skill.name());
                    });
                }
            });

            ui.separator();

            // Conditions
            if !app_state.world.conditions.is_empty() {
                ui.heading("Active Conditions");
                ui.horizontal_wrapped(|ui| {
                    for condition in &app_state.world.conditions {
                        ui.label(
                            egui::RichText::new(format!("{condition}"))
                                .color(egui::Color32::YELLOW)
                                .background_color(egui::Color32::from_rgb(60, 50, 40)),
                        );
                    }
                });
                ui.separator();
            }

            ui.label(
                egui::RichText::new("Press C or Escape to close")
                    .small()
                    .color(egui::Color32::GRAY),
            );
        });
}

/// Render the quest log overlay.
pub fn render_quest_log(ctx: &egui::Context, app_state: &AppState) {
    egui::Window::new("Quest Log")
        .collapsible(false)
        .resizable(true)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .default_size([450.0, 500.0])
        .show(ctx, |ui| {
            if app_state.world.quests.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(50.0);
                    ui.label(
                        egui::RichText::new("No quests yet.")
                            .italics()
                            .color(egui::Color32::GRAY),
                    );
                    ui.add_space(10.0);
                    ui.label("Your adventure awaits...");
                });
            } else {
                // Active quests
                let active_quests: Vec<_> = app_state
                    .world
                    .quests
                    .iter()
                    .filter(|q| q.status == QuestStatus::Active)
                    .collect();

                if !active_quests.is_empty() {
                    ui.heading(
                        egui::RichText::new("Active Quests")
                            .color(egui::Color32::YELLOW),
                    );
                    ui.separator();

                    for quest in active_quests {
                        ui.group(|ui| {
                            ui.label(egui::RichText::new(&quest.name).strong());
                            ui.label(&quest.description);

                            // Objectives
                            if !quest.objectives.is_empty() {
                                ui.add_space(4.0);
                                for obj in &quest.objectives {
                                    let marker = if obj.completed { "[X]" } else { "[ ]" };
                                    let color = if obj.completed {
                                        egui::Color32::GREEN
                                    } else {
                                        egui::Color32::WHITE
                                    };
                                    ui.label(
                                        egui::RichText::new(format!("{} {}", marker, obj.description))
                                            .color(color),
                                    );
                                }
                            }
                        });
                        ui.add_space(4.0);
                    }
                }

                // Completed quests
                let completed_quests: Vec<_> = app_state
                    .world
                    .quests
                    .iter()
                    .filter(|q| q.status == QuestStatus::Completed)
                    .collect();

                if !completed_quests.is_empty() {
                    ui.add_space(10.0);
                    ui.heading(
                        egui::RichText::new("Completed Quests")
                            .color(egui::Color32::GREEN),
                    );
                    ui.separator();

                    for quest in completed_quests {
                        ui.label(
                            egui::RichText::new(format!("[Done] {}", quest.name))
                                .color(egui::Color32::from_rgb(100, 180, 100)),
                        );
                    }
                }

                // Failed quests
                let failed_quests: Vec<_> = app_state
                    .world
                    .quests
                    .iter()
                    .filter(|q| q.status == QuestStatus::Failed || q.status == QuestStatus::Abandoned)
                    .collect();

                if !failed_quests.is_empty() {
                    ui.add_space(10.0);
                    ui.heading(
                        egui::RichText::new("Failed Quests")
                            .color(egui::Color32::RED),
                    );
                    ui.separator();

                    for quest in failed_quests {
                        ui.label(
                            egui::RichText::new(format!("[Failed] {}", quest.name))
                                .color(egui::Color32::from_rgb(180, 100, 100)),
                        );
                    }
                }
            }

            ui.separator();
            ui.label(
                egui::RichText::new("Press Shift+Q or Escape to close")
                    .small()
                    .color(egui::Color32::GRAY),
            );
        });
}

/// Render the help overlay.
pub fn render_help(ctx: &egui::Context) {
    egui::Window::new("Help")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .default_size([400.0, 400.0])
        .show(ctx, |ui| {
            ui.heading("D&D: AI Dungeon Master");
            ui.separator();

            ui.heading("How to Play");
            ui.label("Type natural language commands to interact with the world.");
            ui.label("The AI Dungeon Master will respond to your actions.");
            ui.add_space(10.0);

            ui.heading("Example Commands");
            ui.label("• \"I look around the room\"");
            ui.label("• \"I attack the goblin with my sword\"");
            ui.label("• \"I try to pick the lock\"");
            ui.label("• \"I cast fireball at the enemies\"");
            ui.label("• \"I search the chest\"");
            ui.add_space(10.0);

            ui.heading("Keyboard Shortcuts");
            ui.add_space(4.0);

            ui.label(egui::RichText::new("Always Available:").strong().color(egui::Color32::from_rgb(218, 165, 32)));
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Ctrl+S / Cmd+S").strong());
                ui.label("- Quick Save");
            });
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Up / Down").strong());
                ui.label("- Browse command history");
            });
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Enter").strong());
                ui.label("- Send command");
            });

            ui.add_space(8.0);
            ui.label(
                egui::RichText::new("When not typing:")
                    .strong()
                    .color(egui::Color32::from_rgb(218, 165, 32)),
            );
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("I").strong());
                ui.label("- Inventory");
            });
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("C").strong());
                ui.label("- Character Sheet");
            });
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Shift+Q").strong());
                ui.label("- Quest Log");
            });
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("F1 / ?").strong());
                ui.label("- Help (this screen)");
            });
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Escape").strong());
                ui.label("- Close overlay");
            });

            ui.separator();
            ui.label(
                egui::RichText::new("Press F1, ?, or Escape to close")
                    .small()
                    .color(egui::Color32::GRAY),
            );
        });
}

/// Render the settings overlay.
pub fn render_settings(ctx: &egui::Context, app_state: &mut AppState) {
    egui::Window::new("Settings")
        .collapsible(false)
        .resizable(true)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .default_size([400.0, 450.0])
        .show(ctx, |ui| {
            ui.heading("Game Settings");
            ui.separator();

            // Display section
            ui.collapsing(
                egui::RichText::new("Display").strong(),
                |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Character panel:");
                        if ui.selectable_label(app_state.character_panel_expanded, "Expanded").clicked() {
                            app_state.character_panel_expanded = true;
                        }
                        if ui.selectable_label(!app_state.character_panel_expanded, "Collapsed").clicked() {
                            app_state.character_panel_expanded = false;
                        }
                    });

                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new("Font scaling and other display options coming soon")
                            .small()
                            .italics()
                            .color(egui::Color32::GRAY),
                    );
                },
            );

            ui.add_space(8.0);

            // Audio section (placeholder)
            ui.collapsing(
                egui::RichText::new("Audio").strong(),
                |ui| {
                    ui.label(
                        egui::RichText::new("Audio settings coming soon")
                            .small()
                            .italics()
                            .color(egui::Color32::GRAY),
                    );
                },
            );

            ui.add_space(8.0);

            // Gameplay section
            ui.collapsing(
                egui::RichText::new("Gameplay").strong(),
                |ui| {
                    ui.label(
                        egui::RichText::new("Gameplay options coming soon")
                            .small()
                            .italics()
                            .color(egui::Color32::GRAY),
                    );
                },
            );

            ui.add_space(8.0);

            // Save files section
            ui.collapsing(
                egui::RichText::new("Save Files").strong(),
                |ui| {
                    ui.label("Save directory: saves/");

                    ui.add_space(4.0);

                    if ui.button("Open saves folder").clicked() {
                        #[cfg(target_os = "macos")]
                        {
                            let _ = std::process::Command::new("open").arg("saves").spawn();
                        }
                        #[cfg(target_os = "windows")]
                        {
                            let _ = std::process::Command::new("explorer").arg("saves").spawn();
                        }
                        #[cfg(target_os = "linux")]
                        {
                            let _ = std::process::Command::new("xdg-open").arg("saves").spawn();
                        }
                    }
                },
            );

            ui.add_space(8.0);

            // About section
            ui.collapsing(
                egui::RichText::new("About").strong(),
                |ui| {
                    ui.label(
                        egui::RichText::new("D&D: AI Dungeon Master")
                            .color(egui::Color32::from_rgb(218, 165, 32)),
                    );
                    ui.label("A text-based adventure powered by AI");
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new("Built with Bevy + egui")
                            .small()
                            .color(egui::Color32::GRAY),
                    );
                },
            );

            ui.separator();
            ui.label(
                egui::RichText::new("Press Escape to close")
                    .small()
                    .color(egui::Color32::GRAY),
            );
        });
}
