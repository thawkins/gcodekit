use eframe::egui;

use crate::GcodeKitApp;

pub fn show_tool_management_widget(ui: &mut egui::Ui, app: &mut GcodeKitApp) {
    ui.label("Tool Management");

    ui.horizontal(|ui| {
        ui.label("Current Tool:");
        ui.add(egui::DragValue::new(&mut app.current_tool).range(0..=99));
    });

    if ui.button("Change Tool (M6)").clicked() {
        let cmd = format!("M6 T{}", app.current_tool);
        app.send_gcode(&cmd);
    }

    ui.separator();
    ui.label("Tool Length Offsets");

    if ui.button("Apply Tool Length Offset (G43)").clicked() {
        let cmd = format!("G43 H{} ; Apply tool length offset for tool {}", app.current_tool, app.current_tool);
        app.send_gcode(&cmd);
    }

    if ui.button("Cancel Tool Length Offset (G49)").clicked() {
        app.send_gcode("G49 ; Cancel tool length offset");
    }

    if ui.button("Probe Tool Length (G43.1)").clicked() {
        let cmd = format!("G43.1 Z0 ; Probe tool length for tool {}", app.current_tool);
        app.send_gcode(&cmd);
    }

    ui.separator();
    ui.label("Tool Library");

    ui.horizontal(|ui| {
        if ui.button("Add Tool").clicked() {
            app.tool_library.push(crate::designer::Tool {
                name: format!("Tool {}", app.tool_library.len() + 1),
                diameter: 3.0,
                material: "HSS".to_string(),
                flute_count: 2,
                max_rpm: 10000,
            });
        }
    });

    egui::ScrollArea::vertical().show(ui, |ui| {
        for (i, tool) in app.tool_library.iter_mut().enumerate() {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label(format!("T{}:", i + 1));
                    ui.text_edit_singleline(&mut tool.name);
                });
                ui.horizontal(|ui| {
                    ui.label("Diameter:");
                    ui.add(egui::DragValue::new(&mut tool.diameter).range(0.1..=50.0));
                    ui.label("mm");
                });
                ui.horizontal(|ui| {
                    ui.label("Material:");
                    ui.text_edit_singleline(&mut tool.material);
                });
                ui.horizontal(|ui| {
                    ui.label("Flutes:");
                    ui.add(egui::DragValue::new(&mut tool.flute_count).range(1..=8));
                });
                ui.horizontal(|ui| {
                    ui.label("Max RPM:");
                    ui.add(egui::DragValue::new(&mut tool.max_rpm).range(1000..=30000));
                });
                ui.horizontal(|ui| {
                    if ui.button("Select").clicked() {
                        app.current_tool = i as i32 + 1;
                    }
                    if ui.button("Remove").clicked() {
                        // Mark for removal by setting diameter to negative
                        tool.diameter = -1.0;
                    }
                });
            });
        }
    });

    // Clean up removed tools
    app.tool_library.retain(|tool| tool.diameter >= 0.0);
}
