use eframe::egui;

use crate::GcodeKitApp;
use crate::types::MoveType;

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

    // Header row
    ui.horizontal(|ui| {
        ui.label("3D Visualizer");
        ui.separator();
        if ui.button("🔄 Refresh View").clicked() {}
        if ui.button("📏 Fit to View").clicked() {}
        ui.separator();
        let run_button_enabled = app.gcode.selected_line.is_some()
            && *app.machine.communication.get_connection_state()
                == crate::communication::ConnectionState::Connected;
        if ui
            .add_enabled(run_button_enabled, egui::Button::new("▶️ Run from Selected Line"))
            .clicked()
        {
            if let Some(line_number) = app.gcode.selected_line {
                app.send_gcode_from_line(line_number);
            }
        }
        ui.separator();
        ui.label("(Ctrl+R to run from selected line)");
    });

    ui.separator();

    // Visualization area
    let available_size = ui.available_size();
    let (rect, _response) = ui.allocate_exact_size(available_size, egui::Sense::click());

    if app.gcode.gcode_content.is_empty() {
        ui.centered_and_justified(|ui| ui.label("Load G-code to visualize toolpath"));
        return;
    }

    let painter = ui.painter();
    // Simple unscaled drawing: draw each segment offset by rect.min
    for segment in &app.gcode.parsed_paths {
        let start = egui::pos2(rect.min.x + segment.start.x, rect.min.y + segment.start.y);
        let end = egui::pos2(rect.min.x + segment.end.x, rect.min.y + segment.end.y);
        let color = match segment.move_type {
            MoveType::Rapid => egui::Color32::BLUE,
            MoveType::Feed => egui::Color32::GREEN,
            MoveType::Arc => egui::Color32::YELLOW,
        };
        painter.line_segment([start, end], egui::Stroke::new(2.0, color));
    }

    ui.label(format!("Segments: {}", app.gcode.parsed_paths.len()));
    if let Some(sending_line) = app.gcode.sending_from_line {
        ui.colored_label(egui::Color32::GREEN, format!("Sending from line {}", sending_line + 1));
    }
}
