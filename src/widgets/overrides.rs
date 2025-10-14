use crate::{GcodeKitApp, MachineMode};
use eframe::egui;

pub fn show_overrides_widget(ui: &mut egui::Ui, app: &mut GcodeKitApp) {
    ui.group(|ui| {
        ui.label("Overrides");

        // Machine mode selection
        ui.horizontal(|ui| {
            ui.label("Mode:");
            ui.selectable_value(&mut app.machine.machine_mode, MachineMode::CNC, "CNC");
            ui.selectable_value(&mut app.machine.machine_mode, MachineMode::Laser, "Laser");
        });

        ui.separator();

        // Spindle/Laser control
        let spindle_label = match app.machine.machine_mode {
            MachineMode::CNC => "Spindle Speed:",
            MachineMode::Laser => "Laser Power:",
        };
        let spindle_suffix = match app.machine.machine_mode {
            MachineMode::CNC => "% RPM",
            MachineMode::Laser => "% Power",
        };

        ui.horizontal(|ui| {
            ui.label(spindle_label);
            if ui
                .add(
                    egui::DragValue::new(&mut app.machine.spindle_override)
                        .suffix(spindle_suffix)
                        .range(0.0..=200.0)
                        .speed(1.0),
                )
                .changed()
            {
                app.machine.communication.send_spindle_override(app.machine.spindle_override);
            }
        });

        // Feed rate control
        ui.horizontal(|ui| {
            ui.label("Feed Rate:");
            if ui
                .add(
                    egui::DragValue::new(&mut app.machine.feed_override)
                        .suffix("%")
                        .range(0.0..=200.0)
                        .speed(1.0),
                )
                .changed()
            {
                app.machine.communication.send_feed_override(app.machine.feed_override);
            }
        });

        // Reset button
        ui.horizontal(|ui| {
            if ui.button("Reset to 100%").clicked() {
                app.machine.spindle_override = 100.0;
                app.machine.feed_override = 100.0;
                app.machine.communication.send_spindle_override(app.machine.spindle_override);
                app.machine.communication.send_feed_override(app.machine.feed_override);
            }
        });

        ui.separator();

        // Soft limits
        ui.horizontal(|ui| {
            ui.label("Soft Limits:");
            if ui
                .checkbox(&mut app.machine.soft_limits_enabled, "")
                .changed()
            {
                let value = if app.machine.soft_limits_enabled {
                    1
                } else {
                    0
                };
                app.send_gcode(&format!("$20={}", value));
            }
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_show_overrides_widget_compiles() {
        // This test ensures the function compiles and has the expected signature
        // Full UI testing would require egui context mocking
        let _fn_exists = show_overrides_widget as fn(&mut egui::Ui, &mut GcodeKitApp);
    }
}
