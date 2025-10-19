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
    ui.vertical(|ui| {
        // Line 1: Refresh button and G-code controls
        ui.horizontal(|ui| {
            // Quick action buttons
            ui.add_space(2.0);
            if ui.button("🔄 Refresh").clicked() {
                app.machine.communication.refresh_ports();
            }

            ui.separator();

            // G-code loading section
            ui.group(|ui| {
                ui.label("G-code");

                // File loading and saving
                ui.horizontal(|ui| {
                    if ui.button("📁 Load File").clicked() {
                        app.load_gcode_file();
                    }

                    // Check if editor has content
                    let has_content = !app.gcode_editor.buffer.get_content().is_empty();
                    let has_filepath = app.gcode_editor.current_file_path.is_some();

                    // Save button - enabled only if there's content and a filepath
                    if ui
                        .add_enabled(has_content && has_filepath, egui::Button::new("💾 Save"))
                        .clicked()
                    {
                        if let Err(e) = app.gcode_editor.save_gcode_file() {
                            app.machine.status_message = format!("Save failed: {}", e);
                        } else {
                            app.machine.status_message = "File saved successfully".to_string();
                        }
                    }

                    // Save As button - enabled only if there's content
                    if ui
                        .add_enabled(has_content, egui::Button::new("💾 Save As..."))
                        .clicked()
                    {
                        if let Err(e) = app.gcode_editor.save_gcode_file_as() {
                            app.machine.status_message = format!("Save failed: {}", e);
                        } else {
                            app.machine.status_message = "File saved successfully".to_string();
                        }
                    }

                    ui.label(if app.gcode.gcode_filename.is_empty() {
                        "No file loaded"
                    } else {
                        &app.gcode.gcode_filename
                    });
                });

                // Send controls
                ui.horizontal(|ui| {
                    let is_sending = app.gcode.is_sending;
                    
                    if ui.add_enabled(!is_sending, egui::Button::new("📤 Send to Device")).clicked() {
                        app.send_gcode(&app.gcode.gcode_content.clone());
                    }
                    if ui.add_enabled(is_sending, egui::Button::new("⏹️ Stop")).clicked() {
                        app.stop_sending_gcode();
                    }
                    
                    if is_sending {
                        ui.label(format!("Sending... ({}/{})", 
                            app.gcode.current_line_sending,
                            app.gcode.gcode_content.lines().count()
                        ));
                    }
                });

                // Progress/status
                if !app.gcode.gcode_content.is_empty() {
                    let lines = app.gcode.gcode_content.lines().count();
                    ui.label(format!("{} lines loaded", lines));
                }
            });
        });

        ui.separator();

        // Line 2: Status indicators
        ui.horizontal(|ui| {
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
    });

    ui.separator();
}
