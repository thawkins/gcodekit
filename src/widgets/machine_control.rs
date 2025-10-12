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
    ui.label("Probing");
    ui.horizontal(|ui| {
        if ui.button("Probe Z").clicked() {
            app.send_gcode("G38.2 Z-10 F50 ; Probe Z down");
        }
        if ui.button("Probe X+").clicked() {
            app.send_gcode("G38.2 X100 F100 ; Probe X positive");
        }
        if ui.button("Probe Y+").clicked() {
            app.send_gcode("G38.2 Y100 F100 ; Probe Y positive");
        }
    });
    if ui.button("Set Work Offset (G10 L20)").clicked() {
        app.send_gcode("G10 L20 P1 X0 Y0 Z0 ; Set work offset to current position");
    }
}
