use crate::app::GcodeKitApp;
use egui;

/// Renders the central panel with tab navigation and content.
/// Includes the main tab interface for G-code editor, 3D visualizer, device console,
/// job manager, and designer, plus the add material dialog.
pub fn show_center_panel(app: &mut GcodeKitApp, ctx: &egui::Context) {
    // Central panel with tabs
    egui::CentralPanel::default().show(ctx, |ui| {
        // Top central panel with quick actions and status
        crate::layout::show_top_central_panel(app, ui);

        ui.horizontal(|ui| {
            ui.selectable_value(
                &mut app.ui.selected_tab,
                crate::types::Tab::GcodeEditor,
                "G-code Editor",
            );
            ui.selectable_value(
                &mut app.ui.selected_tab,
                crate::types::Tab::Visualizer3D,
                "3D Visualizer",
            );
            ui.selectable_value(
                &mut app.ui.selected_tab,
                crate::types::Tab::DeviceConsole,
                "Device Console",
            );
            ui.selectable_value(
                &mut app.ui.selected_tab,
                crate::types::Tab::JobManager,
                "Job Manager",
            );
            ui.selectable_value(
                &mut app.ui.selected_tab,
                crate::types::Tab::FeedsSpeeds,
                "Feeds & Speeds",
            );
            ui.selectable_value(
                &mut app.ui.selected_tab,
                crate::types::Tab::Designer,
                "Designer",
            );
        });
        ui.separator();

        match app.ui.selected_tab {
            crate::types::Tab::GcodeEditor => {
                crate::ui::tabs::show_gcode_editor_tab(app, ui);
            }
            crate::types::Tab::Visualizer3D => {
                crate::ui::tabs::show_visualizer_3d_tab(app, ui);
            }
            crate::types::Tab::DeviceConsole => {
                crate::ui::tabs::show_device_console_tab(app, ui);
            }
            crate::types::Tab::JobManager => {
                crate::ui::tabs::show_job_manager_tab(app, ui);
            }
            crate::types::Tab::FeedsSpeeds => {
                crate::ui::tabs::show_feeds_speeds_tab(app, ui);
            }
            crate::types::Tab::Designer => {
                crate::ui::tabs::show_designer_tab(app, ui);
            }
            crate::types::Tab::Scripting => {
                ui.vertical(|ui| {
                    ui.label("Automation Scripting");
                    ui.separator();
                    ui.label("Use Rhai scripting to automate operations:");
                    ui.text_edit_multiline(&mut app.script_content);
                    if ui.button("Run Script").clicked() {
                        app.run_script();
                    }
                });
            }
        }
    });

    // Add Material Dialog
    if app.ui.show_add_material_dialog {
        let mut open = true;
        egui::Window::new("Add New Material")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.heading("Material Information");
                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut app.ui.new_material_name);
                    });

                    ui.horizontal(|ui| {
                        ui.label("Type:");
                        egui::ComboBox::from_label("")
                            .selected_text(format!("{:?}", app.ui.new_material_type))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut app.ui.new_material_type,
                                    crate::materials::MaterialType::Wood,
                                    "Wood",
                                );
                                ui.selectable_value(
                                    &mut app.ui.new_material_type,
                                    crate::materials::MaterialType::Plastic,
                                    "Plastic",
                                );
                                ui.selectable_value(
                                    &mut app.ui.new_material_type,
                                    crate::materials::MaterialType::Metal,
                                    "Metal",
                                );
                                ui.selectable_value(
                                    &mut app.ui.new_material_type,
                                    crate::materials::MaterialType::Composite,
                                    "Composite",
                                );
                                ui.selectable_value(
                                    &mut app.ui.new_material_type,
                                    crate::materials::MaterialType::Stone,
                                    "Stone",
                                );
                                ui.selectable_value(
                                    &mut app.ui.new_material_type,
                                    crate::materials::MaterialType::Foam,
                                    "Foam",
                                );
                                ui.selectable_value(
                                    &mut app.ui.new_material_type,
                                    crate::materials::MaterialType::Other,
                                    "Other",
                                );
                            });
                    });

                    ui.separator();
                    ui.heading("Physical Properties");

                    ui.horizontal(|ui| {
                        ui.label("Density (kg/m³):");
                        ui.add(egui::DragValue::new(&mut app.ui.new_material_density).speed(10.0));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Hardness (HB):");
                        ui.add(egui::DragValue::new(&mut app.ui.new_material_hardness).speed(1.0));
                    });

                    ui.separator();
                    ui.heading("Machining Parameters");

                    ui.horizontal(|ui| {
                        ui.label("Cutting Speed (m/min):");
                        ui.add(
                            egui::DragValue::new(&mut app.ui.new_material_cutting_speed)
                                .speed(10.0),
                        );
                    });

                    ui.horizontal(|ui| {
                        ui.label("Feed Rate (mm/min):");
                        ui.add(
                            egui::DragValue::new(&mut app.ui.new_material_feed_rate).speed(10.0),
                        );
                    });

                    ui.horizontal(|ui| {
                        ui.label("Spindle Speed (RPM):");
                        ui.add(
                            egui::DragValue::new(&mut app.ui.new_material_spindle_speed)
                                .speed(100.0),
                        );
                    });

                    ui.separator();
                    ui.heading("Tool Recommendations");

                    ui.horizontal(|ui| {
                        ui.label("Tool Material:");
                        ui.text_edit_singleline(&mut app.ui.new_material_tool_material);
                    });

                    ui.horizontal(|ui| {
                        ui.label("Tool Coating:");
                        ui.text_edit_singleline(&mut app.ui.new_material_tool_coating);
                    });

                    ui.horizontal(|ui| {
                        ui.label("Chip Load Min (mm):");
                        ui.add(
                            egui::DragValue::new(&mut app.ui.new_material_chip_load_min)
                                .speed(0.01),
                        );
                    });

                    ui.horizontal(|ui| {
                        ui.label("Chip Load Max (mm):");
                        ui.add(
                            egui::DragValue::new(&mut app.ui.new_material_chip_load_max)
                                .speed(0.01),
                        );
                    });

                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.label("Notes:");
                        ui.text_edit_multiline(&mut app.ui.new_material_notes);
                    });

                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("Save Material").clicked() {
                            if !app.ui.new_material_name.trim().is_empty() {
                                let mut material = crate::materials::MaterialProperties::new(
                                    &app.ui.new_material_name,
                                    app.ui.new_material_type.clone(),
                                    crate::materials::MaterialSubtype::Custom, // Default to Custom for now
                                )
                                .with_density(app.ui.new_material_density)
                                .with_hardness(app.ui.new_material_hardness);

                                if app.ui.new_material_cutting_speed > 0.0 {
                                    material.cutting_speed =
                                        Some(app.ui.new_material_cutting_speed);
                                }
                                if app.ui.new_material_feed_rate > 0.0 {
                                    material.feed_rate = Some(app.ui.new_material_feed_rate);
                                }
                                if app.ui.new_material_spindle_speed > 0.0 {
                                    material.spindle_speed =
                                        Some(app.ui.new_material_spindle_speed);
                                }

                                material.recommended_tool_material =
                                    app.ui.new_material_tool_material.clone();
                                if !app.ui.new_material_tool_coating.trim().is_empty() {
                                    material.recommended_coating =
                                        Some(app.ui.new_material_tool_coating.clone());
                                }
                                if app.ui.new_material_chip_load_min > 0.0 {
                                    material.chip_load_min =
                                        Some(app.ui.new_material_chip_load_min);
                                }
                                if app.ui.new_material_chip_load_max > 0.0 {
                                    material.chip_load_max =
                                        Some(app.ui.new_material_chip_load_max);
                                }

                                material.notes = app.ui.new_material_notes.clone();

                                app.material_database.add_material(material);
                                app.machine.status_message =
                                    format!("Added material: {}", app.ui.new_material_name);

                                // Reset dialog fields
                                app.reset_add_material_dialog();
                                app.ui.show_add_material_dialog = false;
                            } else {
                                app.machine.status_message =
                                    "Material name cannot be empty".to_string();
                            }
                        }

                        if ui.button("Cancel").clicked() {
                            app.reset_add_material_dialog();
                            app.ui.show_add_material_dialog = false;
                        }
                    });
                });
            });
        if !open {
            app.reset_add_material_dialog();
            app.ui.show_add_material_dialog = false;
        }
    }
}
