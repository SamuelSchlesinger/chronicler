//! Character sheet overlay.

use bevy_egui::egui;
use dnd_core::world::Ability;

use crate::state::AppState;

/// Render the character sheet overlay.
pub fn render_character_sheet(ctx: &egui::Context, app_state: &mut AppState) {
    // Use responsive sizing based on available screen
    let screen = ctx.screen_rect();
    let width = (screen.width() * 0.85).clamp(320.0, 500.0);
    let height = (screen.height() * 0.8).clamp(350.0, 550.0);

    egui::Window::new("Character Sheet")
        .collapsible(false)
        .resizable(true)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .default_size([width, height])
        .max_size([600.0, 700.0])
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
                    ui.label(
                        egui::RichText::new(format!("{}", app_state.world.player_ac)).strong(),
                    );
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

            // Spellcasting section (if spellcaster)
            let has_spells = !app_state.world.cantrips.is_empty()
                || !app_state.world.known_spells.is_empty()
                || app_state.world.spell_slots.iter().any(|(_, t)| *t > 0);

            if has_spells {
                ui.heading("Spellcasting");

                // Spellcasting stats
                ui.horizontal(|ui| {
                    if let Some(ref ability) = app_state.world.spellcasting_ability {
                        ui.label(format!("Ability: {}", ability));
                    }
                    if let Some(dc) = app_state.world.spell_save_dc {
                        ui.separator();
                        ui.label(format!("Save DC: {}", dc));
                    }
                    if let Some(atk) = app_state.world.spell_attack_bonus {
                        ui.separator();
                        ui.label(format!("Attack: +{}", atk));
                    }
                });

                // Spell slots
                let has_slots = app_state.world.spell_slots.iter().any(|(_, t)| *t > 0);
                if has_slots {
                    ui.add_space(4.0);
                    ui.label(egui::RichText::new("Spell Slots").strong());
                    ui.horizontal_wrapped(|ui| {
                        for (i, (available, total)) in
                            app_state.world.spell_slots.iter().enumerate()
                        {
                            if *total > 0 {
                                let level = i + 1;
                                let color = if *available > 0 {
                                    egui::Color32::from_rgb(100, 180, 255)
                                } else {
                                    egui::Color32::DARK_GRAY
                                };
                                ui.label(
                                    egui::RichText::new(format!(
                                        "Lv{}: {}/{}",
                                        level, available, total
                                    ))
                                    .color(color),
                                );
                                ui.add_space(8.0);
                            }
                        }
                    });
                }

                // Cantrips - clickable to view details
                if !app_state.world.cantrips.is_empty() {
                    ui.add_space(4.0);
                    let cantrips = app_state.world.cantrips.clone();
                    ui.collapsing(
                        egui::RichText::new(format!(
                            "Cantrips ({}) - click for details",
                            cantrips.len()
                        ))
                        .strong(),
                        |ui| {
                            for cantrip in &cantrips {
                                if ui.small_button(format!("- {}", cantrip)).clicked() {
                                    app_state.viewing_spell = Some(cantrip.clone());
                                }
                            }
                        },
                    );
                }

                // Known/Prepared Spells - clickable to view details
                if !app_state.world.known_spells.is_empty() {
                    ui.add_space(4.0);
                    let spells = app_state.world.known_spells.clone();
                    ui.collapsing(
                        egui::RichText::new(format!(
                            "Spells ({}) - click for details",
                            spells.len()
                        ))
                        .strong(),
                        |ui| {
                            egui::ScrollArea::vertical()
                                .max_height(150.0)
                                .show(ui, |ui| {
                                    for spell in &spells {
                                        if ui.small_button(format!("- {}", spell)).clicked() {
                                            app_state.viewing_spell = Some(spell.clone());
                                        }
                                    }
                                });
                        },
                    );
                }

                ui.separator();
            }

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
