use crate::GcodeKitApp;
use crate::designer::parametric_design::{ParametricConfig, ParametricDesigner};
use eframe::egui;

pub fn show_parametric_design_widget(ui: &mut egui::Ui, app: &mut GcodeKitApp) {
    ui.group(|ui| {
        ui.label("Parametric Design");

        ui.horizontal(|ui| {
            ui.label("Shape Template:");
            egui::ComboBox::from_label("")
                .selected_text("Custom")
                .show_ui(ui, |ui| {
                    if ui
                        .selectable_value(
                            &mut app.parametric_shape_type,
                            "Custom".to_string(),
                            "Custom",
                        )
                        .clicked()
                    {
                        app.parametric_config.script_template =
                            ParametricDesigner::generate_shape("custom", &app.parametric_config);
                    }
                    if ui
                        .selectable_value(
                            &mut app.parametric_shape_type,
                            "circle".to_string(),
                            "Circle",
                        )
                        .clicked()
                    {
                        app.parametric_config.script_template =
                            ParametricDesigner::generate_shape("circle", &app.parametric_config);
                    }
                    if ui
                        .selectable_value(
                            &mut app.parametric_shape_type,
                            "ellipse".to_string(),
                            "Ellipse",
                        )
                        .clicked()
                    {
                        app.parametric_config.script_template =
                            ParametricDesigner::generate_shape("ellipse", &app.parametric_config);
                    }
                    if ui
                        .selectable_value(
                            &mut app.parametric_shape_type,
                            "spiral".to_string(),
                            "Spiral",
                        )
                        .clicked()
                    {
                        app.parametric_config.script_template =
                            ParametricDesigner::generate_shape("spiral", &app.parametric_config);
                    }
                    if ui
                        .selectable_value(
                            &mut app.parametric_shape_type,
                            "star".to_string(),
                            "Star",
                        )
                        .clicked()
                    {
                        app.parametric_config.script_template =
                            ParametricDesigner::generate_shape("star", &app.parametric_config);
                    }
                    if ui
                        .selectable_value(
                            &mut app.parametric_shape_type,
                            "heart".to_string(),
                            "Heart",
                        )
                        .clicked()
                    {
                        app.parametric_config.script_template =
                            ParametricDesigner::generate_shape("heart", &app.parametric_config);
                    }
                    if ui
                        .selectable_value(
                            &mut app.parametric_shape_type,
                            "wave".to_string(),
                            "Wave",
                        )
                        .clicked()
                    {
                        app.parametric_config.script_template =
                            ParametricDesigner::generate_shape("wave", &app.parametric_config);
                    }
                });
        });

        ui.horizontal(|ui| {
            ui.label("Steps:");
            ui.add(egui::DragValue::new(&mut app.parametric_config.steps).range(10..=1000));
        });

        ui.label("Variables:");
        for (name, value) in &mut app.parametric_config.variables {
            ui.horizontal(|ui| {
                ui.label(format!("{}:", name));
                ui.add(egui::DragValue::new(value).speed(0.1));
            });
        }

        ui.label("Script:");
        let mut script = app.parametric_config.script_template.clone();
        if ui.code_editor(&mut script).changed() {
            app.parametric_config.script_template = script;
        }

        // Validate script
        if let Err(error) =
            ParametricDesigner::validate_script(&app.parametric_config.script_template)
        {
            ui.colored_label(egui::Color32::RED, format!("Script Error: {}", error));
        } else {
            ui.colored_label(egui::Color32::GREEN, "Script is valid");
        }

        if ui.button("Create Parametric Shape").clicked() {
            // TODO: Implement parametric shape creation
            // app.create_parametric_shape();
        }

        if ui.button("Evaluate Selected Parametric").clicked() {
            // TODO: Implement parametric evaluation
            // app.evaluate_selected_parametric();
        }
    });
}
