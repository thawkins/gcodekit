use crate::cam::stl;
use crate::cam::types::{CAMOperation, ContourDirection};
use crate::designer::DesignerState;
use eframe::egui;
use std::path::PathBuf;

/// Show the CAM operations widget
pub fn show_cam_operations_widget(ui: &mut egui::Ui, designer: &mut DesignerState) {
    ui.heading("CAM Operations");

    ui.separator();

    // CAM Operation selection
    ui.horizontal(|ui| {
        ui.label("Operation:");
        egui::ComboBox::from_label("")
            .selected_text(format!("{:?}", designer.selected_cam_operation))
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut designer.selected_cam_operation,
                    CAMOperation::Contour2D {
                        depth: 5.0,
                        stepover: 1.0,
                        direction: ContourDirection::Climb,
                    },
                    "2D Contour",
                );
                ui.selectable_value(
                    &mut designer.selected_cam_operation,
                    CAMOperation::SideProfile {
                        depth: 5.0,
                        stepover: 1.0,
                        direction: ContourDirection::Climb,
                        wall_angle: 0.0,
                    },
                    "Side Profile",
                );
                ui.selectable_value(
                    &mut designer.selected_cam_operation,
                    CAMOperation::Waterline {
                        min_z: -5.0,
                        max_z: 0.0,
                        stepdown: 1.0,
                        stepover: 2.0,
                    },
                    "Waterline",
                );
                ui.selectable_value(
                    &mut designer.selected_cam_operation,
                    CAMOperation::Scanline {
                        min_z: -5.0,
                        max_z: 0.0,
                        stepdown: 1.0,
                        stepover: 2.0,
                        angle: 0.0,
                    },
                    "Scanline",
                );
                ui.selectable_value(
                    &mut designer.selected_cam_operation,
                    CAMOperation::Turning {
                        diameter: 50.0,
                        length: 100.0,
                        finish_pass: 0.5,
                        roughing_feed: 0.1,
                        finishing_feed: 0.05,
                    },
                    "Lathe Turning",
                );
                ui.selectable_value(
                    &mut designer.selected_cam_operation,
                    CAMOperation::Facing {
                        diameter: 50.0,
                        width: 20.0,
                        depth: 2.0,
                        stepover: 1.0,
                    },
                    "Lathe Facing",
                );
                ui.selectable_value(
                    &mut designer.selected_cam_operation,
                    CAMOperation::Threading {
                        major_diameter: 10.0,
                        minor_diameter: 8.0,
                        pitch: 1.5,
                        length: 20.0,
                    },
                    "Lathe Threading",
                );
            });
    });

    ui.separator();

    // STL Import section
    ui.collapsing("3D Model Import", |ui| {
        ui.label("Import STL files for 3D machining operations");

        if ui.button("Import STL File").clicked() {
            // TODO: Open file dialog and import STL
            // For now, we'll use a placeholder path for testing
            let test_stl_path = PathBuf::from("assets/test.stl");
            match stl::load_stl(&test_stl_path) {
                Ok(mesh) => {
                    designer.current_mesh = Some(mesh);
                    ui.label("STL loaded successfully!");
                }
                Err(e) => {
                    ui.label(format!("Error loading STL: {}", e));
                }
            }
        }

        if let Some(mesh) = &designer.current_mesh {
            ui.label(format!(
                "Loaded mesh with {} triangles",
                mesh.triangles.len()
            ));
            ui.label(format!(
                "Bounds: {:.1} x {:.1} x {:.1} mm",
                mesh.bounds.max.x - mesh.bounds.min.x,
                mesh.bounds.max.y - mesh.bounds.min.y,
                mesh.bounds.max.z - mesh.bounds.min.z
            ));
        } else {
            ui.label("No mesh loaded");
        }
    });

    ui.separator();

    // CAM Parameters
    ui.collapsing("CAM Parameters", |ui| {
        ui.horizontal(|ui| {
            ui.label("Tool Diameter:");
            ui.add(egui::DragValue::new(&mut designer.cam_params.tool_diameter).speed(0.1));
            ui.label("mm");
        });

        ui.horizontal(|ui| {
            ui.label("Stepdown:");
            ui.add(egui::DragValue::new(&mut designer.cam_params.stepdown).speed(0.1));
            ui.label("mm");
        });

        ui.horizontal(|ui| {
            ui.label("Stepover:");
            ui.add(egui::DragValue::new(&mut designer.cam_params.stepover).speed(0.1));
            ui.label("mm");
        });

        ui.horizontal(|ui| {
            ui.label("Feed Rate:");
            ui.add(egui::DragValue::new(&mut designer.cam_params.feed_rate).speed(1.0));
            ui.label("mm/min");
        });

        ui.horizontal(|ui| {
            ui.label("Plunge Rate:");
            ui.add(egui::DragValue::new(&mut designer.cam_params.plunge_rate).speed(1.0));
            ui.label("mm/min");
        });

        ui.horizontal(|ui| {
            ui.label("Spindle Speed:");
            ui.add(egui::DragValue::new(&mut designer.cam_params.spindle_speed).speed(100));
            ui.label("RPM");
        });

        ui.horizontal(|ui| {
            ui.label("Safe Z:");
            ui.add(egui::DragValue::new(&mut designer.cam_params.safe_z).speed(0.1));
            ui.label("mm");
        });

        ui.horizontal(|ui| {
            ui.label("Stock Surface:");
            ui.add(egui::DragValue::new(&mut designer.cam_params.stock_surface).speed(0.1));
            ui.label("mm");
        });

        ui.horizontal(|ui| {
            ui.label("Final Depth:");
            ui.add(egui::DragValue::new(&mut designer.cam_params.final_depth).speed(0.1));
            ui.label("mm");
        });

        ui.separator();

        ui.checkbox(&mut designer.cam_params.tabs_enabled, "Enable Tabs");
        if designer.cam_params.tabs_enabled {
            ui.horizontal(|ui| {
                ui.label("Tab Height:");
                ui.add(egui::DragValue::new(&mut designer.cam_params.tab_height).speed(0.1));
                ui.label("mm");
            });
            ui.horizontal(|ui| {
                ui.label("Tab Width:");
                ui.add(egui::DragValue::new(&mut designer.cam_params.tab_width).speed(0.1));
                ui.label("mm");
            });
        }

        ui.separator();

        ui.checkbox(&mut designer.cam_params.lead_in_enabled, "Enable Lead In");
        if designer.cam_params.lead_in_enabled {
            ui.horizontal(|ui| {
                ui.label("Lead In Length:");
                ui.add(egui::DragValue::new(&mut designer.cam_params.lead_in_length).speed(0.1));
                ui.label("mm");
            });
        }

        ui.checkbox(&mut designer.cam_params.lead_out_enabled, "Enable Lead Out");
        if designer.cam_params.lead_out_enabled {
            ui.horizontal(|ui| {
                ui.label("Lead Out Length:");
                ui.add(egui::DragValue::new(&mut designer.cam_params.lead_out_length).speed(0.1));
                ui.label("mm");
            });
        }
    });

    ui.separator();

    // Generate Toolpath button
    if ui.button("Generate Toolpath").clicked() {
        use crate::cam::toolpaths;

        let gcode = match &designer.selected_cam_operation {
            CAMOperation::Waterline { .. } => {
                if let Some(mesh) = &designer.current_mesh {
                    toolpaths::generate_waterline_toolpath(
                        mesh,
                        &designer.cam_params,
                        &designer.selected_cam_operation,
                    )
                } else {
                    ui.label("No 3D mesh loaded for waterline machining");
                    return;
                }
            }
            CAMOperation::Scanline { .. } => {
                if let Some(mesh) = &designer.current_mesh {
                    toolpaths::generate_scanline_toolpath(
                        mesh,
                        &designer.cam_params,
                        &designer.selected_cam_operation,
                    )
                } else {
                    ui.label("No 3D mesh loaded for scanline machining");
                    return;
                }
            }
            _ => {
                // For 2D operations, use existing shape-based generation
                // TODO: Implement 2D toolpath generation
                ui.label("2D toolpath generation not yet implemented");
                return;
            }
        };

        // TODO: Store or display the generated G-code
        ui.label(format!("Generated {} lines of G-code", gcode.len()));
    }

    // Operation-specific parameters
    match &designer.selected_cam_operation {
        CAMOperation::None => {} // No specific parameters
        CAMOperation::Contour2D { direction, .. } => {
            ui.collapsing("Contour Parameters", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Depth:");
                    ui.add(egui::DragValue::new(&mut 5.0).speed(0.1)); // TODO: Make this editable
                    ui.label("mm");
                });
                ui.horizontal(|ui| {
                    ui.label("Stepover:");
                    ui.add(egui::DragValue::new(&mut 1.0).speed(0.1));
                    ui.label("mm");
                });
                ui.horizontal(|ui| {
                    ui.label("Direction:");
                    egui::ComboBox::from_label("")
                        .selected_text(format!("{:?}", direction))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut ContourDirection::Climb,
                                ContourDirection::Climb,
                                "Climb",
                            );
                            ui.selectable_value(
                                &mut ContourDirection::Conventional,
                                ContourDirection::Conventional,
                                "Conventional",
                            );
                            ui.selectable_value(
                                &mut ContourDirection::Clockwise,
                                ContourDirection::Clockwise,
                                "Clockwise",
                            );
                            ui.selectable_value(
                                &mut ContourDirection::CounterClockwise,
                                ContourDirection::CounterClockwise,
                                "Counter-Clockwise",
                            );
                        });
                });
            });
        }
        CAMOperation::SideProfile { .. } => {
            ui.collapsing("Side Profile Parameters", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Depth:");
                    ui.add(egui::DragValue::new(&mut 5.0).speed(0.1));
                    ui.label("mm");
                });
                ui.horizontal(|ui| {
                    ui.label("Stepover:");
                    ui.add(egui::DragValue::new(&mut 1.0).speed(0.1));
                    ui.label("mm");
                });
                ui.horizontal(|ui| {
                    ui.label("Wall Angle:");
                    ui.add(egui::DragValue::new(&mut 0.0).speed(0.1));
                    ui.label("degrees");
                });
            });
        }
        CAMOperation::Waterline { .. } => {
            ui.collapsing("Waterline Parameters", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Min Z:");
                    ui.add(egui::DragValue::new(&mut -5.0).speed(0.1));
                    ui.label("mm");
                });
                ui.horizontal(|ui| {
                    ui.label("Max Z:");
                    ui.add(egui::DragValue::new(&mut 0.0).speed(0.1));
                    ui.label("mm");
                });
                ui.horizontal(|ui| {
                    ui.label("Stepdown:");
                    ui.add(egui::DragValue::new(&mut 1.0).speed(0.1));
                    ui.label("mm");
                });
                ui.horizontal(|ui| {
                    ui.label("Stepover:");
                    ui.add(egui::DragValue::new(&mut 2.0).speed(0.1));
                    ui.label("mm");
                });
            });
        }
        CAMOperation::Scanline { .. } => {
            ui.collapsing("Scanline Parameters", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Min Z:");
                    ui.add(egui::DragValue::new(&mut -5.0).speed(0.1));
                    ui.label("mm");
                });
                ui.horizontal(|ui| {
                    ui.label("Max Z:");
                    ui.add(egui::DragValue::new(&mut 0.0).speed(0.1));
                    ui.label("mm");
                });
                ui.horizontal(|ui| {
                    ui.label("Stepdown:");
                    ui.add(egui::DragValue::new(&mut 1.0).speed(0.1));
                    ui.label("mm");
                });
                ui.horizontal(|ui| {
                    ui.label("Stepover:");
                    ui.add(egui::DragValue::new(&mut 2.0).speed(0.1));
                    ui.label("mm");
                });
                ui.horizontal(|ui| {
                    ui.label("Angle:");
                    ui.add(egui::DragValue::new(&mut 0.0).speed(1.0));
                    ui.label("degrees");
                });
            });
        }
        CAMOperation::Turning { .. } => {
            ui.collapsing("Turning Parameters", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Diameter:");
                    ui.add(egui::DragValue::new(&mut 50.0).speed(0.1));
                    ui.label("mm");
                });
                ui.horizontal(|ui| {
                    ui.label("Length:");
                    ui.add(egui::DragValue::new(&mut 100.0).speed(0.1));
                    ui.label("mm");
                });
                ui.horizontal(|ui| {
                    ui.label("Finish Pass:");
                    ui.add(egui::DragValue::new(&mut 0.5).speed(0.1));
                    ui.label("mm");
                });
            });
        }
        CAMOperation::Facing { .. } => {
            ui.collapsing("Facing Parameters", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Diameter:");
                    ui.add(egui::DragValue::new(&mut 50.0).speed(0.1));
                    ui.label("mm");
                });
                ui.horizontal(|ui| {
                    ui.label("Width:");
                    ui.add(egui::DragValue::new(&mut 20.0).speed(0.1));
                    ui.label("mm");
                });
                ui.horizontal(|ui| {
                    ui.label("Depth:");
                    ui.add(egui::DragValue::new(&mut 2.0).speed(0.1));
                    ui.label("mm");
                });
                ui.horizontal(|ui| {
                    ui.label("Stepover:");
                    ui.add(egui::DragValue::new(&mut 1.0).speed(0.1));
                    ui.label("mm");
                });
            });
        }
        CAMOperation::Threading { .. } => {
            ui.collapsing("Threading Parameters", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Major Diameter:");
                    ui.add(egui::DragValue::new(&mut 10.0).speed(0.1));
                    ui.label("mm");
                });
                ui.horizontal(|ui| {
                    ui.label("Minor Diameter:");
                    ui.add(egui::DragValue::new(&mut 8.0).speed(0.1));
                    ui.label("mm");
                });
                ui.horizontal(|ui| {
                    ui.label("Pitch:");
                    ui.add(egui::DragValue::new(&mut 1.5).speed(0.1));
                    ui.label("mm");
                });
                ui.horizontal(|ui| {
                    ui.label("Length:");
                    ui.add(egui::DragValue::new(&mut 20.0).speed(0.1));
                    ui.label("mm");
                });
            });
        }
    }
}
