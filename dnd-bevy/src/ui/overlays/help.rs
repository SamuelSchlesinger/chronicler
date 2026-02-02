//! Help overlay.

use bevy_egui::egui;

/// Render the help overlay.
pub fn render_help(ctx: &egui::Context) {
    let screen = ctx.screen_rect();
    let width = (screen.width() * 0.8).clamp(300.0, 450.0);
    let height = (screen.height() * 0.75).clamp(320.0, 480.0);

    egui::Window::new("Help")
        .collapsible(false)
        .resizable(true)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .default_size([width, height])
        .max_size([550.0, 600.0])
        .show(ctx, |ui| {
            ui.heading("D&D: AI Dungeon Master");
            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("How to Play");
                ui.label("Type natural language commands to interact with the world.");
                ui.label("The AI Dungeon Master will respond to your actions.");
                ui.add_space(10.0);

                ui.heading("Example Commands");
                ui.label("- \"I look around the room\"");
                ui.label("- \"I attack the goblin with my sword\"");
                ui.label("- \"I try to pick the lock\"");
                ui.label("- \"I cast fireball at the enemies\"");
                ui.label("- \"I search the chest\"");
                ui.add_space(10.0);

                ui.heading("Keyboard Shortcuts");
                ui.add_space(4.0);

                ui.label(
                    egui::RichText::new("Global:")
                        .strong()
                        .color(egui::Color32::from_rgb(218, 165, 32)),
                );
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Ctrl+Q / Cmd+Q").strong());
                    ui.label("- Quit game");
                });
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Ctrl+S / Cmd+S").strong());
                    ui.label("- Quick Save");
                });
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Escape").strong());
                    ui.label("- Close overlay / Cancel");
                });

                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new("Input:")
                        .strong()
                        .color(egui::Color32::from_rgb(218, 165, 32)),
                );
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Enter").strong());
                    ui.label("- Send command");
                });
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Up / Down").strong());
                    ui.label("- Browse command history");
                });

                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new("Overlays (when not typing):")
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

                ui.add_space(10.0);
                ui.heading("Tips");
                ui.label("- Be descriptive - the DM understands natural language");
                ui.label("- Check your inventory before adventures");
                ui.label("- Save often using Ctrl+S");
                ui.label("- Use the quick action buttons for common actions");
            });

            ui.separator();
            ui.label(
                egui::RichText::new("Press F1, ?, or Escape to close")
                    .small()
                    .color(egui::Color32::GRAY),
            );
        });
}
