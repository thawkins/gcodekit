use crate::GcodeKitApp;
use crate::types::Tab;
use crate::ui::tabs::{
    show_designer_tab, show_device_console_tab, show_gcode_editor_tab, show_job_manager_tab,
    show_visualizer_3d_tab,
};
use eframe::egui;

impl GcodeKitApp {
    pub fn show_central_panel(&mut self, ctx: &egui::Context) {
        // Central panel with tabs
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.ui.selected_tab, Tab::GcodeEditor, "G-code Editor");
                ui.selectable_value(
                    &mut self.ui.selected_tab,
                    Tab::Visualizer3D,
                    "3D Visualizer",
                );
                ui.selectable_value(
                    &mut self.ui.selected_tab,
                    Tab::DeviceConsole,
                    "Device Console",
                );
                ui.selectable_value(&mut self.ui.selected_tab, Tab::JobManager, "Job Manager");
                ui.selectable_value(&mut self.ui.selected_tab, Tab::Designer, "Designer");
            });
            ui.separator();

            match self.ui.selected_tab {
                Tab::GcodeEditor => {
                    show_gcode_editor_tab(self, ui);
                }
                Tab::Visualizer3D => {
                    show_visualizer_3d_tab(self, ui);
                }
                Tab::DeviceConsole => {
                    show_device_console_tab(self, ui);
                }
                Tab::JobManager => {
                    show_job_manager_tab(self, ui);
                }
                Tab::Designer => {
                    show_designer_tab(self, ui);
                }
                Tab::Scripting => {
                    ui.vertical(|ui| {
                        ui.label("Automation Scripting");
                        ui.separator();
                        ui.label("Use Rhai scripting to automate operations:");
                        ui.text_edit_multiline(&mut self.script_content);
                        if ui.button("Run Script").clicked() {
                            self.run_script();
                        }
                    });
                }
            }
        });

        // Add Material Dialog
        if self.ui.show_add_material_dialog {
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
                            ui.text_edit_singleline(&mut self.ui.new_material_name);
                        });

                        ui.horizontal(|ui| {
                            ui.label("Type:");
                            egui::ComboBox::from_label("")
                                .selected_text(format!("{:?}", self.ui.new_material_type))
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut self.ui.new_material_type,
                                        crate::materials::MaterialType::Wood,
                                        "Wood",
                                    );
                                    ui.selectable_value(
                                        &mut self.ui.new_material_type,
                                        crate::materials::MaterialType::Plastic,
                                        "Plastic",
                                    );
                                    ui.selectable_value(
                                        &mut self.ui.new_material_type,
                                        crate::materials::MaterialType::Metal,
                                        "Metal",
                                    );
                                    ui.selectable_value(
                                        &mut self.ui.new_material_type,
                                        crate::materials::MaterialType::Composite,
                                        "Composite",
                                    );
                                    ui.selectable_value(
                                        &mut self.ui.new_material_type,
                                        crate::materials::MaterialType::Stone,
                                        "Stone",
                                    );
                                    ui.selectable_value(
                                        &mut self.ui.new_material_type,
                                        crate::materials::MaterialType::Foam,
                                        "Foam",
                                    );
                                    ui.selectable_value(
                                        &mut self.ui.new_material_type,
                                        crate::materials::MaterialType::Other,
                                        "Other",
                                    );
                                });
                        });

                        ui.separator();
                        ui.heading("Physical Properties");

                        ui.horizontal(|ui| {
                            ui.label("Density (kg/m³):");
                            ui.add(
                                egui::DragValue::new(&mut self.ui.new_material_density).speed(10.0),
                            );
                        });

                        ui.horizontal(|ui| {
                            ui.label("Hardness (HB):");
                            ui.add(
                                egui::DragValue::new(&mut self.ui.new_material_hardness).speed(1.0),
                            );
                        });

                        ui.separator();
                        ui.heading("Machining Parameters");

                        ui.horizontal(|ui| {
                            ui.label("Cutting Speed (m/min):");
                            ui.add(
                                egui::DragValue::new(&mut self.ui.new_material_cutting_speed)
                                    .speed(10.0),
                            );
                        });

                        ui.horizontal(|ui| {
                            ui.label("Feed Rate (mm/min):");
                            ui.add(
                                egui::DragValue::new(&mut self.ui.new_material_feed_rate)
                                    .speed(10.0),
                            );
                        });

                        ui.horizontal(|ui| {
                            ui.label("Spindle Speed (RPM):");
                            ui.add(
                                egui::DragValue::new(&mut self.ui.new_material_spindle_speed)
                                    .speed(100.0),
                            );
                        });

                        ui.separator();
                        ui.heading("Tool Recommendations");

                        ui.horizontal(|ui| {
                            ui.label("Tool Material:");
                            ui.text_edit_singleline(&mut self.ui.new_material_tool_material);
                        });

                        ui.horizontal(|ui| {
                            ui.label("Tool Coating:");
                            ui.text_edit_singleline(&mut self.ui.new_material_tool_coating);
                        });

                        ui.horizontal(|ui| {
                            ui.label("Chip Load Min (mm):");
                            ui.add(
                                egui::DragValue::new(&mut self.ui.new_material_chip_load_min)
                                    .speed(0.01),
                            );
                        });

                        ui.horizontal(|ui| {
                            ui.label("Chip Load Max (mm):");
                            ui.add(
                                egui::DragValue::new(&mut self.ui.new_material_chip_load_max)
                                    .speed(0.01),
                            );
                        });

                        ui.separator();
                        ui.horizontal(|ui| {
                            ui.label("Notes:");
                            ui.text_edit_multiline(&mut self.ui.new_material_notes);
                        });

                        ui.separator();
                        ui.horizontal(|ui| {
                            if ui.button("Save Material").clicked() {
                                if !self.ui.new_material_name.trim().is_empty() {
                                    let mut material = crate::materials::MaterialProperties::new(
                                        &self.ui.new_material_name,
                                        self.ui.new_material_type.clone(),
                                        crate::materials::MaterialSubtype::Custom, // Default to Custom for now
                                    )
                                    .with_density(self.ui.new_material_density)
                                    .with_hardness(self.ui.new_material_hardness);

                                    if self.ui.new_material_cutting_speed > 0.0 {
                                        material.cutting_speed =
                                            Some(self.ui.new_material_cutting_speed);
                                    }
                                    if self.ui.new_material_feed_rate > 0.0 {
                                        material.feed_rate = Some(self.ui.new_material_feed_rate);
                                    }
                                    if self.ui.new_material_spindle_speed > 0.0 {
                                        material.spindle_speed =
                                            Some(self.ui.new_material_spindle_speed);
                                    }

                                    material.recommended_tool_material =
                                        self.ui.new_material_tool_material.clone();
                                    if !self.ui.new_material_tool_coating.trim().is_empty() {
                                        material.recommended_coating =
                                            Some(self.ui.new_material_tool_coating.clone());
                                    }
                                    if self.ui.new_material_chip_load_min > 0.0 {
                                        material.chip_load_min =
                                            Some(self.ui.new_material_chip_load_min);
                                    }
                                    if self.ui.new_material_chip_load_max > 0.0 {
                                        material.chip_load_max =
                                            Some(self.ui.new_material_chip_load_max);
                                    }

                                    material.notes = self.ui.new_material_notes.clone();

                                    self.material_database.add_material(material);
                                    self.machine.status_message =
                                        format!("Added material: {}", self.ui.new_material_name);

                                    // Reset dialog fields
                                    self.reset_add_material_dialog();
                                    self.ui.show_add_material_dialog = false;
                                } else {
                                    self.machine.status_message =
                                        "Material name cannot be empty".to_string();
                                }
                            }

                            if ui.button("Cancel").clicked() {
                                self.reset_add_material_dialog();
                                self.ui.show_add_material_dialog = false;
                            }
                        });
                    });
                });
            if !open {
                self.reset_add_material_dialog();
                self.ui.show_add_material_dialog = false;
            }
        }
    }
}
