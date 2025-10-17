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
        let button_size = 60.0;
        
        // Row 1: Directional controls (3 columns × 3 rows)
        ui.horizontal(|ui| {
            // Column 1: Y+ and Y- controls
            ui.vertical(|ui| {
                // Y+
                if ui.add_sized(
                    [button_size, button_size],
                    egui::Button::new("▲\nY+").fill(button_bg)
                ).on_hover_text("Y+").clicked() {
                    app.machine.communication.jog_axis('Y', app.machine.jog_step_size);
                }
                
                // Home button (center)
                if ui.add_sized(
                    [button_size, button_size],
                    egui::Button::new("⌘\nHOME").fill(button_bg)
                ).on_hover_text("Home all axes").clicked() {
                    app.machine.communication.home_all_axes();
                }
                
                // Y-
                if ui.add_sized(
                    [button_size, button_size],
                    egui::Button::new("▼\nY-").fill(button_bg)
                ).on_hover_text("Y-").clicked() {
                    app.machine.communication.jog_axis('Y', -app.machine.jog_step_size);
                }
            });

            // Column 2: Z+ and Z- controls
            ui.vertical(|ui| {
                // Z+
                if ui.add_sized(
                    [button_size, button_size],
                    egui::Button::new("⬆\nZ+").fill(button_bg)
                ).on_hover_text("Z+").clicked() {
                    app.machine.communication.jog_axis('Z', app.machine.jog_step_size);
                }
                
                // Empty space (was HOME, now moved to column 1)
                ui.add_sized(
                    [button_size, button_size],
                    egui::Label::new("")
                );
                
                // Z-
                if ui.add_sized(
                    [button_size, button_size],
                    egui::Button::new("⬇\nZ-").fill(button_bg)
                ).on_hover_text("Z-").clicked() {
                    app.machine.communication.jog_axis('Z', -app.machine.jog_step_size);
                }
            });

            // Column 3: X+ controls
            ui.vertical(|ui| {
                // Empty space
                ui.add_sized(
                    [button_size, button_size],
                    egui::Label::new("")
                );
                
                // X+
                if ui.add_sized(
                    [button_size, button_size],
                    egui::Button::new("▶\nX+").fill(button_bg)
                ).on_hover_text("X+").clicked() {
                    app.machine.communication.jog_axis('X', app.machine.jog_step_size);
                }
                
                // Empty space
                ui.add_sized(
                    [button_size, button_size],
                    egui::Label::new("")
                );
            });

            // Column 4: Emergency stop
            ui.vertical(|ui| {
                // Empty space
                ui.add_sized(
                    [button_size, button_size],
                    egui::Label::new("")
                );
                
                // Emergency stop - red circle with white text
                if ui.add_sized(
                    [button_size, button_size],
                    egui::Button::new(
                        egui::RichText::new("⛔\nSTOP").color(emergency_text)
                    ).fill(emergency_bg)
                ).on_hover_text("EMERGENCY STOP").clicked() {
                    app.machine.communication.emergency_stop();
                }
                
                // Empty space
                ui.add_sized(
                    [button_size, button_size],
                    egui::Label::new("")
                );
            });
        });

        ui.add_space(10.0);

        // Row 2: X- and axis control buttons
        ui.horizontal(|ui| {
            ui.set_width(ui.available_width());
            
            // X- button (first position)
            if ui.add_sized(
                [button_size, button_size],
                egui::Button::new("◀\nX-").fill(button_bg)
            ).on_hover_text("X-").clicked() {
                app.machine.communication.jog_axis('X', -app.machine.jog_step_size);
            }
            
            if ui.add_sized(
                [button_size, button_size],
                egui::Button::new("↔️").fill(button_bg)
            ).on_hover_text("X Axis").clicked() {
                // X axis info
            }
            
            if ui.add_sized(
                [button_size, button_size],
                egui::Button::new("↕️").fill(button_bg)
            ).on_hover_text("Y Axis").clicked() {
                // Y axis info
            }
            
            if ui.add_sized(
                [button_size, button_size],
                egui::Button::new("⬆️").fill(button_bg)
            ).on_hover_text("Z Axis").clicked() {
                // Z axis info
            }
        });

        ui.add_space(10.0);

        // Row 3: G-code macro buttons (G54, G55, G56, G57)
        ui.horizontal(|ui| {
            ui.set_width(ui.available_width());
            
            if ui.add_sized(
                [button_size, button_size],
                egui::Button::new("📍1").fill(button_bg)
            ).on_hover_text("G54 Workspace").clicked() {
                let _ = app.machine.communication.send_gcode_line("G54");
            }
            
            if ui.add_sized(
                [button_size, button_size],
                egui::Button::new("📍2").fill(button_bg)
            ).on_hover_text("G55 Workspace").clicked() {
                let _ = app.machine.communication.send_gcode_line("G55");
            }
            
            if ui.add_sized(
                [button_size, button_size],
                egui::Button::new("📍3").fill(button_bg)
            ).on_hover_text("G56 Workspace").clicked() {
                let _ = app.machine.communication.send_gcode_line("G56");
            }
            
            if ui.add_sized(
                [button_size, button_size],
                egui::Button::new("📍4").fill(button_bg)
            ).on_hover_text("G57 Workspace").clicked() {
                let _ = app.machine.communication.send_gcode_line("G57");
            }
        });

        ui.add_space(10.0);

        // Row 4: Placeholder buttons for future expansion
        ui.horizontal(|ui| {
            ui.set_width(ui.available_width());
            
            for i in 0..4 {
                ui.add_sized(
                    [button_size, button_size],
                    egui::Button::new(format!("{}️", if i == 0 { "➕" } else if i == 1 { "➖" } else if i == 2 { "⚙️" } else { "📋" })).fill(button_bg)
                );
            }
        });

        ui.add_space(10.0);
        ui.separator();

        // Step size control section
        ui.horizontal(|ui| {
            ui.label("Step Size:");
            ui.add(egui::DragValue::new(&mut app.machine.jog_step_size)
                .speed(0.1)
                .range(0.1..=100.0)
                .max_decimals(2));
            
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
                    .desired_width(ui.available_width() - 80.0)
            );
            
            if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                if !app.machine.manual_command.trim().is_empty() {
                    let cmd = app.machine.manual_command.clone();
                    if let Err(e) = app.machine.communication.send_gcode_line(&cmd) {
                        app.machine.status_message = format!("Command error: {}", e);
                    }
                    app.machine.manual_command.clear();
                }
            }
            
            if ui.button("Send").clicked() {
                if !app.machine.manual_command.trim().is_empty() {
                    let cmd = app.machine.manual_command.clone();
                    if let Err(e) = app.machine.communication.send_gcode_line(&cmd) {
                        app.machine.status_message = format!("Command error: {}", e);
                    }
                    app.machine.manual_command.clear();
                }
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
