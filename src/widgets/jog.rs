use crate::GcodeKitApp;
use eframe::egui;

pub fn show_jog_widget(ui: &mut egui::Ui, app: &mut GcodeKitApp) {
    ui.group(|ui| {
        ui.label("Jog Control");

        // Step size selection
        ui.horizontal(|ui| {
            ui.label("Step:");
            ui.selectable_value(&mut app.machine.jog_step_size, 0.1, "0.1mm");
            ui.selectable_value(&mut app.machine.jog_step_size, 1.0, "1mm");
            ui.selectable_value(&mut app.machine.jog_step_size, 10.0, "10mm");
            ui.selectable_value(&mut app.machine.jog_step_size, 50.0, "50mm");
        });

        ui.separator();

        // Z axis (up/down)
        ui.horizontal(|ui| {
            ui.label("Z");
            if ui.button("⬆##z_up").clicked() {
                app.machine.communication.jog_axis('Z', app.machine.jog_step_size);
            }
            if ui.button("⬇##z_down").clicked() {
                app.machine.communication.jog_axis('Z', -app.machine.jog_step_size);
            }
        });

        // Y axis (forward/back)
        ui.horizontal(|ui| {
            ui.label("Y");
            if ui.button("⬅##y_left").clicked() {
                app.machine.communication.jog_axis('Y', -app.machine.jog_step_size);
            }
            if ui.button("⮕##y_right").clicked() {
                app.machine.communication.jog_axis('Y', app.machine.jog_step_size);
            }
        });

        // X axis (left/right)
        ui.horizontal(|ui| {
            ui.label("X");
            if ui.button("⬅##x_left").clicked() {
                app.machine.communication.jog_axis('X', -app.machine.jog_step_size);
            }
            if ui.button("⮕##x_right").clicked() {
                app.machine.communication.jog_axis('X', app.machine.jog_step_size);
            }
        });

        ui.separator();

        // Additional axes (A, B, C, D) - shown conditionally if supported
        if app.machine.current_position.a.is_some()
            || app.machine.current_position.b.is_some()
            || app.machine.current_position.c.is_some()
            || app.machine.current_position.d.is_some()
        {
            ui.label("Rotary Axes");

            // A axis
            if app.machine.current_position.a.is_some() {
                ui.horizontal(|ui| {
                    ui.label("A");
                    if ui.button("⟲##a_left").clicked() {
                        app.machine.communication.jog_axis('A', -app.machine.jog_step_size);
                    }
                    if ui.button("⟳##a_right").clicked() {
                        app.machine.communication.jog_axis('A', app.machine.jog_step_size);
                    }
                });
            }

            // B axis
            if app.machine.current_position.b.is_some() {
                ui.horizontal(|ui| {
                    ui.label("B");
                    if ui.button("⟲##b_left").clicked() {
                        app.machine.communication.jog_axis('B', -app.machine.jog_step_size);
                    }
                    if ui.button("⟳##b_right").clicked() {
                        app.machine.communication.jog_axis('B', app.machine.jog_step_size);
                    }
                });
            }

            // C axis
            if app.machine.current_position.c.is_some() {
                ui.horizontal(|ui| {
                    ui.label("C");
                    if ui.button("⟲##c_left").clicked() {
                        app.machine.communication.jog_axis('C', -app.machine.jog_step_size);
                    }
                    if ui.button("⟳##c_right").clicked() {
                        app.machine.communication.jog_axis('C', app.machine.jog_step_size);
                    }
                });
            }

            // D axis
            if app.machine.current_position.d.is_some() {
                ui.horizontal(|ui| {
                    ui.label("D");
                    if ui.button("⟲##d_left").clicked() {
                        app.machine.communication.jog_axis('D', -app.machine.jog_step_size);
                    }
                    if ui.button("⟳##d_right").clicked() {
                        app.machine.communication.jog_axis('D', app.machine.jog_step_size);
                    }
                });
            }
        }

        // Home button
        ui.horizontal(|ui| {
            if ui.button("🏠 Home All").clicked() {
                app.machine.communication.home_all_axes();
            }
        });

        ui.separator();

        // Emergency stop
        ui.colored_label(egui::Color32::RED, "⚠ Emergency Stop");
        if ui
            .add(egui::Button::new("🚨 STOP").fill(egui::Color32::RED))
            .clicked()
        {
            app.machine.communication.emergency_stop();
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_show_jog_widget_compiles() {
        // This test ensures the function compiles and has the expected signature
        // Full UI testing would require egui context mocking
        let _fn_exists = show_jog_widget as fn(&mut egui::Ui, &mut GcodeKitApp);
    }
}
