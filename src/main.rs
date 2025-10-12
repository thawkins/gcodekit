use eframe::egui;
use chrono::Utc;

mod widgets;
mod communication;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("gcodekit"),
        ..Default::default()
    };

    eframe::run_native(
        "gcodekit",
        options,
        Box::new(|_cc| Ok(Box::new(GcodeKitApp::default()))),
    )
}

#[derive(Default)]
struct GcodeKitApp {
    selected_tab: Tab,
    communication: communication::GrblCommunication,
    status_message: String,
    gcode_content: String,
    gcode_filename: String,
    jog_step_size: f32,
    spindle_override: f32,
    feed_override: f32,
    machine_mode: MachineMode,
    console_messages: Vec<String>,
    // CAM parameters
    shape_width: f32,
    shape_height: f32,
    shape_radius: f32,
    tool_feed_rate: f32,
    tool_spindle_speed: f32,
    image_resolution: f32,
    image_max_power: f32,
    box_length: f32,
    box_width: f32,
    box_height: f32,
    tab_size: f32,
    jigsaw_pieces: i32,
    jigsaw_complexity: i32,
}

#[derive(Default, PartialEq)]
enum Tab {
    #[default]
    GcodeEditor,
    Visualizer3D,
    DeviceConsole,
}

#[derive(Default, PartialEq)]
enum MachineMode {
    #[default]
    CNC,
    Laser,
}

impl GcodeKitApp {
    // Communication wrapper methods
    fn refresh_ports(&mut self) {
        self.communication.refresh_ports();
    }

    fn connect_to_device(&mut self) {
        self.communication.connect_to_device();
        // Log connection messages to console
        if self.communication.connection_state == communication::ConnectionState::Connected {
            self.log_console(&self.communication.status_message.clone());
        } else if self.communication.connection_state == communication::ConnectionState::Error {
            self.log_console(&self.communication.status_message.clone());
        }
    }

    fn disconnect_from_device(&mut self) {
        self.communication.disconnect_from_device();
        self.log_console(&self.communication.status_message.clone());
    }

