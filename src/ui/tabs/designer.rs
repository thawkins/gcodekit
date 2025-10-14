use egui;

use crate::GcodeKitApp;

/// Renders the designer tab UI showing the design interface.
/// Provides tools for creating and editing designs, with export/import capabilities.
pub fn show_designer_tab(app: &mut GcodeKitApp, ui: &mut egui::Ui) {
    if let Some(event) = app.designer.show_ui(ui) {
        match event {
            crate::designer::DesignerEvent::ExportGcode => {
                app.export_design_to_gcode();
            }
            crate::designer::DesignerEvent::ImportFile => {
                app.import_design_file();
            }
            crate::designer::DesignerEvent::ExportStl => {
                app.machine.status_message = "STL export not yet implemented".to_string();
            }
            crate::designer::DesignerEvent::ExportObj => {
                app.machine.status_message = "OBJ export not yet implemented".to_string();
            }
            crate::designer::DesignerEvent::ExportGltf => {
                app.machine.status_message = "GLTF export not yet implemented".to_string();
            }
        }
    }
}
