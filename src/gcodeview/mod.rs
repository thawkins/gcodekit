use crate::GcodeKitApp;
use eframe::egui;

pub fn show_visualizer_ui(ui: &mut egui::Ui, app: &mut GcodeKitApp) {
    // Handle keyboard shortcuts
    if ui.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::R))
        && let Some(line_number) = app.gcode_editor.selected_line
        && app.communication.is_connected()
    {
        app.send_gcode_from_line(line_number);
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
            let run_button_enabled =
                app.gcode_editor.selected_line.is_some() && app.communication.is_connected();
            if ui
                .add_enabled(
                    run_button_enabled,
                    egui::Button::new("▶️ Run from Selected Line"),
                )
                .clicked()
                && let Some(line_number) = app.gcode_editor.selected_line
            {
                app.send_gcode_from_line(line_number);
            }
            ui.separator();

            // Simulation controls
            ui.label("Simulation:");
            if ui.button("▶️ Start").clicked() {
                app.start_simulation();
            }
            if ui
                .button(if app.simulation_paused {
                    "▶️ Play"
                } else {
                    "⏸️ Pause"
                })
                .clicked()
            {
                app.simulation_paused = !app.simulation_paused;
            }
            if ui.button("⏹️ Stop").clicked() {
                app.simulation_enabled = false;
                app.simulation_current_line = 0;
                app.simulation_paused = true;
            }
            if ui.button("🔄 Reset").clicked() {
                app.simulation_current_line = 0;
                app.simulation_paused = true;
            }

            ui.separator();
            ui.add(
                egui::Slider::new(&mut app.simulation_speed, 1.0..=1000.0)
                    .text("Speed (lines/sec)"),
            );
            ui.label(format!("Current Line: {}", app.simulation_current_line));

            ui.separator();
            ui.label("(Ctrl+R to run from selected line)");
        });

        ui.separator();

        // 2D visualization area
        let available_size = ui.available_size();
        let (rect, response) = ui.allocate_exact_size(available_size, egui::Sense::click());

        if app.gcode_editor.gcode_content.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label("Load G-code to visualize toolpath");
            });
        } else {
            // Draw the paths
            let painter = ui.painter();
            let bounds = rect;

            // Find min/max for scaling (X, Y, and Z for 3D visualization)
            let mut min_x = f32::INFINITY;
            let mut max_x = f32::NEG_INFINITY;
            let mut min_y = f32::INFINITY;
            let mut max_y = f32::NEG_INFINITY;
            let mut min_z = f32::INFINITY;
            let mut max_z = f32::NEG_INFINITY;

            for segment in &app.parsed_paths {
                min_x = min_x.min(segment.start.x).min(segment.end.x);
                max_x = max_x.max(segment.start.x).max(segment.end.x);
                min_y = min_y.min(segment.start.y).min(segment.end.y);
                max_y = max_y.max(segment.start.y).max(segment.end.y);
                min_z = min_z.min(segment.start.z).min(segment.end.z);
                max_z = max_z.max(segment.start.z).max(segment.end.z);
            }

            if !app.parsed_paths.is_empty() {
                let scale_x = bounds.width() / (max_x - min_x).max(1.0);
                let scale_y = bounds.height() / (max_y - min_y).max(1.0);
                let scale = scale_x.min(scale_y) * 0.9; // Leave some margin

                let offset_x =
                    bounds.min.x + (bounds.width() - (max_x - min_x) * scale) / 2.0 - min_x * scale;
                let offset_y = bounds.min.y + (bounds.height() - (max_y - min_y) * scale) / 2.0
                    - min_y * scale;

                // Determine how many segments to show based on simulation state
                let max_segments = if app.simulation_enabled && !app.simulation_paused {
                    app.simulation_current_line.min(app.parsed_paths.len())
                } else {
                    app.parsed_paths.len()
                };

                for (i, segment) in app.parsed_paths.iter().enumerate() {
                    if i >= max_segments {
                        break;
                    }

                    let start_pos = egui::pos2(
                        offset_x + segment.start.x * scale,
                        offset_y + segment.start.y * scale,
                    );
                    let end_pos = egui::pos2(
                        offset_x + segment.end.x * scale,
                        offset_y + segment.end.y * scale,
                    );

                    let base_color = match segment.move_type {
                        crate::MoveType::Rapid => egui::Color32::BLUE,
                        crate::MoveType::Feed => egui::Color32::GREEN,
                        crate::MoveType::Arc => egui::Color32::YELLOW,
                    };

                    // Apply Z-based coloring for 3D visualization
                    let color = if max_z > min_z {
                        let avg_z = (segment.start.z + segment.end.z) / 2.0;
                        let z_ratio = (avg_z - min_z) / (max_z - min_z);
                        // Interpolate from blue (low Z) to red (high Z)
                        let r = (z_ratio * 255.0) as u8;
                        let b = ((1.0 - z_ratio) * 255.0) as u8;
                        let g = 0;
                        egui::Color32::from_rgb(r, g, b)
                    } else {
                        base_color
                    };

                    let is_selected = app.gcode_editor.selected_line == Some(segment.line_number);
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
                let current_screen_x = offset_x + app.current_position.x * scale;
                let current_screen_y = offset_y + app.current_position.y * scale;
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

                    for segment in &app.parsed_paths {
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

                    app.gcode_editor.selected_line = closest_segment;
                    if let Some(line) = app.gcode_editor.selected_line {
                        app.status_message = format!("Selected line {}", line + 1);
                    }
                }

                // Right-click to jog
                if response.clicked_by(egui::PointerButton::Secondary) {
                    if app.communication.is_connected() {
                        if let Some(click_pos) = response.interact_pointer_pos() {
                            let gcode_x = (click_pos.x - offset_x) / scale;
                            let gcode_y = (click_pos.y - offset_y) / scale;
                            let delta_x = gcode_x - app.current_position.x;
                            let delta_y = gcode_y - app.current_position.y;
                            app.jog_axis('X', delta_x);
                            app.jog_axis('Y', delta_y);
                            app.status_message =
                                format!("Jogging to X:{:.3} Y:{:.3}", gcode_x, gcode_y);
                        }
                    } else {
                        app.status_message = "Not connected - cannot jog".to_string();
                    }
                }
            }

            ui.label(format!("Segments: {}", app.parsed_paths.len()));
            if max_z > min_z {
                ui.label(format!(
                    "Z range: {:.3} to {:.3} (color-coded: blue=low Z, red=high Z)",
                    min_z, max_z
                ));
            }

            // Show rotary axes information if present
            let has_rotary_axes = app.parsed_paths.iter().any(|segment|
                segment.start.a.is_some() || segment.start.b.is_some() ||
                segment.start.c.is_some() || segment.start.d.is_some()
            );

            if has_rotary_axes {
                ui.separator();
                ui.label("🔄 Multi-axis support detected");
                ui.label("Rotary axes (A,B,C,D) are parsed and tracked");
                ui.label("Current position includes rotary coordinates");
            }

            if let Some(sending_line) = app.sending_from_line {
                ui.colored_label(
                    egui::Color32::GREEN,
                    format!("Sending from line {}", sending_line + 1),
                );
            }
            if let Some(sending_line) = app.sending_from_line {
                ui.colored_label(
                    egui::Color32::GREEN,
                    format!("Sending from line {}", sending_line + 1),
                );
            }
        }

        ui.separator();
        ui.label("3D visualization: Z-axis represented by color gradient (blue=low Z, red=high Z)");
    });
}
