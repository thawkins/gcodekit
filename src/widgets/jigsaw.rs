use eframe::egui;
use crate::GcodeKitApp;

pub fn show_jigsaw_widget(ui: &mut egui::Ui, app: &mut GcodeKitApp) {
    ui.group(|ui| {
        ui.label("Jigsaw Puzzle");
        ui.horizontal(|ui| {
            ui.label("Pieces:");
            ui.add(egui::DragValue::new(&mut app.jigsaw_pieces).range(4..=100));
        });
        ui.horizontal(|ui| {
            ui.label("Complexity:");
            ui.add(egui::DragValue::new(&mut app.jigsaw_complexity).range(1..=5));
        });

        if ui.button("Generate Jigsaw").clicked() {
            app.generate_jigsaw();
        }
    });
}