use eframe::egui;
use crate::communication::ConsoleSeverity;
use crate::GcodeKitApp;

/// Shows the device console tab with real-time logging and severity filtering
pub fn show_device_console_tab(app: &mut GcodeKitApp, ui: &mut egui::Ui) {
    ui.vertical(|ui| {
        // Header section with title and action buttons
        ui.horizontal(|ui| {
            ui.heading("🖥️ Device Console");
            ui.separator();

            if ui.button("📋 Copy All").clicked() {
                let all_messages = app.machine.console_messages.join("\n");
                ui.ctx().copy_text(all_messages);
            }

            if ui.button("🗑️ Clear").clicked() {
                app.machine.console_messages.clear();
            }

            ui.label(format!(
                "Messages: {}",
                app.machine.console_messages.len()
            ));
        });

        ui.separator();

        // Severity filter section with checkboxes for each level
        ui.horizontal(|ui| {
            ui.label("📊 Filter by severity:");

            for &severity in ConsoleSeverity::all() {
                let is_active = app.machine.active_severities.contains(&severity);
                let mut new_state = is_active;

                let label = format!("☑ {}", severity.label());
                if ui.checkbox(&mut new_state, label).changed() {
                    if new_state {
                        app.machine.active_severities.push(severity);
                        app.machine.active_severities.sort();
                        app.machine.active_severities.dedup();
                    } else {
                        app.machine.active_severities.retain(|&s| s != severity);
                    }
                }
            }
        });

        ui.separator();

        // Console display area with auto-scroll
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .stick_to_bottom(true)
            .show(ui, |ui| {
                if app.machine.console_messages.is_empty() {
                    ui.weak("No messages yet. Connect to a device to see communication logs.");
                } else {
                    for message in &app.machine.console_messages {
                        // Determine color based on message content
                        let (color, icon) = if message.contains("[ERROR]") || message.contains("error:")
                        {
                            (egui::Color32::RED, "❌")
                        } else if message.contains("[WARN") || message.contains("WARNING") {
                            (egui::Color32::YELLOW, "⚠️")
                        } else if message.contains("[DEBUG]") {
                            (egui::Color32::GRAY, "🔍")
                        } else if message.contains("[CMD]") {
                            (egui::Color32::LIGHT_BLUE, "➡️")
                        } else if message.contains("[RES]") {
                            (egui::Color32::LIGHT_GREEN, "⬅️")
                        } else if message.contains("[TRC]") {
                            (egui::Color32::WHITE, "📝")
                        } else {
                            (egui::Color32::WHITE, "•")
                        };

                        ui.colored_label(
                            color,
                            format!("{} {}", icon, message),
                        );
                    }
                }
            });
    });
}
