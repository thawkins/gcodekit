use crate::GcodeKitApp;
use eframe::egui;

pub fn show_shape_generation_widget(ui: &mut egui::Ui, app: &mut GcodeKitApp) {
    ui.group(|ui| {
        ui.label("Shape Generation");

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

        egui::ComboBox::from_id_salt("shape_material_combobox")
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
            ui.label("Width:");
            ui.add(egui::DragValue::new(&mut app.cam.shape_width).suffix("mm"));
        });
        ui.horizontal(|ui| {
            ui.label("Height:");
            ui.add(egui::DragValue::new(&mut app.cam.shape_height).suffix("mm"));
        });
        ui.horizontal(|ui| {
            ui.label("Radius:");
            ui.add(egui::DragValue::new(&mut app.cam.shape_radius).suffix("mm"));
        });

        ui.horizontal(|ui| {
            if ui.button("Rectangle").clicked() {
                app.generate_rectangle();
                app.create_job_from_generated_gcode(
                    "Rectangle",
                    crate::jobs::JobType::CAMOperation,
                );
            }
            if ui.button("Circle").clicked() {
                app.generate_circle();
                app.create_job_from_generated_gcode("Circle", crate::jobs::JobType::CAMOperation);
            }
        });
    });
}
