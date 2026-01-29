//! Render orchestration for the D&D TUI

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use dnd_core::dice::RollResult;
use dnd_core::world::GameMode;

use crate::ai_worker::WorldUpdate;
use crate::app::{App, InputMode};
use crate::ui::layout::{centered_rect_fixed, AppLayout, CombatLayout};
use crate::ui::widgets::{
    CombatTrackerWidget, DiceAnimationState, DiceRollWidget, EnemyHpWidget, EnemyStatus,
    HotkeyBarWidget, HpEstimate, InputWidget, NarrativeWidget, StatusBarWidget,
};

/// Which panel is focused
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FocusedPanel {
    #[default]
    Narrative,
    Character,
    Combat,
}

/// Overlay types
#[derive(Debug, Clone)]
pub enum Overlay {
    Help,
    DiceRoll {
        result: Option<RollResult>,
        purpose: String,
        dc: Option<i32>,
    },
    Inventory,
    CharacterSheet,
    QuestLog,
}

/// Main render function
pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // Choose layout based on game mode
    match app.world.mode {
        GameMode::Combat => render_combat_layout(frame, app, area),
        _ => render_exploration_layout(frame, app, area),
    }

    // Render overlay if present
    if let Some(overlay) = app.overlay() {
        render_overlay(frame, app, overlay, area);
    }
}

/// Render exploration layout (70/30 split)
fn render_exploration_layout(frame: &mut Frame, app: &App, area: Rect) {
    let layout = AppLayout::calculate(area);

    // Title bar
    render_title_bar(frame, &app.world, layout.title_area);

    // Narrative panel
    let narrative_widget = NarrativeWidget::new(&app.narrative_history, &app.theme)
        .scroll(app.narrative_scroll)
        .focused(matches!(app.focused_panel, FocusedPanel::Narrative))
        .streaming(app.streaming_text.as_deref());
    frame.render_widget(narrative_widget, layout.narrative_area);

    // Character panel
    let character_widget = SimplifiedCharacterWidget::new(&app.world, &app.theme)
        .focused(matches!(app.focused_panel, FocusedPanel::Character));
    frame.render_widget(character_widget, layout.sidebar_area);

    // Status bar
    render_status_bar(frame, app, layout.status_bar);

    // Hotkey bar
    render_hotkey_bar(frame, app, layout.hotkey_bar);

    // Input area
    render_input(frame, app, layout.input_area);
}

/// Render combat layout (65/35 split)
fn render_combat_layout(frame: &mut Frame, app: &App, area: Rect) {
    let layout = CombatLayout::calculate(area);

    // Title bar (combat mode)
    render_combat_title(frame, &app.world, layout.title_area);

    // Narrative/combat log
    let narrative_widget = NarrativeWidget::new(&app.narrative_history, &app.theme)
        .scroll(app.narrative_scroll)
        .focused(matches!(app.focused_panel, FocusedPanel::Narrative))
        .streaming(app.streaming_text.as_deref());
    frame.render_widget(narrative_widget, layout.narrative_area);

    // Initiative tracker
    if let Some(ref combat) = app.world.combat {
        let combat_widget = CombatTrackerWidget::new(combat, &app.theme)
            .focused(matches!(app.focused_panel, FocusedPanel::Combat));
        frame.render_widget(combat_widget, layout.initiative_area);

        // Build enemy status list from combatants
        let enemies: Vec<EnemyStatus> = combat
            .get_enemies()
            .iter()
            .map(|c| {
                let hp_ratio = if c.max_hp > 0 {
                    c.current_hp as f64 / c.max_hp as f64
                } else {
                    1.0
                };
                let hp_estimate = if c.current_hp <= 0 {
                    HpEstimate::Dead
                } else if hp_ratio <= 0.25 {
                    HpEstimate::Critical
                } else if hp_ratio <= 0.5 {
                    HpEstimate::Bloodied
                } else {
                    HpEstimate::Healthy
                };
                EnemyStatus {
                    name: c.name.clone(),
                    hp_estimate,
                    conditions: vec![],
                }
            })
            .collect();

        let enemy_widget = EnemyHpWidget::new(enemies, &app.theme);
        frame.render_widget(enemy_widget, layout.enemy_hp_area);
    }

    // Status bar (with combat info)
    render_combat_status(frame, app, layout.status_bar);

    // Hotkey bar
    render_hotkey_bar(frame, app, layout.hotkey_bar);

    // Input area
    render_input(frame, app, layout.input_area);
}

