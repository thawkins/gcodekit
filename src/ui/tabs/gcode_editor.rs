use eframe::egui;

use crate::GcodeKitApp;

/// Shows the G-code editor tab with all enhanced features
pub fn show_gcode_editor_tab(app: &mut GcodeKitApp, ui: &mut egui::Ui) {
    // Sync content between old GcodeState and new GcodeEditorState on first load
    if !app.gcode.gcode_content.is_empty() && app.gcode_editor.buffer.get_content().is_empty() {
        app.gcode_editor.buffer.set_content(&app.gcode.gcode_content);
        app.gcode_editor.gcode_content = app.gcode.gcode_content.clone();
        app.gcode_editor.gcode_filename = app.gcode.gcode_filename.clone();
        app.gcode_editor.on_buffer_change();
        
        // Reset to top of file
        app.gcode_editor.selected_line = Some(0);
        app.gcode_editor.virtualized_state = Default::default();
    }

    // Show sending progress if active
    if app.gcode_editor.sending_progress > 0.0 && app.gcode_editor.sending_progress < 1.0 {
        ui.horizontal(|ui| {
            ui.label("Sending G-code:");
            let progress_bar = egui::ProgressBar::new(app.gcode_editor.sending_progress)
                .show_percentage()
                .animate(true);
            ui.add(progress_bar);
        });
        ui.separator();
    }

    // Use the enhanced editor with all features
    app.gcode_editor.show_ui(ui, &app.gcode_editor.parsed_paths.clone());

    // Sync any changes back to the old GcodeState for compatibility with other parts of the app
    let current_content = app.gcode_editor.buffer.get_content();
    if current_content != app.gcode.gcode_content {
        app.gcode.gcode_content = current_content;
        // Re-parse to update visualizer
        app.parse_gcode();
    }

    // Sync selected line for visualizer integration
    if app.gcode_editor.selected_line != app.gcode.selected_line {
        app.gcode.selected_line = app.gcode_editor.selected_line;
    }
}
