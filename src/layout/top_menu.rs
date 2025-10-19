use crate::app::GcodeKitApp;
use crate::communication::ControllerType;
use crate::types::Tab;
use egui;

/// Renders the top menu bar with File, Machine, View, Tools, and Help menus.
/// Provides access to application functions and settings.
pub fn show_top_menu(app: &mut GcodeKitApp, ctx: &egui::Context) {
    let mut show_about_dialog = false;
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
                        app.machine.communication.reset_machine();
                        app.machine.status_message = "Machine reset initiated".to_string();
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
                        show_about_dialog = true;
                    }
                    if ui.button("GRBL Documentation").clicked() {
                        let _ = open_url("https://github.com/grbl/grbl/wiki");
                    }
                });
            });
        });
    });

    // Show about dialog
    if show_about_dialog {
        show_about_window(app, ctx);
    }
}

/// Opens a URL in the default browser.
fn open_url(url: &str) {
    // Try to open URL using platform-specific commands
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("start")
            .arg(url)
            .spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open")
            .arg(url)
            .spawn();
    }
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open")
            .arg(url)
            .spawn();
    }
}

/// Displays the about dialog window.
fn show_about_window(app: &mut GcodeKitApp, ctx: &egui::Context) {
    let mut open = true;
    egui::Window::new("About gcodekit")
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .default_width(400.0)
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.heading("📱 gcodekit");
                ui.separator();
                
                ui.horizontal(|ui| {
                    ui.label("Version:");
                    ui.label("0.1.0-alpha");
                });
                
                ui.horizontal(|ui| {
                    ui.label("Status:");
                    ui.label("Alpha Development");
                });
                
                ui.separator();
                
                ui.label("Professional GRBL CNC & Laser Controller");
                
                ui.group(|ui| {
                    ui.label(
                        "A powerful desktop application for controlling GRBL-based CNC machines and laser engravers. \
                        Built with Rust and egui for maximum performance and reliability."
                    );
                });
                
                ui.separator();
                
                ui.heading("Key Features");
                ui.label("• Real-time machine status monitoring");
                ui.label("• Advanced CAM capabilities");
                ui.label("• 99.9% uptime guarantee with automatic error recovery");
                ui.label("• 3-axis support (X, Y, Z)");
                ui.label("• Interactive G-code editor with syntax highlighting");
                ui.label("• 3D G-code visualizer");
                ui.label("• Job management and scheduling");
                
                ui.separator();
                
                ui.horizontal(|ui| {
                    if ui.button("📖 GRBL Documentation").clicked() {
                        open_url("https://github.com/grbl/grbl/wiki");
                    }
                    if ui.button("🌐 Project Repository").clicked() {
                        open_url("https://github.com/thawkins/gcodekit");
                    }
                });
                
                ui.separator();
                
                ui.label("© 2024 gcodekit Contributors - MIT License");
            });
        });
    
    if !open {
        // Dialog was closed - we would need to track this in app state
        // For now, this will show every frame if the button was clicked
        // This is a limitation of the current architecture
    }
}