    fn load_gcode_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("G-code files", &["gcode", "nc", "txt"])
            .pick_file()
        {
            match std::fs::read_to_string(&path) {
                Ok(content) => {
                    self.gcode_content = content;
                    self.gcode_filename = path.file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    self.status_message = format!("Loaded {}", self.gcode_filename);
                }
                Err(e) => {
                    self.status_message = format!("Error loading file: {}", e);
                }
            }
        }
    }

    fn send_gcode_to_device(&mut self) {
        if self.communication.connection_state != communication::ConnectionState::Connected {
            self.status_message = "Not connected to device".to_string();
            return;
        }

        if self.gcode_content.is_empty() {
            self.status_message = "No G-code loaded".to_string();
            return;
        }

        // TODO: Implement actual sending with queuing
        self.status_message = "Sending G-code to device...".to_string();
    }

    fn jog_axis(&mut self, axis: char, distance: f32) {
        self.communication.jog_axis(axis, distance);
        self.status_message = self.communication.status_message.clone();
    }

    fn home_all_axes(&mut self) {
        self.communication.home_all_axes();
        self.status_message = self.communication.status_message.clone();
    }

    fn send_spindle_override(&mut self) {
        self.communication.send_spindle_override(self.spindle_override);
        self.status_message = self.communication.status_message.clone();
    }

    fn send_feed_override(&mut self) {
        self.communication.send_feed_override(self.feed_override);
        self.status_message = self.communication.status_message.clone();
        let message = self.status_message.clone();
        self.log_console(&message);
    }

    fn log_console(&mut self, message: &str) {
        let timestamp = Utc::now().format("%H:%M:%S");
        self.console_messages.push(format!("[{}] {}", timestamp, message));

        // Keep only last 1000 messages
        if self.console_messages.len() > 1000 {
            self.console_messages.remove(0);
        }
    }

    // CAM Functions
    fn generate_rectangle(&mut self) {
        let gcode = format!(
            "G21 ; Set units to mm\n\
             G90 ; Absolute positioning\n\
             G0 X0 Y0 ; Go to origin\n\
             G1 X{} Y0 F{} ; Bottom edge\n\
             G1 X{} Y{} F{} ; Right edge\n\
             G1 X0 Y{} F{} ; Top edge\n\
             G1 X0 Y0 F{} ; Left edge\n\
             M30 ; End program\n",
            self.shape_width, self.tool_feed_rate,
            self.shape_width, self.shape_height, self.tool_feed_rate,
            self.shape_height, self.tool_feed_rate,
            self.tool_feed_rate
        );
        self.gcode_content = gcode;
        self.gcode_filename = "generated_rectangle.gcode".to_string();
        self.status_message = "Rectangle G-code generated".to_string();
    }

    fn generate_circle(&mut self) {
        let gcode = format!(
            "G21 ; Set units to mm\n\
             G90 ; Absolute positioning\n\
             G0 X{} Y{} ; Go to circle center\n\
             G2 I-{} J-{} F{} ; Clockwise circle\n\
             M30 ; End program\n",
            self.shape_radius, self.shape_radius,
            self.shape_radius, self.shape_radius,
            self.tool_feed_rate
        );
        self.gcode_content = gcode;
        self.gcode_filename = "generated_circle.gcode".to_string();
        self.status_message = "Circle G-code generated".to_string();
    }

    fn generate_toolpath(&mut self) {
        // For now, just add toolpath parameters to existing G-code
        if !self.gcode_content.is_empty() {
            let header = format!(
                "G21 ; Set units to mm\n\
                 M3 S{} ; Spindle on\n\
                 G1 F{} ; Set feed rate\n",
                self.tool_spindle_speed, self.tool_feed_rate
            );
            self.gcode_content = header + &self.gcode_content;
            self.status_message = "Toolpath parameters added".to_string();
        } else {
            self.status_message = "No G-code to modify".to_string();
        }
    }

    fn import_vector_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Vector files", &["svg", "dxf"])
            .pick_file()
        {
            // TODO: Implement actual SVG/DXF parsing
            // For now, just load as text
            match std::fs::read_to_string(&path) {
                Ok(content) => {
                    self.gcode_content = format!("; Imported from: {}\n; TODO: Convert to G-code\n{}", path.display(), content);
                    self.gcode_filename = path.file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    self.status_message = format!("Vector file loaded: {}", self.gcode_filename);
                }
                Err(e) => {
                    self.status_message = format!("Error loading vector file: {}", e);
                }
            }
        }
    }

    fn save_gcode_file(&mut self) {
        if self.gcode_content.is_empty() {
            self.status_message = "No G-code to save".to_string();
            return;
        }

        if let Some(path) = rfd::FileDialog::new()
            .add_filter("G-code files", &["gcode", "nc", "txt"])
            .set_file_name(&self.gcode_filename)
            .save_file()
        {
            match std::fs::write(&path, &self.gcode_content) {
                Ok(_) => {
                    self.gcode_filename = path.file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    self.status_message = format!("G-code saved: {}", self.gcode_filename);
                }
                Err(e) => {
                    self.status_message = format!("Error saving file: {}", e);
                }
            }
        }
    }

    fn load_image_for_engraving(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Image files", &["png", "jpg", "jpeg", "bmp"])
            .pick_file()
        {
            // TODO: Implement actual image processing
            self.status_message = format!("Image loaded for engraving: {}", path.display());
        }
    }

    fn generate_image_engraving(&mut self) {
        // TODO: Implement image to G-code conversion
        let gcode = format!(
            "; Image engraving G-code\n\
             ; Resolution: {} dpi\n\
             ; Max Power: {}%\n\
             ; TODO: Implement actual image processing\n\
             M30 ; End program\n",
            self.image_resolution, self.image_max_power
        );
        self.gcode_content = gcode;
        self.gcode_filename = "image_engraving.gcode".to_string();
        self.status_message = "Image engraving G-code generated (placeholder)".to_string();
    }

    fn generate_tabbed_box(&mut self) {
        // TODO: Implement actual tabbed box generation
        let gcode = format!(
            "; Tabbed box G-code\n\
             ; Dimensions: {}x{}x{}mm\n\
             ; Tab size: {}mm\n\
             ; TODO: Implement actual box cutting paths\n\
             M30 ; End program\n",
            self.box_length, self.box_width, self.box_height, self.tab_size
        );
        self.gcode_content = gcode;
        self.gcode_filename = "tabbed_box.gcode".to_string();
        self.status_message = "Tabbed box G-code generated (placeholder)".to_string();
    }

    fn generate_jigsaw(&mut self) {
        // TODO: Implement actual jigsaw generation
        let gcode = format!(
            "; Jigsaw puzzle G-code\n\
             ; Pieces: {}\n\
             ; Complexity: {}\n\
             ; TODO: Implement actual puzzle piece cutting\n\
             M30 ; End program\n",
            self.jigsaw_pieces, self.jigsaw_complexity
        );
        self.gcode_content = gcode;
        self.gcode_filename = "jigsaw_puzzle.gcode".to_string();
        self.status_message = "Jigsaw G-code generated (placeholder)".to_string();
    }
}

