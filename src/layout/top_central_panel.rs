use crate::app::GcodeKitApp;
use crate::communication::ConnectionState;
use egui;

/// Renders the top central panel above the tabbed content.
/// Provides quick access to common actions, G-code loading, and status information.
///
/// # Arguments
/// * `app` - Mutable reference to the GcodeKitApp instance
/// * `ui` - The egui UI context for rendering
pub fn show_top_central_panel(app: &mut GcodeKitApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        // Quick action buttons
        if ui.button("🔄 Refresh").clicked() {
            app.machine.communication.refresh_ports();
        }

        ui.separator();

        // G-code loading section
        ui.group(|ui| {
            ui.label("G-code");

            // File loading
            ui.horizontal(|ui| {
                if ui.button("📁 Load File").clicked() {
                    app.load_gcode_file();
                }
                ui.label(if app.gcode.gcode_filename.is_empty() {
                    "No file loaded"
                } else {
                    &app.gcode.gcode_filename
                });
            });

            // Send controls
            ui.horizontal(|ui| {
                if ui.button("📤 Send to Device").clicked() {
                    app.send_gcode(&app.gcode.gcode_content.clone());
                }
                if ui.button("⏹️ Stop").clicked() {
                    // TODO: Implement stop sending
                }
            });

            // Progress/status
            if !app.gcode.gcode_content.is_empty() {
                let lines = app.gcode.gcode_content.lines().count();
                ui.label(format!("{} lines loaded", lines));
            }
        });

        ui.separator();

        // Status indicators
        ui.label(format!("Status: {}", app.machine.status_message));

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let connection_text = match *app.machine.communication.get_connection_state() {
                ConnectionState::Connected => "🟢 Connected",
                ConnectionState::Connecting => "🟡 Connecting...",
                ConnectionState::Disconnected => "🔴 Disconnected",
                ConnectionState::Error => "🔴 Error",
                ConnectionState::Recovering => "🟡 Recovering...",
            };
            ui.label(connection_text);

            // Current position
            ui.label(format!(
                "Position: {}",
                app.machine.current_position.format()
            ));
        });
    });

    ui.separator();
}
