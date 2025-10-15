use eframe::egui;

use crate::types::MoveType;
use crate::GcodeKitApp;

/// Shows the 3D visualizer tab (simplified, avoids deep nested closures to fix parsing errors)
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

    // Header row with step-through controls
    ui.horizontal(|ui| {
        ui.label("3D Visualizer");
        ui.separator();
        
        // Step-through controls
        ui.label("Step-through:");
        if ui.button("⏮️ First").clicked() {
            if !app.gcode_editor.parsed_paths.is_empty() {
                app.gcode.selected_line = Some(app.gcode_editor.parsed_paths[0].line_number);
            }
        }
        
        if ui.button("◀️ Prev").clicked() {
            if let Some(current) = app.gcode.selected_line {
                // Find previous segment
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
                // Find next segment
                for segment in &app.gcode_editor.parsed_paths {
                    if segment.line_number > current {
                        app.gcode.selected_line = Some(segment.line_number);
                        break;
                    }
                }
            } else if !app.gcode_editor.parsed_paths.is_empty() {
                // Start from first if nothing selected
                app.gcode.selected_line = Some(app.gcode_editor.parsed_paths[0].line_number);
            }
        }
        
        if ui.button("⏭️ Last").clicked() {
            if !app.gcode_editor.parsed_paths.is_empty() {
                app.gcode.selected_line = Some(app.gcode_editor.parsed_paths.last().unwrap().line_number);
            }
        }
        
        ui.separator();
        if ui.button("🔄 Refresh View").clicked() {}
        if ui.button("📏 Fit to View").clicked() {}
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
        {
            if let Some(line_number) = app.gcode.selected_line {
                app.send_gcode_from_line(line_number);
            }
        }
        ui.separator();
        ui.label("(Ctrl+R to run)");
    });

    ui.separator();

    // Visualization area with click handling
    let available_size = ui.available_size();
    let (rect, response) = ui.allocate_exact_size(available_size, egui::Sense::click());

    if app.gcode.gcode_content.is_empty() {
        ui.centered_and_justified(|ui| ui.label("Load G-code to visualize toolpath"));
        return;
    }

    // Handle click to select line
    if response.clicked() {
        if let Some(pos) = response.interact_pointer_pos() {
            // Find closest segment to click position
            let mut closest_segment_idx = None;
            let mut min_distance = f32::MAX;

            for (idx, segment) in app.gcode_editor.parsed_paths.iter().enumerate() {
                let start = egui::pos2(rect.min.x + segment.start.x, rect.min.y + segment.start.y);
                let end = egui::pos2(rect.min.x + segment.end.x, rect.min.y + segment.end.y);
                
                // Calculate distance from click to line segment
                let distance = distance_point_to_segment(pos, start, end);
                
                if distance < min_distance && distance < 20.0 {  // 20px threshold
                    min_distance = distance;
                    closest_segment_idx = Some(idx);
                }
            }

            // Select the line if segment found
            if let Some(idx) = closest_segment_idx {
                app.gcode.selected_line = Some(app.gcode_editor.parsed_paths[idx].line_number);
            }
        }
    }

    let painter = ui.painter();
    
    // Draw all segments
    for (idx, segment) in app.gcode_editor.parsed_paths.iter().enumerate() {
        let start = egui::pos2(rect.min.x + segment.start.x, rect.min.y + segment.start.y);
        let end = egui::pos2(rect.min.x + segment.end.x, rect.min.y + segment.end.y);
        
        // Determine if this segment's line is selected
        let is_selected = app.gcode.selected_line == Some(segment.line_number);
        
        let (color, width) = if is_selected {
            // Highlight selected line
            (egui::Color32::from_rgb(255, 128, 0), 4.0)
        } else {
            match segment.move_type {
                MoveType::Rapid => (egui::Color32::BLUE, 2.0),
                MoveType::Feed => (egui::Color32::GREEN, 2.0),
                MoveType::Arc => (egui::Color32::YELLOW, 2.0),
            }
        };
        
        painter.line_segment([start, end], egui::Stroke::new(width, color));
        
        // Draw start point for selected segment
        if is_selected {
            painter.circle_filled(start, 4.0, egui::Color32::RED);
        }
    }
    
    // Show tooltip for selected segment (outside the loop to avoid borrow issues)
    if let Some(selected_line) = app.gcode.selected_line {
        for segment in &app.gcode_editor.parsed_paths {
            if segment.line_number == selected_line {
                let tooltip_text = format!(
                    "Line {}: {:?} move\nFrom ({:.2}, {:.2}, {:.2})\nTo ({:.2}, {:.2}, {:.2})",
                    segment.line_number + 1,
                    segment.move_type,
                    segment.start.x, segment.start.y, segment.start.z,
                    segment.end.x, segment.end.y, segment.end.z
                );
                response.on_hover_text(tooltip_text);
                break;
            }
        }
    }

    ui.label(format!("Segments: {}", app.gcode_editor.parsed_paths.len()));
    
    if let Some(selected) = app.gcode.selected_line {
        ui.colored_label(
            egui::Color32::from_rgb(255, 128, 0),
            format!("Selected: Line {}", selected + 1),
        );
    }
    
    if let Some(sending_line) = app.gcode_editor.sending_from_line {
        ui.colored_label(
            egui::Color32::GREEN,
            format!("Sending from line {}", sending_line + 1),
        );
    }
    
    ui.separator();
    ui.label("💡 Tip: Click on toolpath to select line in editor");
}

/// Calculate distance from point to line segment
fn distance_point_to_segment(point: egui::Pos2, seg_start: egui::Pos2, seg_end: egui::Pos2) -> f32 {
    let dx = seg_end.x - seg_start.x;
    let dy = seg_end.y - seg_start.y;
    let length_squared = dx * dx + dy * dy;
    
    if length_squared == 0.0 {
        // Segment is a point
        return ((point.x - seg_start.x).powi(2) + (point.y - seg_start.y).powi(2)).sqrt();
    }
    
    // Project point onto line segment
    let t = ((point.x - seg_start.x) * dx + (point.y - seg_start.y) * dy) / length_squared;
    let t = t.clamp(0.0, 1.0);
    
    let proj_x = seg_start.x + t * dx;
    let proj_y = seg_start.y + t * dy;
    
    ((point.x - proj_x).powi(2) + (point.y - proj_y).powi(2)).sqrt()
}