impl eframe::App for GcodeKitApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Initialize ports on first run
        if self.communication.available_ports.is_empty() && self.communication.connection_state == communication::ConnectionState::Disconnected {
            self.refresh_ports();
        }

        // Read GRBL responses
        if self.communication.connection_state == communication::ConnectionState::Connected {
            let messages = self.communication.read_grbl_responses();
            for message in messages {
                self.log_console(&message);
            }
        }

        // Top menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open G-code...").clicked() {
                        self.load_gcode_file();
                    }
                    if ui.button("Save G-code...").clicked() {
                        self.save_gcode_file();
                    }
                    ui.separator();
                    if ui.button("Import Vector...").clicked() {
                        self.import_vector_file();
                    }
                    if ui.button("Export G-code...").clicked() {
                        self.save_gcode_file();
                    }
                    ui.separator();
                    if ui.button("Exit").clicked() {
                        std::process::exit(0);
                    }
                });
                ui.menu_button("Machine", |ui| {
                    if ui.button("Connect").clicked() {
                        self.connect_to_device();
                    }
                    if ui.button("Disconnect").clicked() {
                        self.disconnect_from_device();
                    }
                    ui.separator();
                    if ui.button("Home All").clicked() {
                        self.home_all_axes();
                    }
                    if ui.button("Reset").clicked() {
                        // TODO: Reset machine
                    }
                });
                ui.menu_button("View", |ui| {
                    if ui.button("G-code Editor").clicked() {
                        self.selected_tab = Tab::GcodeEditor;
                    }
                    if ui.button("3D Visualizer").clicked() {
                        self.selected_tab = Tab::Visualizer3D;
                    }
                    if ui.button("Device Console").clicked() {
                        self.selected_tab = Tab::DeviceConsole;
                    }
                    ui.separator();
                    if ui.button("Refresh Ports").clicked() {
                        self.refresh_ports();
                    }
                });
                ui.menu_button("Tools", |ui| {
                    if ui.button("Generate Rectangle").clicked() {
                        self.generate_rectangle();
                    }
                    if ui.button("Generate Circle").clicked() {
                        self.generate_circle();
                    }
                    ui.separator();
                    if ui.button("Image Engraving").clicked() {
                        self.load_image_for_engraving();
                    }
                    if ui.button("Tabbed Box").clicked() {
                        self.generate_tabbed_box();
                    }
                    if ui.button("Jigsaw Puzzle").clicked() {
                        self.generate_jigsaw();
                    }
                });
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

        // Bottom status bar
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Connection status
                let status_text = match self.communication.connection_state {
                    communication::ConnectionState::Disconnected => "Disconnected",
                    communication::ConnectionState::Connecting => "Connecting...",
                    communication::ConnectionState::Connected => "Connected",
                    communication::ConnectionState::Error => "Error",
                };
                ui.colored_label(
                    match self.communication.connection_state {
                        communication::ConnectionState::Connected => egui::Color32::GREEN,
                        communication::ConnectionState::Error => egui::Color32::RED,
                        communication::ConnectionState::Connecting => egui::Color32::YELLOW,
                        _ => egui::Color32::GRAY,
                    },
                    format!("Status: {}", status_text)
                );

                ui.separator();

                // Device state (locked/alarmed)
                ui.label("State: Idle"); // TODO: Track actual GRBL state

                ui.separator();

                // Current position
                ui.label("Position: X:0.000 Y:0.000 Z:0.000"); // TODO: Track actual position

                ui.separator();

                // GRBL version
                if !self.communication.grbl_version.is_empty() {
                    ui.label(format!("GRBL: {}", self.communication.grbl_version));
                    ui.separator();
                }

                // Selected port
                if !self.communication.selected_port.is_empty() {
                    ui.label(format!("Port: {}", self.communication.selected_port));
                }

                // Version info on the right
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label("gcodekit v0.1.0");
                });
            });
        });

        // Left panel - Machine control
        egui::SidePanel::left("left_panel")
            .resizable(true)
            .default_width(200.0)
            .show(ctx, |ui| {
                ui.heading("Machine Control");
                ui.separator();

                widgets::show_connection_widget(ui, &mut self.communication);
                ui.separator();
                widgets::show_gcode_loading_widget(ui, self);
                ui.separator();
                widgets::show_jog_widget(ui, self);
                ui.separator();
                widgets::show_overrides_widget(ui, self);
            });

        // Right panel - CAM functions
        egui::SidePanel::right("right_panel")
            .resizable(true)
            .default_width(250.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.heading("CAM Functions");

                    widgets::show_shape_generation_widget(ui, self);
                    ui.separator();
                    widgets::show_toolpath_generation_widget(ui, self);
                    ui.separator();
                    widgets::show_vector_import_widget(ui, self);
                    ui.separator();
                    widgets::show_image_engraving_widget(ui, self);
                    ui.separator();
                    widgets::show_tabbed_box_widget(ui, self);
                    ui.separator();
                    widgets::show_jigsaw_widget(ui, self);
                });
            });

        // Central panel with tabs
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.selected_tab, Tab::GcodeEditor, "G-code Editor");
                ui.selectable_value(&mut self.selected_tab, Tab::Visualizer3D, "3D Visualizer");
                ui.selectable_value(&mut self.selected_tab, Tab::DeviceConsole, "Device Console");
            });
            ui.separator();

            match self.selected_tab {
                Tab::GcodeEditor => {
                    if self.gcode_content.is_empty() {
                        ui.centered_and_justified(|ui| {
                            ui.label("No G-code file loaded. Use 'Load File' in the left panel.");
                        });
                    } else {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            ui.add(
                                egui::TextEdit::multiline(&mut self.gcode_content)
                                    .font(egui::TextStyle::Monospace)
                                    .desired_rows(20)
                            );
                        });
                    }
                }
                Tab::Visualizer3D => {
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label("3D Visualizer");
                            ui.separator();
                            if ui.button("🔄 Refresh View").clicked() {
                                // TODO: Refresh visualization
                            }
                            if ui.button("📏 Fit to View").clicked() {
                                // TODO: Fit view to content
                            }
                        });

                        ui.separator();

                        // Basic 2D visualization area (placeholder for 3D)
                        let available_size = ui.available_size();
                        let (_rect, _) = ui.allocate_exact_size(available_size, egui::Sense::hover());

                        // Basic visualization placeholder
                        ui.centered_and_justified(|ui| {
                            ui.label("3D Visualizer");
                            ui.separator();
                            if self.gcode_content.is_empty() {
                                ui.label("Load G-code to visualize toolpath");
                            } else {
                                ui.label("G-code loaded - 3D visualization coming soon");
                                ui.label(format!("Lines: {}", self.gcode_content.lines().count()));
                            }
                        });

                        ui.separator();
                        ui.label("Note: Full 3D visualization requires additional 3D rendering libraries.");
                    });
                }
                Tab::DeviceConsole => {
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label("Device Console");
                            ui.separator();
                            if ui.button("🗑️ Clear").clicked() {
                                self.console_messages.clear();
                            }
                        });

                        ui.separator();

                        egui::ScrollArea::vertical()
                            .auto_shrink([false, false])
                            .stick_to_bottom(true)
                            .show(ui, |ui| {
                                for message in &self.console_messages {
                                    ui.label(message);
                                }
                                if self.console_messages.is_empty() {
                                    ui.weak("No messages yet. Connect to a device to see communication logs.");
                                }
                            });
                    });
                }
            }
        });
    }
}
