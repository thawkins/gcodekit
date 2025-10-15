use crate::app::GcodeKitApp;
use crate::communication::ControllerType;
use crate::types::Tab;
use egui;

/// Renders the top menu bar with File, Machine, View, Tools, and Help menus.
/// Provides access to application functions and settings.
pub fn show_top_menu(app: &mut GcodeKitApp, ctx: &egui::Context) {
    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.push_id("file_menu", |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open G-code...").clicked() {
                        app.load_gcode_file();
                    }
                    if ui.button("Save G-code...").clicked() {
                        app.save_gcode_file();
                    }
                    ui.separator();
                    if ui.button("Import Vector...").clicked() {
                        app.import_vector_file();
                    }
                    if ui.button("Export G-code...").clicked() {
                        app.save_gcode_file();
                    }
                    ui.separator();
                    if ui.button("Exit").clicked() {
                        std::process::exit(0);
                    }
                });
            });
            ui.push_id("machine_menu", |ui| {
                ui.menu_button("Machine", |ui| {
                    ui.menu_button("Controller Type", |ui| {
                        if ui
                            .selectable_value(
                                &mut app.machine.controller_type,
                                ControllerType::Grbl,
                                "GRBL",
                            )
                            .clicked()
                        {
                            app.machine.communication =
                                Box::new(crate::communication::GrblCommunication::default());
                            app.machine.communication.refresh_ports();
                        }
                    });
                    ui.separator();
                    if ui.button("Connect").clicked() {
                        let _ = app.machine.communication.connect();
                    }
                    if ui.button("Disconnect").clicked() {
                        app.machine.communication.disconnect();
                    }
                    ui.separator();
                    if ui.button("Home All").clicked() {
                        app.machine.communication.home_all_axes();
                    }
                    if ui.button("Reset").clicked() {
                        // TODO: Reset machine
                    }
                    ui.separator();
                    if app.machine.controller_type == ControllerType::Grbl {
                        ui.menu_button("Work Coordinate System", |_ui| {
                            // This is GRBL-specific, need to handle properly
                            // For now, skip
                        });
                    }
                });
            });
            ui.push_id("view_menu", |ui| {
                ui.menu_button("View", |ui| {
                    if ui.button("G-code Editor").clicked() {
                        app.ui.selected_tab = Tab::GcodeEditor;
                    }
                    if ui.button("3D Visualizer").clicked() {
                        app.ui.selected_tab = Tab::Visualizer3D;
                    }
                    if ui.button("Device Console").clicked() {
                        app.ui.selected_tab = Tab::DeviceConsole;
                    }
                    ui.separator();
                    ui.checkbox(&mut app.ui.show_left_panel, "Machine Control Panel");
                    ui.checkbox(&mut app.ui.show_right_panel, "CAM Functions Panel");
                    ui.separator();
                    if ui.button("Refresh Ports").clicked() {
                        app.machine.communication.refresh_ports();
                    }
                });
            });
            ui.push_id("tools_menu", |ui| {
                ui.menu_button("Tools", |ui| {
                    if ui.button("Generate Rectangle").clicked() {
                        app.generate_rectangle();
                    }
                    if ui.button("Generate Circle").clicked() {
                        app.generate_circle();
                    }
                    ui.separator();
                    if ui.button("Image Engraving").clicked() {
                        app.load_image_for_engraving();
                    }
                    if ui.button("Tabbed Box").clicked() {
                        app.generate_tabbed_box();
                    }
                    if ui.button("Jigsaw Puzzle").clicked() {
                        app.generate_jigsaw();
                    }
                    ui.separator();
                    if ui.button("Optimize G-code").clicked() {
                        app.optimize_gcode();
                    }
                });
            });
            ui.push_id("help_menu", |ui| {
                ui.menu_button("Help", |ui| {
                    if ui.button("About gcodekit").clicked() {
                        // TODO: Show about dialog
                    }
                    if ui.button("GRBL Documentation").clicked() {
                        // TODO: Open GRBL docs
                    }
                });
            });
        });
    });
}
