use crate::GcodeKitApp;
use eframe::egui;

/// Shows the jog control widget with a 4x4 grid layout matching the reference design.
///
/// Layout structure:
/// - Row 0: Top control icons and current position display
/// - Row 1: 3x3 directional buttons (Y axis vertical, X axis horizontal) + Emergency stop
/// - Row 2: Axis control buttons (Settings, X, Y, Z)
/// - Row 3: G-code macro buttons (G54, G55, G56, G57)
/// - Row 4: Additional buttons (NA placeholders for future use)
///
/// All buttons are theme-aware and respect the system's dark/light mode settings.
pub fn show_jog_widget(ui: &mut egui::Ui, app: &mut GcodeKitApp) {
    ui.group(|ui| {
        // Get theme-aware colors from the current visuals
        let visuals = ui.visuals();
        let button_bg = visuals.widgets.active.bg_fill;
        let emergency_bg = egui::Color32::from_rgb(220, 50, 50);
        let emergency_text = egui::Color32::WHITE;

        // Header row with controls and position display
        ui.vertical(|ui| {
            // Line 1: Title and settings button
            ui.horizontal(|ui| {
                ui.heading("Step Size | Jog Feed");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("⚙️").on_hover_text("Edit settings").clicked() {
                        // Settings button action
                    }
                });
            });

            // Line 2: Position and feed rate
            ui.horizontal(|ui| {
                ui.label(format!(
                    "Position: X:{:.1} Y:{:.1} Z:{:.1}     Feed: 2400.0 mm/min",
                    app.machine.current_position.x,
                    app.machine.current_position.y,
                    app.machine.current_position.z
                ));
            });
        });

        ui.separator();

        // Main grid of jog controls with theme-aware styling
        let button_size = 80.0;

        // Row 1: Blank, Y+, Blank, Z+
        ui.horizontal(|ui| {
            // Column 1: Blank
            ui.add_sized([button_size, button_size], egui::Label::new(""));

            // Column 2: Y+
            if ui
                .add_sized(
                    [button_size, button_size],
                    egui::Button::new("⬆\nY+").fill(button_bg),
                )
                .on_hover_text("Y+")
                .clicked()
            {
                app.machine
                    .communication
                    .jog_axis('Y', app.machine.jog_step_size);
            }

            // Column 3: Blank
            ui.add_sized([button_size, button_size], egui::Label::new(""));

            // Column 4: Z+
            if ui
                .add_sized(
                    [button_size, button_size],
                    egui::Button::new("⬆\nZ+").fill(button_bg),
                )
                .on_hover_text("Z+")
                .clicked()
            {
                app.machine
                    .communication
                    .jog_axis('Z', app.machine.jog_step_size);
            }
        });

        ui.add_space(10.0);

        // Row 2: X-, Home, X+, Stop
        ui.horizontal(|ui| {
            // Column 1: X-
            if ui
                .add_sized(
                    [button_size, button_size],
                    egui::Button::new("⬅\nX-").fill(button_bg),
                )
                .on_hover_text("X-")
                .clicked()
            {
                app.machine
                    .communication
                    .jog_axis('X', -app.machine.jog_step_size);
            }

            // Column 2: Home
            if ui
                .add_sized(
                    [button_size, button_size],
                    egui::Button::new("⌘\nHOME").fill(button_bg),
                )
                .on_hover_text("Home all axes")
                .clicked()
            {
                app.machine.communication.home_all_axes();
            }

            // Column 3: X+
            if ui
                .add_sized(
                    [button_size, button_size],
                    egui::Button::new("➡\nX+").fill(button_bg),
                )
                .on_hover_text("X+")
                .clicked()
            {
                app.machine
                    .communication
                    .jog_axis('X', app.machine.jog_step_size);
            }

            // Column 4: Stop
            if ui
                .add_sized(
                    [button_size, button_size],
                    egui::Button::new(egui::RichText::new("⛔\nSTOP").color(emergency_text))
                        .fill(emergency_bg),
                )
                .on_hover_text("EMERGENCY STOP")
                .clicked()
            {
                app.machine.communication.emergency_stop();
            }
        });

        ui.add_space(10.0);

        // Row 3: Blank, Y-, Blank, Z-
        ui.horizontal(|ui| {
            // Column 1: Blank
            ui.add_sized([button_size, button_size], egui::Label::new(""));

            // Column 2: Y-
            if ui
                .add_sized(
                    [button_size, button_size],
                    egui::Button::new("⬇\nY-").fill(button_bg),
                )
                .on_hover_text("Y-")
                .clicked()
            {
                app.machine
                    .communication
                    .jog_axis('Y', -app.machine.jog_step_size);
            }

            // Column 3: Blank
            ui.add_sized([button_size, button_size], egui::Label::new(""));

            // Column 4: Z-
            if ui
                .add_sized(
                    [button_size, button_size],
                    egui::Button::new("⬇\nZ-").fill(button_bg),
                )
                .on_hover_text("Z-")
                .clicked()
            {
                app.machine
                    .communication
                    .jog_axis('Z', -app.machine.jog_step_size);
            }
        });

        ui.add_space(10.0);
        ui.separator();

        // Alarm unlock button (only show when in alarm)
        if app.machine.realtime_status.state == crate::communication::grbl_status::MachineState::Alarm {
            ui.horizontal(|ui| {
                let alarm_bg = egui::Color32::from_rgb(255, 165, 0);
                let alarm_text = egui::Color32::BLACK;
                
                ui.label("⚠️ ALARM STATE:");
                
                if ui
                    .add_sized(
                        [200.0, 40.0],
                        egui::Button::new(
                            egui::RichText::new("🔓 UNLOCK DEVICE").size(14.0).color(alarm_text)
                        )
                        .fill(alarm_bg),
                    )
                    .on_hover_text("Clear alarm status and unlock device")
                    .clicked()
                {
                    app.machine.communication.clear_alarm();
                    app.machine.status_message = "Device alarm cleared".to_string();
                }
            });
            ui.add_space(5.0);
        }

        // Resume button (only show when in hold/pause state)
        if app.machine.realtime_status.state == crate::communication::grbl_status::MachineState::Hold {
            ui.horizontal(|ui| {
                let pause_bg = egui::Color32::from_rgb(100, 150, 200);
                let pause_text = egui::Color32::WHITE;
                
                ui.label("⏸️ PAUSED:");
                
                if ui
                    .add_sized(
                        [200.0, 40.0],
                        egui::Button::new(
                            egui::RichText::new("▶️ RESUME JOB").size(14.0).color(pause_text)
                        )
                        .fill(pause_bg),
                    )
                    .on_hover_text("Resume paused job execution")
                    .clicked()
                {
                    app.machine.communication.resume_job();
                    app.machine.status_message = "Job resumed".to_string();
                }
            });
            ui.add_space(5.0);
        }

        ui.separator();
        ui.horizontal(|ui| {
            ui.label("Step Size:");
            ui.add(
                egui::DragValue::new(&mut app.machine.jog_step_size)
                    .speed(0.1)
                    .range(0.1..=100.0)
                    .max_decimals(2),
            );

            egui::ComboBox::from_id_salt("jog_step_size_preset")
                .selected_text("")
                .width(60.0)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut app.machine.jog_step_size, 0.1, "0.1 mm");
                    ui.selectable_value(&mut app.machine.jog_step_size, 0.5, "0.5 mm");
                    ui.selectable_value(&mut app.machine.jog_step_size, 1.0, "1.0 mm");
                    ui.selectable_value(&mut app.machine.jog_step_size, 5.0, "5.0 mm");
                    ui.selectable_value(&mut app.machine.jog_step_size, 10.0, "10 mm");
                });
        });

        ui.add_space(5.0);

        // Command input field
        ui.horizontal(|ui| {
            ui.label("Command:");
            let response = ui.add(
                egui::TextEdit::singleline(&mut app.machine.manual_command)
                    .desired_width(ui.available_width() - 80.0),
            );

            if response.lost_focus()
                && ui.input(|i| i.key_pressed(egui::Key::Enter))
                && !app.machine.manual_command.trim().is_empty()
            {
                let cmd = app.machine.manual_command.clone();
                if let Err(e) = app.machine.communication.send_gcode_line(&cmd) {
                    app.machine.status_message = format!("Command error: {}", e);
                }
                app.machine.manual_command.clear();
            }

            if ui.button("Send").clicked() && !app.machine.manual_command.trim().is_empty() {
                let cmd = app.machine.manual_command.clone();
                if let Err(e) = app.machine.communication.send_gcode_line(&cmd) {
                    app.machine.status_message = format!("Command error: {}", e);
                }
                app.machine.manual_command.clear();
            }
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_show_jog_widget_compiles() {
        // This test ensures the function compiles and has the expected signature
        // Full UI testing would require egui context mocking
        let _fn_exists = show_jog_widget as fn(&mut egui::Ui, &mut GcodeKitApp);
    }
}
