use crate::GcodeKitApp;
use eframe::egui;

pub fn show_gcode_loading_widget(ui: &mut egui::Ui, app: &mut GcodeKitApp) {
    ui.group(|ui| {
        ui.label("G-code");

        // File loading
        ui.horizontal(|ui| {
            if ui.button("📁 Load File").clicked() {
                app.load_gcode_file();
            }
            ui.label(if app.gcode_filename.is_empty() {
                "No file loaded"
            } else {
                &app.gcode_filename
            });
        });

        // Send controls
        ui.horizontal(|ui| {
            if ui.button("📤 Send to Device").clicked() {
                app.send_gcode_to_device();
            }
            if ui.button("⏹️ Stop").clicked() {
                // TODO: Implement stop sending
            }
        });

        // Progress/status
        if !app.gcode_content.is_empty() {
            let lines = app.gcode_content.lines().count();
            ui.label(format!("{} lines loaded", lines));
        }
    });
}