/// Render the title bar
fn render_title_bar(frame: &mut Frame, world: &WorldUpdate, area: Rect) {
    let time = &world.game_time;
    let title = format!(
        " {} | {} | {}:{:02} {} ",
        world.current_location,
        time.time_of_day(),
        time.hour,
        time.minute,
        if time.is_daytime() { "☀" } else { "☽" }
    );

    let line = Line::from(Span::styled(
        title,
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    ));
    frame.render_widget(Paragraph::new(line), area);
}

/// Render the combat title bar
fn render_combat_title(frame: &mut Frame, world: &WorldUpdate, area: Rect) {
    let round = world.combat.as_ref().map(|c| c.round).unwrap_or(1);

    let title = format!(" ⚔ COMBAT - Round {round} ⚔ ");

    let line = Line::from(Span::styled(
        title,
        Style::default()
            .fg(Color::LightRed)
            .add_modifier(Modifier::BOLD),
    ));
    frame.render_widget(Paragraph::new(line), area);
}

/// Render the status bar
fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let status_widget = StatusBarWidget::from_world(&app.world, app.input_mode, &app.theme)
        .message(app.status_message());

    frame.render_widget(status_widget, area);
}

/// Render combat-specific status bar
fn render_combat_status(frame: &mut Frame, app: &App, area: Rect) {
    let hp = &app.world.player_hp;
    let hp_color = app.theme.hp_color(hp.ratio());

    // Build combat-specific status
    let is_player_turn = app
        .world
        .combat
        .as_ref()
        .and_then(|c| c.current_combatant())
        .map(|c| c.is_player)
        .unwrap_or(false);

    let turn_indicator = if is_player_turn {
        Span::styled(
            "YOUR TURN",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled("Waiting...", Style::default().fg(Color::DarkGray))
    };

    // Build condition display if any
    let conditions_span = if !app.world.conditions.is_empty() {
        let condition_names: Vec<String> = app
            .world
            .conditions
            .iter()
            .map(|c| c.name().to_string())
            .collect();
        vec![
            Span::raw(" | "),
            Span::styled(
                format!("⚠ {}", condition_names.join(", ")),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        ]
    } else {
        vec![]
    };

    // Build death saves display if dying
    let death_save_span = if hp.current <= 0 {
        let ds = &app.world.death_saves;
        vec![
            Span::raw(" | "),
            Span::styled(
                format!("Deaths: ✓{} ✗{}", ds.successes, ds.failures),
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
        ]
    } else {
        vec![]
    };

    let mut spans = vec![
        Span::styled(
            format!("HP: {}/{}", hp.current, hp.maximum),
            Style::default().fg(hp_color),
        ),
        Span::raw(" | "),
        Span::styled(format!("AC: {}", app.world.player_ac), Style::default()),
        Span::raw(" | "),
        turn_indicator,
    ];
    spans.extend(conditions_span);
    spans.extend(death_save_span);

    let line = Line::from(spans);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(app.theme.border_style(false));

    frame.render_widget(Paragraph::new(line).block(block), area);
}

/// Render the hotkey bar
fn render_hotkey_bar(frame: &mut Frame, app: &App, area: Rect) {
    let hotkey_widget = HotkeyBarWidget::new(app.world.mode, app.input_mode, &app.theme);
    frame.render_widget(hotkey_widget, area);
}

/// Render the input area
fn render_input(frame: &mut Frame, app: &App, area: Rect) {
    let is_active = matches!(app.input_mode, InputMode::Insert | InputMode::Command);
    let is_command = matches!(app.input_mode, InputMode::Command);

    let placeholder = if app.ai_processing {
        "Processing... (Esc to cancel)"
    } else {
        "Enter your action..."
    };

    let input_widget = InputWidget::new(app.input_buffer(), &app.theme)
        .cursor_position(app.cursor_position())
        .active(is_active)
        .command_mode(is_command)
        .placeholder(placeholder);

    frame.render_widget(input_widget, area);
}

/// Render overlay
fn render_overlay(frame: &mut Frame, app: &App, overlay: &Overlay, area: Rect) {
    match overlay {
        Overlay::Help => render_help_overlay(frame, app, area),
        Overlay::DiceRoll {
            result,
            purpose,
            dc,
        } => render_dice_overlay(frame, app, result.as_ref(), purpose, *dc, area),
        Overlay::Inventory => render_inventory_overlay(frame, app, area),
        Overlay::CharacterSheet => render_character_sheet_overlay(frame, app, area),
        Overlay::QuestLog => render_quest_log_overlay(frame, app, area),
    }
}

/// Render help overlay
fn render_help_overlay(frame: &mut Frame, app: &App, area: Rect) {
    let popup_area = centered_rect_fixed(50, 20, area);

    // Clear the background
    frame.render_widget(Clear, popup_area);

    let help_text = vec![
        Line::from(Span::styled(
            " D&D Dungeon Master - Help ",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Input Modes:",
            Style::default().add_modifier(Modifier::UNDERLINED),
        )),
        Line::from("  i       Enter INSERT mode (type actions)"),
        Line::from("  :       Enter COMMAND mode"),
        Line::from("  Esc     Return to NORMAL mode"),
        Line::from(""),
        Line::from(Span::styled(
            "Navigation (NORMAL mode):",
            Style::default().add_modifier(Modifier::UNDERLINED),
        )),
        Line::from("  j/k or ↑/↓     Scroll up/down"),
        Line::from("  PgUp/PgDn      Scroll by page"),
        Line::from("  Ctrl+u/d       Scroll by half page"),
        Line::from("  g/G            Jump to top/bottom"),
        Line::from("  Tab            Cycle panel focus"),
        Line::from("  Mouse wheel    Scroll narrative"),
        Line::from(""),
        Line::from(Span::styled(
            "Overlays:",
            Style::default().add_modifier(Modifier::UNDERLINED),
        )),
        Line::from("  Shift+I        Show inventory"),
        Line::from("  Shift+C        Show character sheet"),
        Line::from("  Shift+Q        Show quest log"),
        Line::from(""),
        Line::from(Span::styled(
            "Actions:",
            Style::default().add_modifier(Modifier::UNDERLINED),
        )),
        Line::from("  'i' then type  Enter player action"),
        Line::from("  q              Quit"),
        Line::from(""),
        Line::from(Span::styled(
            "Commands:",
            Style::default().add_modifier(Modifier::UNDERLINED),
        )),
        Line::from("  :q      Quit"),
        Line::from("  :roll   Roll dice (e.g., :roll 2d6+3)"),
        Line::from("  :w      Save game"),
        Line::from(""),
        Line::from(Span::styled(
            "Press Esc or q to close",
            Style::default().add_modifier(Modifier::DIM),
        )),
    ];

    let block = Block::default()
        .title(" Help ")
        .borders(Borders::ALL)
        .border_style(app.theme.border_style(true));

    let paragraph = Paragraph::new(help_text)
        .block(block)
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, popup_area);
}

/// Render dice roll overlay
fn render_dice_overlay(
    frame: &mut Frame,
    app: &App,
    result: Option<&RollResult>,
    purpose: &str,
    dc: Option<i32>,
    area: Rect,
) {
    let popup_area = centered_rect_fixed(30, 15, area);

    // Clear the background
    frame.render_widget(Clear, popup_area);

    // Determine animation state based on whether we have a result
    let animation_state = if result.is_some() {
        DiceAnimationState::Complete
    } else {
        DiceAnimationState::Rolling {
            frame: app.animation_frame,
        }
    };

    let mut dice_widget = DiceRollWidget::new(&app.theme)
        .purpose(purpose)
        .dc(dc)
        .animation_state(animation_state);

    if let Some(r) = result {
        dice_widget = dice_widget.result(r);
    }

    frame.render_widget(dice_widget, popup_area);
}

/// Render inventory overlay
fn render_inventory_overlay(frame: &mut Frame, app: &App, area: Rect) {
    // Size the popup based on content
    let height = (app.world.inventory_items.len() as u16 + 8).min(area.height - 4).max(12);
    let popup_area = centered_rect_fixed(60, height, area);

    // Clear the background
    frame.render_widget(Clear, popup_area);

    let mut lines = vec![
        Line::from(Span::styled(
            " Inventory ",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];

    // Gold
    lines.push(Line::from(vec![
        Span::styled("Gold: ", Style::default().add_modifier(Modifier::DIM)),
        Span::styled(
            format!("{:.0} gp", app.world.gold),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
    ]));
    lines.push(Line::from(""));

    // Equipment section
    lines.push(Line::from(Span::styled(
        "Equipped:",
        Style::default().add_modifier(Modifier::UNDERLINED),
    )));

    let weapon_text = app
        .world
        .equipped_weapon.as_deref()
        .unwrap_or("(none)");
    lines.push(Line::from(vec![
        Span::styled("  Weapon: ", Style::default().add_modifier(Modifier::DIM)),
        Span::styled(weapon_text, Style::default().fg(Color::LightBlue)),
    ]));

    let armor_text = app
        .world
        .equipped_armor.as_deref()
        .unwrap_or("(none)");
    lines.push(Line::from(vec![
        Span::styled("  Armor: ", Style::default().add_modifier(Modifier::DIM)),
        Span::styled(armor_text, Style::default().fg(Color::Gray)),
    ]));

    lines.push(Line::from(""));

    // Items section
    lines.push(Line::from(Span::styled(
        "Items:",
        Style::default().add_modifier(Modifier::UNDERLINED),
    )));

    if app.world.inventory_items.is_empty() {
        lines.push(Line::from(Span::styled(
            "  (empty)",
            Style::default().add_modifier(Modifier::DIM),
        )));
    } else {
        for item in &app.world.inventory_items {
            let qty_str = if item.quantity > 1 {
                format!("{}x ", item.quantity)
            } else {
                String::new()
            };

            let item_color = if item.magical {
                Color::Magenta
            } else {
                match item.item_type {
                    dnd_core::world::ItemType::Weapon => Color::LightBlue,
                    dnd_core::world::ItemType::Armor => Color::Gray,
                    dnd_core::world::ItemType::Potion => Color::LightRed,
                    dnd_core::world::ItemType::Scroll => Color::LightYellow,
                    _ => Color::White,
                }
            };

            let value_str = if item.value_gp > 0.0 {
                format!(" ({:.0} gp)", item.value_gp * item.quantity as f32)
            } else {
                String::new()
            };

            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(format!("{}{}", qty_str, item.name), Style::default().fg(item_color)),
                Span::styled(value_str, Style::default().add_modifier(Modifier::DIM)),
            ]));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Press Esc or I to close",
        Style::default().add_modifier(Modifier::DIM),
    )));

    let block = Block::default()
        .title(" Inventory ")
        .borders(Borders::ALL)
        .border_style(app.theme.border_style(true));

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, popup_area);
}

