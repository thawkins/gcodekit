use crate::GcodeKitApp;
use eframe::egui;

pub fn show_vector_import_widget(ui: &mut egui::Ui, app: &mut GcodeKitApp) {
    ui.group(|ui| {
        ui.label("Vector Import");
        if ui.button("Import SVG/DXF").clicked() {
            app.import_vector_file();
        }
    });
}
