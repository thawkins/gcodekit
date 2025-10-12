use chrono::Utc;
use eframe::egui;

mod communication;
mod widgets;

use communication::grbl::GrblResponse;

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
    parsed_paths: Vec<PathSegment>,
    current_position: (f32, f32, f32),
    selected_line: Option<usize>,
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

#[derive(Default, PartialEq, Debug)]
enum Tab {
    #[default]
    GcodeEditor,
    Visualizer3D,
    DeviceConsole,
}

#[derive(Default, PartialEq, Debug)]
enum MachineMode {
    #[default]
    CNC,
    Laser,
}

#[derive(Clone, Debug)]
struct PathSegment {
    start: (f32, f32, f32),
    end: (f32, f32, f32),
    move_type: MoveType,
    line_number: usize,
}

#[derive(Clone, Debug, PartialEq)]
enum MoveType {
    Rapid,
    Feed,
    Arc,
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
                    self.gcode_filename = path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    self.parse_gcode();
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
        self.communication
            .send_spindle_override(self.spindle_override);
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
        self.console_messages
            .push(format!("[{}] {}", timestamp, message));

        // Keep only last 1000 messages
        if self.console_messages.len() > 1000 {
            self.console_messages.remove(0);
        }
    }

    fn parse_gcode(&mut self) {
        self.parsed_paths.clear();
        let mut current_pos = (0.0f32, 0.0f32, 0.0f32);
        let mut current_move_type = MoveType::Rapid;

        for (line_idx, line) in self.gcode_content.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with(';') {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            let mut new_pos = current_pos;
            let mut move_type = current_move_type.clone();

            for part in parts {
                if part.starts_with('G') {
                    if let Ok(code) = part[1..].parse::<u32>() {
                        match code {
                            0 => move_type = MoveType::Rapid,
                            1 => move_type = MoveType::Feed,
                            2 | 3 => move_type = MoveType::Arc,
                            _ => {}
                        }
                    }
                } else if part.starts_with('X') {
                    if let Ok(x) = part[1..].parse::<f32>() {
                        new_pos.0 = x;
                    }
                } else if part.starts_with('Y') {
                    if let Ok(y) = part[1..].parse::<f32>() {
                        new_pos.1 = y;
                    }
                } else if part.starts_with('Z') {
                    if let Ok(z) = part[1..].parse::<f32>() {
                        new_pos.2 = z;
                    }
                }
            }

            if new_pos != current_pos {
                self.parsed_paths.push(PathSegment {
                    start: current_pos,
                    end: new_pos,
                    move_type: move_type.clone(),
                    line_number: line_idx,
                });
                current_pos = new_pos;
            }
            current_move_type = move_type;
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
            self.shape_width,
            self.tool_feed_rate,
            self.shape_width,
            self.shape_height,
            self.tool_feed_rate,
            self.shape_height,
            self.tool_feed_rate,
            self.tool_feed_rate
        );
        self.gcode_content = gcode;
        self.gcode_filename = "generated_rectangle.gcode".to_string();
        self.parse_gcode();
        self.status_message = "Rectangle G-code generated".to_string();
    }

    fn generate_circle(&mut self) {
        let gcode = format!(
            "G21 ; Set units to mm\n\
             G90 ; Absolute positioning\n\
             G0 X{} Y{} ; Go to circle center\n\
             G2 I-{} J-{} F{} ; Clockwise circle\n\
             M30 ; End program\n",
            self.shape_radius,
            self.shape_radius,
            self.shape_radius,
            self.shape_radius,
            self.tool_feed_rate
        );
        self.gcode_content = gcode;
        self.gcode_filename = "generated_circle.gcode".to_string();
        self.parse_gcode();
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
            self.parse_gcode();
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
                    self.gcode_content = format!(
                        "; Imported from: {}\n; TODO: Convert to G-code\n{}",
                        path.display(),
                        content
                    );
                    self.gcode_filename = path
                        .file_name()
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
                    self.gcode_filename = path
                        .file_name()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gcode_app_initialization() {
        let app = GcodeKitApp::default();

        assert_eq!(app.selected_tab, Tab::GcodeEditor);
        assert!(app.gcode_content.is_empty());
        assert!(app.gcode_filename.is_empty());
        assert_eq!(app.jog_step_size, 0.0); // Default f32 is 0.0
        assert_eq!(app.spindle_override, 0.0);
        assert_eq!(app.feed_override, 0.0);
        assert_eq!(app.machine_mode, MachineMode::CNC);
        assert!(app.console_messages.is_empty());
        assert_eq!(app.status_message, String::new());
    }

    #[test]
    fn test_generate_rectangle_gcode() {
        let mut app = GcodeKitApp::default();
        app.shape_width = 100.0;
        app.shape_height = 50.0;
        app.tool_feed_rate = 500.0;

        app.generate_rectangle();

        assert!(app.gcode_content.contains("G21 ; Set units to mm"));
        assert!(app.gcode_content.contains("G90 ; Absolute positioning"));
        assert!(app.gcode_content.contains("G0 X0 Y0 ; Go to origin"));
        assert!(app.gcode_content.contains("G1 X100 Y0 F500 ; Bottom edge"));
        assert!(app.gcode_content.contains("G1 X100 Y50 F500 ; Right edge"));
        assert!(app.gcode_content.contains("G1 X0 Y50 F500 ; Top edge"));
        assert!(app.gcode_content.contains("G1 X0 Y0 F500 ; Left edge"));
        assert!(app.gcode_content.contains("M30 ; End program"));
        assert_eq!(app.gcode_filename, "generated_rectangle.gcode");
        assert_eq!(app.status_message, "Rectangle G-code generated".to_string());
    }

    #[test]
    fn test_generate_circle_gcode() {
        let mut app = GcodeKitApp::default();
        app.shape_radius = 25.0;
        app.tool_feed_rate = 300.0;

        app.generate_circle();

        assert!(app.gcode_content.contains("G21 ; Set units to mm"));
        assert!(app.gcode_content.contains("G90 ; Absolute positioning"));
        assert!(
            app.gcode_content
                .contains("G0 X25 Y25 ; Go to circle center")
        );
        assert!(
            app.gcode_content
                .contains("G2 I-25 J-25 F300 ; Clockwise circle")
        );
        assert!(app.gcode_content.contains("M30 ; End program"));
        assert_eq!(app.gcode_filename, "generated_circle.gcode");
        assert_eq!(app.status_message, "Circle G-code generated".to_string());
    }

    #[test]
    fn test_generate_toolpath_with_existing_gcode() {
        let mut app = GcodeKitApp::default();
        app.gcode_content = "G1 X10 Y10\nG1 X20 Y20".to_string();
        app.tool_spindle_speed = 1000.0;
        app.tool_feed_rate = 400.0;

        app.generate_toolpath();

        assert!(app.gcode_content.contains("G21 ; Set units to mm"));
        assert!(app.gcode_content.contains("M3 S1000 ; Spindle on"));
        assert!(app.gcode_content.contains("G1 F400 ; Set feed rate"));
        assert!(app.gcode_content.contains("G1 X10 Y10"));
        assert!(app.gcode_content.contains("G1 X20 Y20"));
        assert_eq!(app.status_message, "Toolpath parameters added".to_string());
    }

    #[test]
    fn test_generate_toolpath_without_gcode() {
        let mut app = GcodeKitApp::default();
        // gcode_content is empty by default
        app.tool_spindle_speed = 1000.0;
        app.tool_feed_rate = 400.0;

        app.generate_toolpath();

        assert_eq!(app.status_message, "No G-code to modify".to_string());
        assert!(app.gcode_content.is_empty());
    }

    #[test]
    fn test_log_console_functionality() {
        let mut app = GcodeKitApp::default();

        app.log_console("Test message");

        assert_eq!(app.console_messages.len(), 1);
        assert!(app.console_messages[0].contains("Test message"));
        assert!(app.console_messages[0].contains("[")); // Should contain timestamp
        assert!(app.console_messages[0].contains("]"));
    }

    #[test]
    fn test_console_message_limit() {
        let mut app = GcodeKitApp::default();

        // Add more than 1000 messages
        for i in 0..1010 {
            app.log_console(&format!("Message {}", i));
        }

        // Should only keep the last 1000 messages
        assert_eq!(app.console_messages.len(), 1000);
        assert!(app.console_messages[0].contains("Message 10")); // First message should be removed
        assert!(app.console_messages[999].contains("Message 1009")); // Last message should be kept
    }

    #[test]
    fn test_generate_image_engraving_placeholder() {
        let mut app = GcodeKitApp::default();
        app.image_resolution = 300.0;
        app.image_max_power = 80.0;

        app.generate_image_engraving();

        assert!(app.gcode_content.contains("; Image engraving G-code"));
        assert!(app.gcode_content.contains("; Resolution: 300 dpi"));
        assert!(app.gcode_content.contains("; Max Power: 80%"));
        assert!(
            app.gcode_content
                .contains("; TODO: Implement actual image processing")
        );
        assert!(app.gcode_content.contains("M30 ; End program"));
        assert_eq!(app.gcode_filename, "image_engraving.gcode");
        assert_eq!(
            app.status_message,
            "Image engraving G-code generated (placeholder)".to_string()
        );
    }

    #[test]
    fn test_generate_tabbed_box_placeholder() {
        let mut app = GcodeKitApp::default();
        app.box_length = 100.0;
        app.box_width = 80.0;
        app.box_height = 50.0;
        app.tab_size = 10.0;

        app.generate_tabbed_box();

        assert!(app.gcode_content.contains("; Tabbed box G-code"));
        assert!(app.gcode_content.contains("; Dimensions: 100x80x50mm"));
        assert!(app.gcode_content.contains("; Tab size: 10mm"));
        assert!(
            app.gcode_content
                .contains("; TODO: Implement actual box cutting paths")
        );
        assert!(app.gcode_content.contains("M30 ; End program"));
        assert_eq!(app.gcode_filename, "tabbed_box.gcode");
        assert_eq!(
            app.status_message,
            "Tabbed box G-code generated (placeholder)".to_string()
        );
    }

    #[test]
    fn test_generate_jigsaw_placeholder() {
        let mut app = GcodeKitApp::default();
        app.jigsaw_pieces = 50;
        app.jigsaw_complexity = 3;

        app.generate_jigsaw();

        assert!(app.gcode_content.contains("; Jigsaw puzzle G-code"));
        assert!(app.gcode_content.contains("; Pieces: 50"));
        assert!(app.gcode_content.contains("; Complexity: 3"));
        assert!(
            app.gcode_content
                .contains("; TODO: Implement actual puzzle piece cutting")
        );
        assert!(app.gcode_content.contains("M30 ; End program"));
        assert_eq!(app.gcode_filename, "jigsaw_puzzle.gcode");
        assert_eq!(
            app.status_message,
            "Jigsaw G-code generated (placeholder)".to_string()
        );
    }
}

