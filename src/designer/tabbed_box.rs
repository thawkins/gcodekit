use crate::GcodeKitApp;
use eframe::egui;

pub fn show_tabbed_box_widget(ui: &mut egui::Ui, app: &mut GcodeKitApp) {
    ui.group(|ui| {
        ui.label("Tabbed Box");
        ui.horizontal(|ui| {
            ui.label("Length:");
            ui.add(egui::DragValue::new(&mut app.cam.box_length).suffix("mm"));
        });
        ui.horizontal(|ui| {
            ui.label("Width:");
            ui.add(egui::DragValue::new(&mut app.cam.box_width).suffix("mm"));
        });
        ui.horizontal(|ui| {
            ui.label("Height:");
            ui.add(egui::DragValue::new(&mut app.cam.box_height).suffix("mm"));
        });
        ui.horizontal(|ui| {
            ui.label("Tab Size:");
            ui.add(egui::DragValue::new(&mut app.cam.tab_size).suffix("mm"));
        });

        if ui.button("Generate Box").clicked() {
            app.generate_tabbed_box();
        }
    });
}
