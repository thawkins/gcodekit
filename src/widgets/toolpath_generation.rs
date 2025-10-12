use eframe::egui;
use crate::GcodeKitApp;

pub fn show_toolpath_generation_widget(ui: &mut egui::Ui, app: &mut GcodeKitApp) {
    ui.group(|ui| {
        ui.label("Toolpath Generation");
        ui.horizontal(|ui| {
            ui.label("Feed Rate:");
            ui.add(egui::DragValue::new(&mut app.tool_feed_rate).suffix("mm/min"));
        });
        ui.horizontal(|ui| {
            ui.label("Spindle:");
            ui.add(egui::DragValue::new(&mut app.tool_spindle_speed).suffix("RPM"));
        });

        if ui.button("Generate Toolpath").clicked() {
            app.generate_toolpath();
        }
    });
}