/// Render character sheet overlay
fn render_character_sheet_overlay(frame: &mut Frame, app: &App, area: Rect) {
    use dnd_core::world::Ability;

    let popup_area = centered_rect_fixed(65, 25, area);

    // Clear the background
    frame.render_widget(Clear, popup_area);

    let mut lines = vec![
        Line::from(Span::styled(
            format!(" {} - Character Sheet ", app.world.player_name),
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];

    // Class and level
    let class_text = app
        .world
        .player_class
        .as_ref()
        .map(|c| format!("Level {} {}", app.world.player_level, c))
        .unwrap_or_else(|| format!("Level {}", app.world.player_level));
    lines.push(Line::from(vec![
        Span::styled("Class: ", Style::default().add_modifier(Modifier::DIM)),
        Span::styled(class_text, Style::default().add_modifier(Modifier::BOLD)),
        Span::raw("    "),
        Span::styled("Prof: ", Style::default().add_modifier(Modifier::DIM)),
        Span::styled(
            format!("+{}", app.world.proficiency_bonus),
            Style::default().fg(Color::Cyan),
        ),
    ]));
    lines.push(Line::from(""));

    // Ability scores section
    lines.push(Line::from(Span::styled(
        "Ability Scores:",
        Style::default().add_modifier(Modifier::UNDERLINED),
    )));

    let abilities = [
        (Ability::Strength, "STR"),
        (Ability::Dexterity, "DEX"),
        (Ability::Constitution, "CON"),
        (Ability::Intelligence, "INT"),
        (Ability::Wisdom, "WIS"),
        (Ability::Charisma, "CHA"),
    ];

    // Display abilities in two rows of three
    for chunk in abilities.chunks(3) {
        let mut spans = vec![Span::raw("  ")];
        for (i, (ability, abbrev)) in chunk.iter().enumerate() {
            let score = app.world.ability_scores.get(*ability);
            let modifier = app.world.ability_scores.modifier(*ability);
            let mod_str = if modifier >= 0 {
                format!("+{modifier}")
            } else {
                format!("{modifier}")
            };

            if i > 0 {
                spans.push(Span::raw("  "));
            }
            spans.push(Span::styled(
                format!("{abbrev}: "),
                Style::default().add_modifier(Modifier::DIM),
            ));
            spans.push(Span::styled(
                format!("{score:2}"),
                Style::default().add_modifier(Modifier::BOLD),
            ));
            spans.push(Span::styled(
                format!(" ({mod_str})"),
                Style::default().fg(if modifier >= 0 {
                    Color::Green
                } else {
                    Color::Red
                }),
            ));
        }
        lines.push(Line::from(spans));
    }

    lines.push(Line::from(""));

    // Combat stats
    lines.push(Line::from(Span::styled(
        "Combat:",
        Style::default().add_modifier(Modifier::UNDERLINED),
    )));

    let hp = &app.world.player_hp;
    let hp_color = app.theme.hp_color(hp.ratio());

    lines.push(Line::from(vec![
        Span::raw("  "),
        Span::styled("HP: ", Style::default().add_modifier(Modifier::DIM)),
        Span::styled(
            format!("{}/{}", hp.current, hp.maximum),
            Style::default().fg(hp_color),
        ),
        Span::raw("    "),
        Span::styled("AC: ", Style::default().add_modifier(Modifier::DIM)),
        Span::styled(
            format!("{}", app.world.player_ac),
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::raw("    "),
        Span::styled("Speed: ", Style::default().add_modifier(Modifier::DIM)),
        Span::styled(format!("{} ft", app.world.player_speed), Style::default()),
    ]));

    lines.push(Line::from(""));

    // Skill proficiencies (if any)
    if !app.world.skill_proficiencies.is_empty() {
        lines.push(Line::from(Span::styled(
            "Skill Proficiencies:",
            Style::default().add_modifier(Modifier::UNDERLINED),
        )));

        let mut skills: Vec<_> = app.world.skill_proficiencies.keys().collect();
        skills.sort_by_key(|s| s.name());

        let skill_names: Vec<String> = skills.iter().map(|s| s.name().to_string()).collect();
        // Wrap skills into lines of ~50 chars
        let mut current_line = String::from("  ");
        for (i, name) in skill_names.iter().enumerate() {
            if i > 0 {
                current_line.push_str(", ");
            }
            if current_line.len() + name.len() > 55 && !current_line.trim().is_empty() {
                lines.push(Line::from(Span::styled(
                    current_line.clone(),
                    Style::default().fg(Color::Cyan),
                )));
                current_line = String::from("  ");
            }
            current_line.push_str(name);
        }
        if !current_line.trim().is_empty() {
            lines.push(Line::from(Span::styled(
                current_line,
                Style::default().fg(Color::Cyan),
            )));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Press Esc or C to close",
        Style::default().add_modifier(Modifier::DIM),
    )));

    let block = Block::default()
        .title(" Character Sheet ")
        .borders(Borders::ALL)
        .border_style(app.theme.border_style(true));

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, popup_area);
}

/// Render quest log overlay
fn render_quest_log_overlay(frame: &mut Frame, app: &App, area: Rect) {
    use dnd_core::world::QuestStatus;

    let popup_area = centered_rect_fixed(60, 22, area);

    // Clear the background
    frame.render_widget(Clear, popup_area);

    let mut lines = vec![
        Line::from(Span::styled(
            " Quest Log ",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];

    // Separate quests by status
    let active: Vec<_> = app
        .world
        .quests
        .iter()
        .filter(|q| matches!(q.status, QuestStatus::Active))
        .collect();
    let completed: Vec<_> = app
        .world
        .quests
        .iter()
        .filter(|q| matches!(q.status, QuestStatus::Completed))
        .collect();
    let failed: Vec<_> = app
        .world
        .quests
        .iter()
        .filter(|q| matches!(q.status, QuestStatus::Failed | QuestStatus::Abandoned))
        .collect();

    // Active quests
    if !active.is_empty() {
        lines.push(Line::from(Span::styled(
            "Active Quests:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::UNDERLINED),
        )));

        for quest in &active {
            lines.push(Line::from(vec![
                Span::styled("  ● ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    &quest.name,
                    Style::default().add_modifier(Modifier::BOLD),
                ),
            ]));
            // Show description if not too long
            let desc: String = quest.description.chars().take(50).collect();
            let desc = if quest.description.chars().count() > 50 {
                format!("{desc}...")
            } else {
                desc
            };
            lines.push(Line::from(Span::styled(
                format!("    {desc}"),
                Style::default().add_modifier(Modifier::DIM),
            )));

            // Show objectives
            for obj in &quest.objectives {
                let marker = if obj.completed { "✓" } else { "○" };
                let style = if obj.completed {
                    Style::default().fg(Color::Green)
                } else if obj.optional {
                    Style::default().add_modifier(Modifier::DIM)
                } else {
                    Style::default()
                };
                let optional_tag = if obj.optional { " (optional)" } else { "" };
                lines.push(Line::from(Span::styled(
                    format!("    {} {}{}", marker, obj.description, optional_tag),
                    style,
                )));
            }
        }
        lines.push(Line::from(""));
    }

    // Completed quests
    if !completed.is_empty() {
        lines.push(Line::from(Span::styled(
            "Completed Quests:",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::UNDERLINED),
        )));

        for quest in &completed {
            lines.push(Line::from(vec![
                Span::styled("  ✓ ", Style::default().fg(Color::Green)),
                Span::styled(
                    &quest.name,
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::DIM),
                ),
            ]));
        }
        lines.push(Line::from(""));
    }

    // Failed/abandoned quests
    if !failed.is_empty() {
        lines.push(Line::from(Span::styled(
            "Failed Quests:",
            Style::default()
                .fg(Color::Red)
                .add_modifier(Modifier::UNDERLINED),
        )));

        for quest in &failed {
            lines.push(Line::from(vec![
                Span::styled("  ✗ ", Style::default().fg(Color::Red)),
                Span::styled(
                    &quest.name,
                    Style::default().fg(Color::Red).add_modifier(Modifier::DIM),
                ),
            ]));
        }
        lines.push(Line::from(""));
    }

    // No quests message
    if active.is_empty() && completed.is_empty() && failed.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  No quests yet. Explore the world!",
            Style::default().add_modifier(Modifier::DIM),
        )));
        lines.push(Line::from(""));
    }

    lines.push(Line::from(Span::styled(
        "Press Esc or Q to close",
        Style::default().add_modifier(Modifier::DIM),
    )));

    let block = Block::default()
        .title(" Quest Log ")
        .borders(Borders::ALL)
        .border_style(app.theme.border_style(true));

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, popup_area);
}

