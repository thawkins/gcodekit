use eframe::egui;

use crate::GcodeKitApp;
use crate::types::MoveType;

/// Shows the 3D visualizer tab
pub fn show_visualizer_3d_tab(app: &mut GcodeKitApp, ui: &mut egui::Ui) {
    // Handle keyboard shortcuts
    if ui.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::R))
        && app.gcode.selected_line.is_some()
        && *app.machine.communication.get_connection_state()
            == crate::communication::ConnectionState::Connected
    {
        if let Some(line_number) = app.gcode.selected_line {
            app.send_gcode_from_line(line_number);
        }
    }

    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            ui.label("3D Visualizer");
            ui.separator();
            if ui.button("🔄 Refresh View").clicked() {
                // TODO: Refresh visualization
            }
            if ui.button("📏 Fit to View").clicked() {
                // TODO: Fit view to content
            }
            ui.separator();
            let run_button_enabled = app.gcode.selected_line.is_some()
                && *app.machine.communication.get_connection_state()
                    == crate::communication::ConnectionState::Connected;
            if ui
                .add_enabled(
                    run_button_enabled,
                    egui::Button::new("▶️ Run from Selected Line"),
                )
                .clicked()
                && let Some(line_number) = app.gcode.selected_line
            {
                app.send_gcode_from_line(line_number);
            }
            ui.separator();
            ui.label("(Ctrl+R to run from selected line)");
        });

        ui.separator();

        // 2D visualization area
        let available_size = ui.available_size();
        let (rect, response) = ui.allocate_exact_size(available_size, egui::Sense::click());

        if app.gcode.gcode_content.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label("Load G-code to visualize toolpath");
            });
        } else {
            // Draw the paths
            let painter = ui.painter();
            let bounds = rect;

            // Find min/max for scaling (only X and Y for 2D visualization)
            let mut min_x = f32::INFINITY;
            let mut max_x = f32::NEG_INFINITY;
            let mut min_y = f32::INFINITY;
            let mut max_y = f32::NEG_INFINITY;

            for segment in &app.gcode.parsed_paths {
                min_x = min_x.min(segment.start.x).min(segment.end.x);
                max_x = max_x.max(segment.start.x).max(segment.end.x);
                min_y = min_y.min(segment.start.y).min(segment.end.y);
                max_y = max_y.max(segment.start.y).max(segment.end.y);
            }

            if !app.gcode.parsed_paths.is_empty() {
                let scale_x = bounds.width() / (max_x - min_x).max(1.0);
                let scale_y = bounds.height() / (max_y - min_y).max(1.0);
                let scale = scale_x.min(scale_y) * 0.9; // Leave some margin

                let offset_x =
                    bounds.min.x + (bounds.width() - (max_x - min_x) * scale) / 2.0 - min_x * scale;
                let offset_y = bounds.min.y + (bounds.height() - (max_y - min_y) * scale) / 2.0
                    - min_y * scale;

                for segment in &app.gcode.parsed_paths {
                    let start_pos = egui::pos2(
                        offset_x + segment.start.x * scale,
                        offset_y + segment.start.y * scale,
                    );
                    let end_pos = egui::pos2(
                        offset_x + segment.end.x * scale,
                        offset_y + segment.end.y * scale,
                    );

                    let color = match segment.move_type {
                        MoveType::Rapid => egui::Color32::BLUE,
                        MoveType::Feed => egui::Color32::GREEN,
                        MoveType::Arc => egui::Color32::YELLOW,
                    };

                    let is_selected = app.gcode.selected_line == Some(segment.line_number);
                    let stroke_width = if is_selected { 4.0 } else { 2.0 };
                    let stroke_color = if is_selected {
                        egui::Color32::WHITE
                    } else {
                        color
                    };

                    painter.line_segment(
                        [start_pos, end_pos],
                        egui::Stroke::new(stroke_width, stroke_color),
                    );
                }

                // Draw current machine position
                let current_screen_x = offset_x + app.machine.current_position.x * scale;
                let current_screen_y = offset_y + app.machine.current_position.y * scale;
                painter.circle_filled(
                    egui::pos2(current_screen_x, current_screen_y),
                    5.0,
                    egui::Color32::RED,
                );

                // Left-click to select segment
                if response.clicked_by(egui::PointerButton::Primary)
                    && let Some(click_pos) = response.interact_pointer_pos()
                {
                    // Find closest segment to click position
                    let mut closest_segment = None;
                    let mut min_distance = f32::INFINITY;

                    for segment in &app.gcode.parsed_paths {
                        let start_screen = egui::pos2(
                            offset_x + segment.start.x * scale,
                            offset_y + segment.start.y * scale,
                        );
                        let end_screen = egui::pos2(
                            offset_x + segment.end.x * scale,
                            offset_y + segment.end.y * scale,
                        );

                        // Distance to line segment (simplified as distance to midpoint)
                        let mid_x = (start_screen.x + end_screen.x) / 2.0;
                        let mid_y = (start_screen.y + end_screen.y) / 2.0;
                        let dx = click_pos.x - mid_x;
                        let dy = click_pos.y - mid_y;
                        let distance = (dx * dx + dy * dy).sqrt();

                        if distance < min_distance && distance < 20.0 {
                            // Within 20 pixels
                            min_distance = distance;
                            closest_segment = Some(segment.line_number);
                        }
                    }

                    app.gcode.selected_line = closest_segment;
                    if let Some(line) = app.gcode.selected_line {
                        app.machine.status_message = format!("Selected line {}", line + 1);
                    }
                }

                // Right-click to jog
                if response.clicked_by(egui::PointerButton::Secondary) {
                    if *app.machine.communication.get_connection_state()
                        == crate::communication::ConnectionState::Connected
                    {
                        if let Some(click_pos) = response.interact_pointer_pos() {
                            let gcode_x = (click_pos.x - offset_x) / scale;
                            let gcode_y = (click_pos.y - offset_y) / scale;
                            let delta_x = gcode_x - app.machine.current_position.x;
                            let delta_y = gcode_y - app.machine.current_position.y;
                            app.machine.communication.jog_axis('X', delta_x);
                            app.machine.communication.jog_axis('Y', delta_y);
                            app.machine.status_message =
                                format!("Jogging to X:{:.3} Y:{:.3}", gcode_x, gcode_y);
                        }
                    } else {
                        app.machine.status_message = "Not connected - cannot jog".to_string();
                    }
                }
            }

            ui.label(format!("Segments: {}", app.gcode.parsed_paths.len()));
            if let Some(sending_line) = app.gcode.sending_from_line {
                ui.colored_label(
                    egui::Color32::GREEN,
                    format!("Sending from line {}", sending_line + 1),
                );
            }
        }
    });
}
