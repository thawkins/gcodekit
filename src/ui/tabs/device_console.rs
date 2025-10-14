use eframe::egui;

use crate::GcodeKitApp;

/// Shows the device console tab
pub fn show_device_console_tab(app: &mut GcodeKitApp, ui: &mut egui::Ui) {
    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            ui.label("Device Console");
            ui.separator();
            if ui.button("🗑️ Clear").clicked() {
                app.machine.console_messages.clear();
            }
        });

        ui.separator();

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .stick_to_bottom(true)
            .show(ui, |ui| {
                for message in &app.machine.console_messages {
                    ui.label(message);
                }
                if app.machine.console_messages.is_empty() {
                    ui.weak("No messages yet. Connect to a device to see communication logs.");
                }
            });
    });
}