// ============================================================================
// Simplified Character Panel Widget (doesn't need full Character reference)
// ============================================================================

use ratatui::widgets::Widget;

/// Simplified character panel that works with WorldUpdate
pub struct SimplifiedCharacterWidget<'a> {
    world: &'a WorldUpdate,
    theme: &'a crate::ui::theme::GameTheme,
    focused: bool,
}

impl<'a> SimplifiedCharacterWidget<'a> {
    pub fn new(world: &'a WorldUpdate, theme: &'a crate::ui::theme::GameTheme) -> Self {
        Self {
            world,
            theme,
            focused: false,
        }
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }
}

impl Widget for SimplifiedCharacterWidget<'_> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        use ratatui::{
            layout::{Constraint, Direction, Layout},
            widgets::Gauge,
        };

        let block = Block::default()
            .title(format!(" {} ", self.world.player_name))
            .borders(Borders::ALL)
            .border_style(self.theme.border_style(self.focused));

        let inner = block.inner(area);
        block.render(area, buf);

        // Calculate how many rows we need for conditions/death saves
        let has_conditions = !self.world.conditions.is_empty();
        let is_dying = self.world.player_hp.current <= 0;

        // Split into sections
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Class/Level
                Constraint::Length(2), // HP bar
                Constraint::Length(2), // AC/Init/Speed
                Constraint::Length(if has_conditions { 2 } else { 0 }), // Conditions
                Constraint::Length(if is_dying { 2 } else { 0 }),       // Death saves
                Constraint::Length(2), // Equipment
                Constraint::Min(0),    // Spacer
            ])
            .split(inner);

        // Class and level
        let class_text = if let Some(ref class) = self.world.player_class {
            format!("Level {} {}", self.world.player_level, class)
        } else {
            format!("Level {}", self.world.player_level)
        };
        let class_line = Line::from(Span::styled(
            class_text,
            Style::default().add_modifier(Modifier::DIM),
        ));
        Paragraph::new(class_line).render(chunks[0], buf);

        // HP bar
        let hp = &self.world.player_hp;
        let hp_ratio = hp.ratio();
        let hp_color = self.theme.hp_color(hp_ratio);

        let hp_label = if hp.temporary > 0 {
            format!("HP: {}/{} (+{})", hp.current, hp.maximum, hp.temporary)
        } else {
            format!("HP: {}/{}", hp.current, hp.maximum)
        };

        let gauge = Gauge::default()
            .block(Block::default())
            .gauge_style(Style::default().bg(hp_color).fg(Color::Black))
            .ratio(hp_ratio as f64)
            .label(Span::styled(
                hp_label,
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            ));
        gauge.render(chunks[1], buf);

        // AC, Initiative, Speed
        let init = self.world.player_initiative;
        let init_str = if init >= 0 {
            format!("+{init}")
        } else {
            format!("{init}")
        };

        let combat_stats = vec![
            Line::from(vec![
                Span::raw("AC: "),
                Span::styled(
                    format!("{}", self.world.player_ac),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw("  Init: "),
                Span::styled(init_str, Style::default()),
            ]),
            Line::from(vec![
                Span::raw("Speed: "),
                Span::styled(format!("{} ft", self.world.player_speed), Style::default()),
            ]),
        ];
        Paragraph::new(combat_stats).render(chunks[2], buf);

        // Active conditions (if any)
        if has_conditions {
            let condition_names: Vec<String> = self
                .world
                .conditions
                .iter()
                .map(|c| c.name().to_string())
                .collect();
            let conditions_line = Line::from(vec![
                Span::styled("⚠ ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    condition_names.join(", "),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
            ]);
            Paragraph::new(vec![
                Line::from(Span::styled(
                    "Conditions:",
                    Style::default().add_modifier(Modifier::DIM),
                )),
                conditions_line,
            ])
            .render(chunks[3], buf);
        }

        // Death saves (if dying)
        if is_dying {
            let ds = &self.world.death_saves;
            let successes = "●".repeat(ds.successes as usize)
                + &"○".repeat(3 - ds.successes as usize);
            let failures = "●".repeat(ds.failures as usize)
                + &"○".repeat(3 - ds.failures as usize);

            let death_save_lines = vec![
                Line::from(Span::styled(
                    "DEATH SAVES",
                    Style::default()
                        .fg(Color::Red)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(vec![
                    Span::styled("✓", Style::default().fg(Color::Green)),
                    Span::raw(" "),
                    Span::styled(successes, Style::default().fg(Color::Green)),
                    Span::raw("  "),
                    Span::styled("✗", Style::default().fg(Color::Red)),
                    Span::raw(" "),
                    Span::styled(failures, Style::default().fg(Color::Red)),
                ]),
            ];
            Paragraph::new(death_save_lines).render(chunks[4], buf);
        }

        // Equipment and gold
        let weapon_text = self
            .world
            .equipped_weapon.as_deref()
            .unwrap_or("Unarmed");
        let armor_text = self
            .world
            .equipped_armor.as_deref()
            .unwrap_or("Unarmored");

        let equipment_lines = vec![
            Line::from(vec![
                Span::raw("Weapon: "),
                Span::styled(weapon_text, Style::default().fg(Color::LightBlue)),
            ]),
            Line::from(vec![
                Span::raw("Armor: "),
                Span::styled(armor_text, Style::default().fg(Color::Gray)),
                Span::raw("  "),
                Span::styled(
                    format!("{:.0} gp", self.world.gold),
                    Style::default().fg(Color::Yellow),
                ),
            ]),
        ];
        Paragraph::new(equipment_lines).render(chunks[5], buf);
    }
}
