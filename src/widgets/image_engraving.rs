use crate::GcodeKitApp;
use eframe::egui;

pub fn show_image_engraving_widget(ui: &mut egui::Ui, app: &mut GcodeKitApp) {
    ui.group(|ui| {
        ui.label("Image Engraving");
        ui.horizontal(|ui| {
            ui.label("Resolution:");
            ui.add(egui::DragValue::new(&mut app.image_resolution).suffix("dpi"));
        });
        ui.horizontal(|ui| {
            ui.label("Max Power:");
            ui.add(egui::DragValue::new(&mut app.image_max_power).suffix("%"));
        });

        if ui.button("Load Image").clicked() {
            app.load_image_for_engraving();
        }
        if ui.button("Generate Engraving").clicked() {
            app.generate_image_engraving();
        }
    });
}