impl eframe::App for GcodeKitApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Initialize ports on first run
        if self.communication.available_ports.is_empty()
            && self.communication.connection_state == communication::ConnectionState::Disconnected
        {
            self.refresh_ports();
        }

        // Read GRBL responses
        if self.communication.connection_state == communication::ConnectionState::Connected {
            let messages = self.communication.read_grbl_responses();
            for message in messages {
                // Parse GRBL response and handle appropriately
                let response = self.communication.parse_grbl_response(&message);
                match response {
                    GrblResponse::Status(status) => {
                        // Update status display with real-time information
                        self.current_position = (
                            status.work_position.x,
                            status.work_position.y,
                            status.work_position.z,
                        );
                        self.log_console(&format!(
                            "Status: {:?} | Pos: {:.3},{:.3},{:.3}",
                            status.machine_state,
                            status.work_position.x,
                            status.work_position.y,
                            status.work_position.z
                        ));
                    }
                    GrblResponse::Error(err) => {
                        self.log_console(&format!("GRBL Error: {}", err));
                    }
                    GrblResponse::Alarm(alarm) => {
                        self.log_console(&format!("GRBL Alarm: {}", alarm));
                    }
                    GrblResponse::Ok => {
                        // Command acknowledged
                    }
                    _ => {
                        self.log_console(&message);
                    }
                }
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
                    format!("Status: {}", status_text),
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

                        // 2D visualization area
                        let available_size = ui.available_size();
                        let (rect, response) = ui.allocate_exact_size(available_size, egui::Sense::click());

                        if self.gcode_content.is_empty() {
                            ui.centered_and_justified(|ui| {
                                ui.label("Load G-code to visualize toolpath");
                            });
                        } else {
                            // Draw the paths
                            let painter = ui.painter();
                            let bounds = rect;

                            // Find min/max for scaling
                            let mut min_x = f32::INFINITY;
                            let mut max_x = f32::NEG_INFINITY;
                            let mut min_y = f32::INFINITY;
                            let mut max_y = f32::NEG_INFINITY;

                            for segment in &self.parsed_paths {
                                min_x = min_x.min(segment.start.0).min(segment.end.0);
                                max_x = max_x.max(segment.start.0).max(segment.end.0);
                                min_y = min_y.min(segment.start.1).min(segment.end.1);
                                max_y = max_y.max(segment.start.1).max(segment.end.1);
                            }

                            if !self.parsed_paths.is_empty() {
                                let scale_x = bounds.width() / (max_x - min_x).max(1.0);
                                let scale_y = bounds.height() / (max_y - min_y).max(1.0);
                                let scale = scale_x.min(scale_y) * 0.9; // Leave some margin

                                let offset_x = bounds.min.x + (bounds.width() - (max_x - min_x) * scale) / 2.0 - min_x * scale;
                                let offset_y = bounds.min.y + (bounds.height() - (max_y - min_y) * scale) / 2.0 - min_y * scale;

                                for segment in &self.parsed_paths {
                                    let start_pos = egui::pos2(
                                        offset_x + segment.start.0 * scale,
                                        offset_y + segment.start.1 * scale,
                                    );
                                    let end_pos = egui::pos2(
                                        offset_x + segment.end.0 * scale,
                                        offset_y + segment.end.1 * scale,
                                    );

                                    let color = match segment.move_type {
                                        MoveType::Rapid => egui::Color32::BLUE,
                                        MoveType::Feed => egui::Color32::GREEN,
                                        MoveType::Arc => egui::Color32::YELLOW,
                                    };

                                    let is_selected = self.selected_line == Some(segment.line_number);
                                    let stroke_width = if is_selected { 4.0 } else { 2.0 };
                                    let stroke_color = if is_selected { egui::Color32::WHITE } else { color };

                                    painter.line_segment([start_pos, end_pos], egui::Stroke::new(stroke_width, stroke_color));
                                }

                                // Draw current machine position
                                let current_screen_x = offset_x + self.current_position.0 * scale;
                                let current_screen_y = offset_y + self.current_position.1 * scale;
                                painter.circle_filled(egui::pos2(current_screen_x, current_screen_y), 5.0, egui::Color32::RED);

                                // Left-click to select segment
                                if response.clicked_by(egui::PointerButton::Primary) {
                                    if let Some(click_pos) = response.interact_pointer_pos() {
                                        // Find closest segment to click position
                                        let mut closest_segment = None;
                                        let mut min_distance = f32::INFINITY;

                                        for segment in &self.parsed_paths {
                                            let start_screen = egui::pos2(
                                                offset_x + segment.start.0 * scale,
                                                offset_y + segment.start.1 * scale,
                                            );
                                            let end_screen = egui::pos2(
                                                offset_x + segment.end.0 * scale,
                                                offset_y + segment.end.1 * scale,
                                            );

                                            // Distance to line segment (simplified as distance to midpoint)
                                            let mid_x = (start_screen.x + end_screen.x) / 2.0;
                                            let mid_y = (start_screen.y + end_screen.y) / 2.0;
                                            let dx = click_pos.x - mid_x;
                                            let dy = click_pos.y - mid_y;
                                            let distance = (dx * dx + dy * dy).sqrt();

                                            if distance < min_distance && distance < 20.0 { // Within 20 pixels
                                                min_distance = distance;
                                                closest_segment = Some(segment.line_number);
                                            }
                                        }

                                        self.selected_line = closest_segment;
                                        if let Some(line) = self.selected_line {
                                            self.status_message = format!("Selected line {}", line + 1);
                                        }
                                    }
                                }

                                // Right-click to jog
                                if response.clicked_by(egui::PointerButton::Secondary) {
                                    if self.communication.connection_state == communication::ConnectionState::Connected {
                                        if let Some(click_pos) = response.interact_pointer_pos() {
                                            let gcode_x = (click_pos.x - offset_x) / scale;
                                            let gcode_y = (click_pos.y - offset_y) / scale;
                                            let delta_x = gcode_x - self.current_position.0;
                                            let delta_y = gcode_y - self.current_position.1;
                                            self.jog_axis('X', delta_x);
                                            self.jog_axis('Y', delta_y);
                                            self.status_message = format!("Jogging to X:{:.3} Y:{:.3}", gcode_x, gcode_y);
                                        }
                                    } else {
                                        self.status_message = "Not connected - cannot jog".to_string();
                                    }
                                }
                            }

                            ui.label(format!("Segments: {}", self.parsed_paths.len()));
                        }

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
