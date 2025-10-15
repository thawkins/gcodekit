use eframe::egui;

use crate::GcodeKitApp;

pub fn show_tool_management_widget(ui: &mut egui::Ui, app: &mut GcodeKitApp) {
    ui.label("Tool Management");

    ui.horizontal(|ui| {
        ui.label("Current Tool:");
        ui.add(egui::DragValue::new(&mut app.cam.current_tool).range(0..=99));
    });

    ui.horizontal(|ui| {
        ui.label("Current Tool:");
        ui.add(egui::DragValue::new(&mut app.cam.current_tool).range(0..=99));
        if ui.button("Change Tool (M6)").clicked() {
            let cmd = format!("M6 T{}", app.cam.current_tool);
            app.send_gcode(&cmd);
        }
        if ui.button("Load Tool").clicked() {
            if let Some(tool) = app
                .cam
                .tool_library
                .iter()
                .find(|t| t.tool_number == app.cam.current_tool as u32)
            {
                let cmd = format!("T{} M6 ; Load tool {}", tool.tool_number, tool.name);
                app.send_gcode(&cmd);
            } else {
                app.machine.status_message =
                    format!("Tool {} not found in library", app.cam.current_tool);
            }
        }
    });

    ui.separator();
    ui.label("Tool Length Offsets");

    ui.horizontal(|ui| {
        if ui.button("Apply Tool Length Offset (G43)").clicked() {
            if let Some(tool) = app
                .cam
                .tool_library
                .iter()
                .find(|t| t.tool_number == app.cam.current_tool as u32)
            {
                let cmd = format!(
                    "G43 H{} ; Apply tool length offset for tool {} ({})",
                    tool.tool_number, tool.tool_number, tool.name
                );
                app.send_gcode(&cmd);
            } else {
                app.send_gcode(&format!(
                    "G43 H{} ; Apply tool length offset",
                    app.cam.current_tool
                ));
            }
        }

        if ui.button("Cancel Tool Length Offset (G49)").clicked() {
            app.send_gcode("G49 ; Cancel tool length offset");
        }
    });

    ui.horizontal(|ui| {
        if ui.button("Probe Tool Length (G43.1)").clicked() {
            let cmd = format!(
                "G43.1 Z0 ; Probe and set tool length offset for tool {}",
                app.cam.current_tool
            );
            app.send_gcode(&cmd);
        }

        if ui.button("Set Tool Offset (G10 L1)").clicked()
            && let Some(tool) = app
                .cam
                .tool_library
                .iter()
                .find(|t| t.tool_number == app.cam.current_tool as u32)
            {
                let cmd = format!(
                    "G10 L1 P{} Z{} ; Set tool {} length offset to current position",
                    tool.tool_number, tool.length_offset, tool.name
                );
                app.send_gcode(&cmd);
            }
    });

    ui.separator();
    ui.label("Tool Library");

    ui.horizontal(|ui| {
        if ui.button("Add Tool").clicked() {
            app.cam.tool_library.push(crate::designer::Tool {
                name: format!("Tool {}", app.cam.tool_library.len() + 1),
                diameter: 3.0,
                length: 40.0,
                material: "HSS".to_string(),
                flute_count: 2,
                max_rpm: 10000,
                tool_number: (app.cam.tool_library.len() + 1) as u32,
                length_offset: 0.0,
                wear_offset: 0.0,
            });
        }
    });

    // Display tool library (read-only for now)
    egui::ScrollArea::vertical().show(ui, |ui| {
        for tool in &app.cam.tool_library {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label(format!("T{}:", tool.tool_number));
                    ui.label(&tool.name);
                });
                ui.horizontal(|ui| {
                    ui.label(format!("Diameter: {:.1}mm", tool.diameter));
                    ui.label(format!("Length: {:.1}mm", tool.length));
                    ui.label(format!("Material: {}", tool.material));
                });
                ui.horizontal(|ui| {
                    ui.label(format!("Flutes: {}", tool.flute_count));
                    ui.label(format!("Max RPM: {}", tool.max_rpm));
                });
                ui.horizontal(|ui| {
                    ui.label(format!("Length Offset: {:.3}mm", tool.length_offset));
                    ui.label(format!("Wear Offset: {:.3}mm", tool.wear_offset));
                });
            });
        }
    });

    ui.label("Note: Tool editing interface will be enhanced in future updates");

    // Clean up removed tools
    app.cam.tool_library.retain(|tool| tool.diameter >= 0.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_show_tool_management_widget_compiles() {
        // This test ensures the function compiles and has the expected signature
        // Full UI testing would require egui context mocking
        let _fn_exists = show_tool_management_widget as fn(&mut egui::Ui, &mut GcodeKitApp);
    }
}
