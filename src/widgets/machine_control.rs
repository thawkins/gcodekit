use eframe::egui;

use crate::GcodeKitApp;

pub fn show_machine_control_widget(ui: &mut egui::Ui, app: &mut GcodeKitApp) {
    ui.label("Machine Control");
    if ui.button("Home All (G28)").clicked() {
        app.send_gcode("G28");
    }
    ui.horizontal(|ui| {
        if ui.button("Home X").clicked() {
            app.send_gcode("$HX");
        }
        if ui.button("Home Y").clicked() {
            app.send_gcode("$HY");
        }
        if ui.button("Home Z").clicked() {
            app.send_gcode("$HZ");
        }
    });
    ui.separator();
    ui.label("Probing Routines");

    ui.collapsing("Basic Probing", |ui| {
        ui.horizontal(|ui| {
            if ui.button("Probe Z Down").clicked() {
                app.send_gcode("G38.2 Z-50 F100 ; Probe Z down to find surface");
            }
            if ui.button("Probe Z Up").clicked() {
                app.send_gcode("G38.2 Z50 F100 ; Probe Z up");
            }
        });

        ui.horizontal(|ui| {
            if ui.button("Probe X+").clicked() {
                app.send_gcode("G38.2 X200 F200 ; Probe X positive direction");
            }
            if ui.button("Probe X-").clicked() {
                app.send_gcode("G38.2 X-200 F200 ; Probe X negative direction");
            }
        });

        ui.horizontal(|ui| {
            if ui.button("Probe Y+").clicked() {
                app.send_gcode("G38.2 Y200 F200 ; Probe Y positive direction");
            }
            if ui.button("Probe Y-").clicked() {
                app.send_gcode("G38.2 Y-200 F200 ; Probe Y negative direction");
            }
        });
    });

    ui.collapsing("Auto-Leveling", |ui| {
        ui.label("Probe a grid of points for surface leveling");
        ui.horizontal(|ui| {
            ui.label("Grid Size:");
            let mut grid_size = 3;
            ui.add(egui::DragValue::new(&mut grid_size).range(2..=10));
            ui.label("x");
            ui.add(egui::DragValue::new(&mut grid_size).range(2..=10));
        });

        ui.horizontal(|ui| {
            ui.label("Spacing:");
            let mut spacing = 10.0;
            ui.add(egui::DragValue::new(&mut spacing).range(1.0..=100.0));
            ui.label("mm");
        });

        if ui.button("Start Auto-Leveling").clicked() {
            // This would implement a probing routine for surface leveling
            app.send_gcode("G38.2 Z-20 F50 ; Initial probe to find surface");
            app.status_message = "Auto-leveling not fully implemented yet".to_string();
        }
    });
    ui.separator();
    ui.label("Work Coordinate Systems");

    ui.collapsing("Work Offsets (G54-G59)", |ui| {
        ui.horizontal(|ui| {
            if ui.button("G54").clicked() {
                app.send_gcode("G54 ; Select work coordinate system 1");
            }
            if ui.button("G55").clicked() {
                app.send_gcode("G55 ; Select work coordinate system 2");
            }
            if ui.button("G56").clicked() {
                app.send_gcode("G56 ; Select work coordinate system 3");
            }
        });

        ui.horizontal(|ui| {
            if ui.button("G57").clicked() {
                app.send_gcode("G57 ; Select work coordinate system 4");
            }
            if ui.button("G58").clicked() {
                app.send_gcode("G58 ; Select work coordinate system 5");
            }
            if ui.button("G59").clicked() {
                app.send_gcode("G59 ; Select work coordinate system 6");
            }
        });

        ui.horizontal(|ui| {
            if ui.button("Set Current WCS (G10 L20)").clicked() {
                app.send_gcode("G10 L20 P1 X0 Y0 Z0 ; Set WCS origin to current position");
            }
            if ui.button("Set WCS from Probe").clicked() {
                app.send_gcode("G38.2 Z-10 F50 ; Probe surface");
                app.send_gcode("G10 L20 P1 Z0 ; Set Z zero at probed position");
            }
        });
    });

    ui.separator();
    ui.label("Safety & Limits");

    ui.horizontal(|ui| {
        ui.checkbox(&mut app.soft_limits_enabled, "Enable Soft Limits");
        if ui.button("Update Settings").clicked() {
            let value = if app.soft_limits_enabled { 1 } else { 0 };
            app.send_gcode(&format!("$20={}", value));
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_show_machine_control_widget_compiles() {
        // This test ensures the function compiles and has the expected signature
        // Full UI testing would require egui context mocking
        let _fn_exists = show_machine_control_widget as fn(&mut egui::Ui, &mut GcodeKitApp);
    }
}
