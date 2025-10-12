use crate::GcodeKitApp;
use eframe::egui;

pub fn show_jog_widget(ui: &mut egui::Ui, app: &mut GcodeKitApp) {
    ui.group(|ui| {
        ui.label("Jog Control");

        // Step size selection
        ui.horizontal(|ui| {
            ui.label("Step:");
            ui.selectable_value(&mut app.jog_step_size, 0.1, "0.1mm");
            ui.selectable_value(&mut app.jog_step_size, 1.0, "1mm");
            ui.selectable_value(&mut app.jog_step_size, 10.0, "10mm");
            ui.selectable_value(&mut app.jog_step_size, 50.0, "50mm");
        });

        ui.separator();

        // Z axis (up/down)
        ui.horizontal(|ui| {
            ui.label("Z");
            if ui.button("⬆").clicked() {
                app.jog_axis('Z', app.jog_step_size);
            }
            if ui.button("⬇").clicked() {
                app.jog_axis('Z', -app.jog_step_size);
            }
        });

        // Y axis (forward/back)
        ui.horizontal(|ui| {
            ui.label("Y");
            if ui.button("⬅").clicked() {
                app.jog_axis('Y', -app.jog_step_size);
            }
            if ui.button("⮕").clicked() {
                app.jog_axis('Y', app.jog_step_size);
            }
        });

        // X axis (left/right)
        ui.horizontal(|ui| {
            ui.label("X");
            if ui.button("⬅").clicked() {
                app.jog_axis('X', -app.jog_step_size);
            }
            if ui.button("⮕").clicked() {
                app.jog_axis('X', app.jog_step_size);
            }
        });

        // Home button
        ui.horizontal(|ui| {
            if ui.button("🏠 Home All").clicked() {
                app.home_all_axes();
            }
        });
    });
}
