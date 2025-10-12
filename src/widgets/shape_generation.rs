use eframe::egui;
use crate::GcodeKitApp;

pub fn show_shape_generation_widget(ui: &mut egui::Ui, app: &mut GcodeKitApp) {
    ui.group(|ui| {
        ui.label("Shape Generation");
        ui.horizontal(|ui| {
            ui.label("Width:");
            ui.add(egui::DragValue::new(&mut app.shape_width).suffix("mm"));
        });
        ui.horizontal(|ui| {
            ui.label("Height:");
            ui.add(egui::DragValue::new(&mut app.shape_height).suffix("mm"));
        });
        ui.horizontal(|ui| {
            ui.label("Radius:");
            ui.add(egui::DragValue::new(&mut app.shape_radius).suffix("mm"));
        });

        ui.horizontal(|ui| {
            if ui.button("Rectangle").clicked() {
                app.generate_rectangle();
            }
            if ui.button("Circle").clicked() {
                app.generate_circle();
            }
        });
    });
}