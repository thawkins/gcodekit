/// 3D Visualizer Tab
///
/// Enhanced 3D visualization of toolpaths with real-time camera controls,
/// machine position tracking, and stock visualization.

use crate::types::MoveType;
use crate::visualization::{
    draw_3d_grid, draw_3d_line, draw_machine_position, draw_stock, Visualizer3DState,
};
use crate::GcodeKitApp;
use eframe::egui;

/// Shows the enhanced 3D visualizer tab
pub fn show_visualizer_3d_tab(app: &mut GcodeKitApp, ui: &mut egui::Ui) {
    // Keyboard shortcut: Ctrl+R -> run from selected line
    if ui.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::R)) {
        if let Some(line_number) = app.gcode.selected_line {
            if *app.machine.communication.get_connection_state()
                == crate::communication::ConnectionState::Connected
            {
                app.send_gcode_from_line(line_number);
            }
        }
    }

    // Create visualizer state
    let mut vis_state = Visualizer3DState::default();

    // Camera controls
    ui.horizontal(|ui| {
        ui.label("🎥 3D Visualizer");
        ui.separator();

        if ui.button("🏠 Reset Camera").clicked() {
            vis_state.reset_camera();
        }
        if ui.button("📐 Fit View").clicked() {
            vis_state.fit_to_view();
        }
    });

    // Camera adjustment controls
    ui.horizontal(|ui| {
        ui.label("Pitch:");
        ui.add(egui::Slider::new(&mut vis_state.camera_pitch, -90.0..=90.0).step_by(5.0));

        ui.label("Yaw:");
        ui.add(egui::Slider::new(&mut vis_state.camera_yaw, 0.0..=360.0).step_by(5.0));

        ui.label("Zoom:");
        ui.add(egui::Slider::new(&mut vis_state.zoom, 0.1..=5.0).step_by(0.1));
    });

    // Display options
    ui.horizontal(|ui| {
        ui.label("Display:");
        ui.checkbox(&mut vis_state.show_grid, "Grid");
        ui.checkbox(&mut vis_state.show_stock, "Stock");
        ui.checkbox(&mut vis_state.show_machine_position, "Position");
    });

    // Toolpath visibility
    ui.horizontal(|ui| {
        ui.label("Toolpath:");
        ui.checkbox(&mut vis_state.show_rapid_moves, "Rapid (blue)");
        ui.checkbox(&mut vis_state.show_feed_moves, "Feed (green)");
        ui.checkbox(&mut vis_state.show_arc_moves, "Arc (yellow)");
    });

    // Stock dimensions
    ui.horizontal(|ui| {
        ui.label("Stock (X × Y × Z):");
        ui.add(egui::DragValue::new(&mut vis_state.stock_x).speed(1.0));
        ui.label("×");
        ui.add(egui::DragValue::new(&mut vis_state.stock_y).speed(1.0));
        ui.label("×");
        ui.add(egui::DragValue::new(&mut vis_state.stock_z).speed(1.0));
        ui.label("mm");
    });

    ui.separator();

    // Visualization area
    let available_size = ui.available_size();
    let (rect, response) = ui.allocate_exact_size(available_size, egui::Sense::click());

    if app.gcode.gcode_content.is_empty() {
        ui.centered_and_justified(|ui| ui.label("📄 Load G-code to visualize toolpath"));
        return;
    }

    let painter = ui.painter();
    let center = rect.center();

    // Draw 3D scene
    draw_3d_grid(&painter, &vis_state, center, 100.0, 20.0);
    draw_stock(&painter, &vis_state, center);

    // Draw toolpath segments
    for segment in &app.gcode_editor.parsed_paths {
        if (segment.move_type == MoveType::Rapid && !vis_state.show_rapid_moves)
            || (segment.move_type == MoveType::Feed && !vis_state.show_feed_moves)
            || (segment.move_type == MoveType::Arc && !vis_state.show_arc_moves)
        {
            continue;
        }

        let is_selected = app.gcode.selected_line == Some(segment.line_number);

        let (color, width) = if is_selected {
            (egui::Color32::from_rgb(255, 128, 0), 3.0)
        } else {
            match segment.move_type {
                MoveType::Rapid => (egui::Color32::BLUE, 1.5),
                MoveType::Feed => (egui::Color32::GREEN, 1.5),
                MoveType::Arc => (egui::Color32::YELLOW, 1.5),
            }
        };

        draw_3d_line(
            &painter,
            &vis_state,
            center,
            segment.start.x,
            segment.start.y,
            segment.start.z,
            segment.end.x,
            segment.end.y,
            segment.end.z,
            egui::Stroke::new(width, color),
        );
    }

    // Draw machine position
    draw_machine_position(
        &painter,
        &vis_state,
        center,
        app.machine.current_position.x,
        app.machine.current_position.y,
        app.machine.current_position.z,
    );

    // Click to select line
    if response.clicked() {
        if let Some(pos) = response.interact_pointer_pos() {
            let mut closest_distance = f32::MAX;
            let mut closest_line = None;

            for segment in &app.gcode_editor.parsed_paths {
                let start = vis_state.project_to_2d(
                    segment.start.x,
                    segment.start.y,
                    segment.start.z,
                    center,
                );
                let end = vis_state.project_to_2d(
                    segment.end.x,
                    segment.end.y,
                    segment.end.z,
                    center,
                );

                let distance = distance_point_to_segment(pos, start, end);
                if distance < closest_distance && distance < 20.0 {
                    closest_distance = distance;
                    closest_line = Some(segment.line_number);
                }
            }

            if let Some(line) = closest_line {
                app.gcode.selected_line = Some(line);
            }
        }
    }

    ui.separator();

    // Step-through controls
    ui.horizontal(|ui| {
        ui.label("Navigate:");
        if ui.button("⏮️ First").clicked() && !app.gcode_editor.parsed_paths.is_empty() {
            app.gcode.selected_line = Some(app.gcode_editor.parsed_paths[0].line_number);
        }

        if ui.button("◀️ Prev").clicked() {
            if let Some(current) = app.gcode.selected_line {
                for segment in app.gcode_editor.parsed_paths.iter().rev() {
                    if segment.line_number < current {
                        app.gcode.selected_line = Some(segment.line_number);
                        break;
                    }
                }
            }
        }

        if ui.button("▶️ Next").clicked() {
            if let Some(current) = app.gcode.selected_line {
                for segment in &app.gcode_editor.parsed_paths {
                    if segment.line_number > current {
                        app.gcode.selected_line = Some(segment.line_number);
                        break;
                    }
                }
            } else if !app.gcode_editor.parsed_paths.is_empty() {
                app.gcode.selected_line = Some(app.gcode_editor.parsed_paths[0].line_number);
            }
        }

        if ui.button("⏭️ Last").clicked() && !app.gcode_editor.parsed_paths.is_empty() {
            if let Some(last_path) = app.gcode_editor.parsed_paths.last() {
                app.gcode.selected_line = Some(last_path.line_number);
            }
        }

        ui.separator();

        let run_enabled = app.gcode.selected_line.is_some()
            && *app.machine.communication.get_connection_state()
                == crate::communication::ConnectionState::Connected;
        if ui
            .add_enabled(run_enabled, egui::Button::new("▶️ Run"))
            .clicked()
        {
            if let Some(line) = app.gcode.selected_line {
                app.send_gcode_from_line(line);
            }
        }

        ui.label("(Ctrl+R)");
    });

    // Status info
    ui.horizontal(|ui| {
        ui.label(format!("Segments: {}", app.gcode_editor.parsed_paths.len()));
        if let Some(selected) = app.gcode.selected_line {
            ui.colored_label(
                egui::Color32::from_rgb(255, 128, 0),
                format!("Selected: Line {}", selected + 1),
            );
        }
    });

    ui.label("💡 Tip: Click on toolpath to select, drag camera controls to rotate view");
}

