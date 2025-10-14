use crate::GcodeKitApp;
use eframe::egui;

pub fn show_jigsaw_widget(ui: &mut egui::Ui, app: &mut GcodeKitApp) {
    ui.group(|ui| {
        ui.label("Jigsaw Puzzle");
        ui.horizontal(|ui| {
            ui.label("Pieces:");
            ui.add(egui::DragValue::new(&mut app.cam.jigsaw_pieces).range(4..=100));
        });
        ui.horizontal(|ui| {
            ui.label("Complexity:");
            ui.add(egui::DragValue::new(&mut app.cam.jigsaw_complexity).range(1..=5));
        });

        if ui.button("Generate Jigsaw").clicked() {
            app.generate_jigsaw();
        }
    });
}
