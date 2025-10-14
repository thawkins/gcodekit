use crate::GcodeKitApp;
use eframe::egui;

pub fn show_toolpath_generation_widget(ui: &mut egui::Ui, app: &mut GcodeKitApp) {
    ui.group(|ui| {
        ui.label("Toolpath Generation");

        // Material selection
        ui.label("Material:");
        let mut material_names: Vec<String> = app
            .material_database
            .get_all_materials()
            .iter()
            .map(|m| m.name.clone())
            .collect();
        material_names.insert(0, "None".to_string());

        let current_selection = app
            .ui
            .selected_material
            .as_ref()
            .unwrap_or(&"None".to_string())
            .clone();

        egui::ComboBox::from_id_salt("toolpath_material_combobox")
            .selected_text(&current_selection)
            .width(ui.available_width())
            .show_ui(ui, |ui| {
                for material_name in &material_names {
                    let is_selected = Some(material_name.clone()) == app.ui.selected_material
                        || (material_name == "None" && app.ui.selected_material.is_none());
                    if ui.selectable_label(is_selected, material_name).clicked() {
                        if material_name == "None" {
                            app.ui.selected_material = None;
                        } else {
                            app.ui.selected_material = Some(material_name.clone());
                        }
                    }
                }
            });

        ui.horizontal(|ui| {
            ui.label("Feed Rate:");
            ui.add(egui::DragValue::new(&mut app.cam.tool_feed_rate).suffix("mm/min"));
        });
        ui.horizontal(|ui| {
            ui.label("Spindle:");
            ui.add(egui::DragValue::new(&mut app.cam.tool_spindle_speed).suffix("RPM"));
        });

        if ui.button("Generate Toolpath").clicked() {
            app.generate_toolpath();
            app.create_job_from_generated_gcode("Toolpath", crate::jobs::JobType::CAMOperation);
        }
    });
}
