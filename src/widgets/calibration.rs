use crate::GcodeKitApp;
use eframe::egui;

pub fn show_calibration_widget(ui: &mut egui::Ui, app: &mut GcodeKitApp) {
    ui.group(|ui| {
        ui.label("Calibration");

        ui.collapsing("Step Calibration (steps/mm)", |ui| {
            ui.horizontal(|ui| {
                ui.label("X:");
                let mut x_steps = 0.0;
                if ui
                    .add(egui::DragValue::new(&mut x_steps).suffix(" steps/mm"))
                    .changed()
                {
                    app.send_gcode(&format!("$100={:.3}", x_steps));
                }
            });
            ui.horizontal(|ui| {
                ui.label("Y:");
                let mut y_steps = 0.0;
                if ui
                    .add(egui::DragValue::new(&mut y_steps).suffix(" steps/mm"))
                    .changed()
                {
                    app.send_gcode(&format!("$101={:.3}", y_steps));
                }
            });
            ui.horizontal(|ui| {
                ui.label("Z:");
                let mut z_steps = 0.0;
                if ui
                    .add(egui::DragValue::new(&mut z_steps).suffix(" steps/mm"))
                    .changed()
                {
                    app.send_gcode(&format!("$102={:.3}", z_steps));
                }
            });
        });

        ui.collapsing("Backlash Compensation (mm)", |ui| {
            ui.horizontal(|ui| {
                ui.label("X:");
                let mut x_backlash = 0.0;
                if ui
                    .add(egui::DragValue::new(&mut x_backlash).suffix(" mm"))
                    .changed()
                {
                    app.send_gcode(&format!("$130={:.3}", x_backlash));
                }
            });
            ui.horizontal(|ui| {
                ui.label("Y:");
                let mut y_backlash = 0.0;
                if ui
                    .add(egui::DragValue::new(&mut y_backlash).suffix(" mm"))
                    .changed()
                {
                    app.send_gcode(&format!("$131={:.3}", y_backlash));
                }
            });
            ui.horizontal(|ui| {
                ui.label("Z:");
                let mut z_backlash = 0.0;
                if ui
                    .add(egui::DragValue::new(&mut z_backlash).suffix(" mm"))
                    .changed()
                {
                    app.send_gcode(&format!("$132={:.3}", z_backlash));
                }
            });
        });

        ui.collapsing("Homing Configuration", |ui| {
            ui.horizontal(|ui| {
                ui.label("Homing Cycle Enable:");
                let mut homing_enable = false;
                if ui.checkbox(&mut homing_enable, "").changed() {
                    app.send_gcode(&format!("$22={}", if homing_enable { 1 } else { 0 }));
                }
            });
            ui.horizontal(|ui| {
                ui.label("Homing Dir Invert:");
                let mut homing_dir = 0;
                if ui
                    .add(egui::DragValue::new(&mut homing_dir).range(0..=255))
                    .changed()
                {
                    app.send_gcode(&format!("$23={}", homing_dir));
                }
            });
            ui.horizontal(|ui| {
                ui.label("Homing Feed (mm/min):");
                let mut homing_feed = 0.0;
                if ui
                    .add(egui::DragValue::new(&mut homing_feed).suffix(" mm/min"))
                    .changed()
                {
                    app.send_gcode(&format!("$24={:.3}", homing_feed));
                }
            });
            ui.horizontal(|ui| {
                ui.label("Homing Seek (mm/min):");
                let mut homing_seek = 0.0;
                if ui
                    .add(egui::DragValue::new(&mut homing_seek).suffix(" mm/min"))
                    .changed()
                {
                    app.send_gcode(&format!("$25={:.3}", homing_seek));
                }
            });
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_show_calibration_widget_compiles() {
        // This test ensures the function compiles and has the expected signature
        // Full UI testing would require egui context mocking
        let _fn_exists = show_calibration_widget as fn(&mut egui::Ui, &mut GcodeKitApp);
    }
}
