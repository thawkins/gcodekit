use eframe::egui;

use crate::GcodeKitApp;

pub fn show_safety_widget(ui: &mut egui::Ui, app: &mut GcodeKitApp) {
    ui.group(|ui| {
        ui.label("🛡️ Machine Safety");

        // Emergency Stop - prominently displayed
        ui.separator();
        ui.colored_label(egui::Color32::RED, "🚨 EMERGENCY STOP");
        ui.horizontal(|ui| {
            let emergency_button = ui.add(
                egui::Button::new("STOP ALL MOTION")
                    .fill(egui::Color32::RED)
                    .stroke(egui::Stroke::new(2.0, egui::Color32::WHITE)),
            );
            if emergency_button.clicked() {
                app.communication.emergency_stop();
                app.status_message = "Emergency stop activated!".to_string();
            }
            ui.label("Immediately halts all machine motion");
        });

        ui.separator();
        ui.label("Safety Settings");

        ui.collapsing("Soft Limits", |ui| {
            ui.checkbox(&mut app.soft_limits_enabled, "Enable soft limits");
            ui.label("Prevents machine from moving beyond defined boundaries");
            if ui.button("Apply Setting").clicked() {
                let value = if app.soft_limits_enabled { 1 } else { 0 };
                app.send_gcode(&format!("$20={}", value));
            }
        });

        ui.collapsing("Homing & Limits", |ui| {
            ui.label("Homing cycle ensures machine knows its position");
            ui.horizontal(|ui| {
                if ui.button("Home All Axes").clicked() {
                    app.send_gcode("G28 ; Home all axes");
                }
                if ui.button("Home Z First").clicked() {
                    app.send_gcode("$HZ ; Home Z axis first");
                }
            });

            ui.horizontal(|ui| {
                if ui.button("Home X").clicked() {
                    app.send_gcode("$HX ; Home X axis");
                }
                if ui.button("Home Y").clicked() {
                    app.send_gcode("$HY ; Home Y axis");
                }
                if ui.button("Home Z").clicked() {
                    app.send_gcode("$HZ ; Home Z axis");
                }
            });
        });

        ui.collapsing("Feed Hold & Resume", |ui| {
            ui.horizontal(|ui| {
                if ui.button("Feed Hold").clicked() {
                    app.send_gcode("! ; Feed hold (pause motion)");
                }
                if ui.button("Resume").clicked() {
                    app.send_gcode("~ ; Resume motion");
                }
            });
            ui.label("Temporarily pause and resume machine motion");
        });

        ui.collapsing("Machine State", |ui| {
            ui.label("Current machine status and controls");
            ui.horizontal(|ui| {
                if ui.button("Check Status").clicked() {
                    app.send_gcode("? ; Request status report");
                }
                if ui.button("Reset").clicked() {
                    app.send_gcode("\x18 ; Software reset");
                }
            });
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_show_safety_widget_compiles() {
        // This test ensures the function compiles and has the expected signature
        // Full UI testing would require egui context mocking
        let _fn_exists = show_safety_widget as fn(&mut egui::Ui, &mut GcodeKitApp);
    }
}
