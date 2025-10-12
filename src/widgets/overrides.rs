use eframe::egui;
use crate::{GcodeKitApp, MachineMode};

pub fn show_overrides_widget(ui: &mut egui::Ui, app: &mut GcodeKitApp) {
    ui.group(|ui| {
        ui.label("Overrides");

        // Machine mode selection
        ui.horizontal(|ui| {
            ui.label("Mode:");
            ui.selectable_value(&mut app.machine_mode, MachineMode::CNC, "CNC");
            ui.selectable_value(&mut app.machine_mode, MachineMode::Laser, "Laser");
        });

        ui.separator();

        // Spindle/Laser control
        let spindle_label = match app.machine_mode {
            MachineMode::CNC => "Spindle Speed:",
            MachineMode::Laser => "Laser Power:",
        };
        let spindle_suffix = match app.machine_mode {
            MachineMode::CNC => "% RPM",
            MachineMode::Laser => "% Power",
        };

        ui.horizontal(|ui| {
            ui.label(spindle_label);
            if ui.add(egui::DragValue::new(&mut app.spindle_override)
                .suffix(spindle_suffix)
                .range(0.0..=200.0)
                .speed(1.0)).changed() {
                app.send_spindle_override();
            }
        });

        // Feed rate control
        ui.horizontal(|ui| {
            ui.label("Feed Rate:");
            if ui.add(egui::DragValue::new(&mut app.feed_override)
                .suffix("%")
                .range(0.0..=200.0)
                .speed(1.0)).changed() {
                app.send_feed_override();
            }
        });

        // Reset button
        ui.horizontal(|ui| {
            if ui.button("Reset to 100%").clicked() {
                app.spindle_override = 100.0;
                app.feed_override = 100.0;
                app.send_spindle_override();
                app.send_feed_override();
            }
        });
    });
}