/// Calculate distance from point to line segment
fn distance_point_to_segment(point: egui::Pos2, seg_start: egui::Pos2, seg_end: egui::Pos2) -> f32 {
    let dx = seg_end.x - seg_start.x;
    let dy = seg_end.y - seg_start.y;
    let length_squared = dx * dx + dy * dy;

    if length_squared == 0.0 {
        return ((point.x - seg_start.x).powi(2) + (point.y - seg_start.y).powi(2)).sqrt();
    }

    let t = ((point.x - seg_start.x) * dx + (point.y - seg_start.y) * dy) / length_squared;
    let t = t.clamp(0.0, 1.0);

    let proj_x = seg_start.x + t * dx;
    let proj_y = seg_start.y + t * dy;

    ((point.x - proj_x).powi(2) + (point.y - proj_y).powi(2)).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_show_visualizer_3d_tab_compiles() {
        let _fn_exists = show_visualizer_3d_tab as fn(&mut GcodeKitApp, &mut egui::Ui);
    }

    #[test]
    fn test_distance_calculation() {
        let point = egui::pos2(50.0, 50.0);
        let start = egui::pos2(0.0, 0.0);
        let end = egui::pos2(100.0, 100.0);

        let dist = distance_point_to_segment(point, start, end);
        // Distance should be reasonable
        assert!(dist >= 0.0 && dist < 100.0);
    }
}
