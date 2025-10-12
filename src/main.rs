use chrono::Utc;
use eframe::egui;
use egui::text::{LayoutJob, TextFormat};
use egui::text_edit::TextBuffer;
use std::collections::HashMap;
use std::time::Duration;

mod communication;
mod designer;
mod jobs;
mod materials;
mod widgets;

use designer::Tool;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Action {
    OpenFile,
    SaveFile,
    ExportGcode,
    ImportVector,
    Undo,
    Redo,
    ZoomIn,
    ZoomOut,
    Home,
    JogXPlus,
    JogXMinus,
    JogYPlus,
    JogYMinus,
    JogZPlus,
    JogZMinus,
    ProbeZ,
    FeedHold,
    Resume,
    Reset,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct KeyBinding {
    pub key: egui::Key,
    pub modifiers: egui::Modifiers,
}

#[derive(Clone, Debug, Default)]
pub struct MachinePosition {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub a: Option<f32>,
    pub b: Option<f32>,
    pub c: Option<f32>,
    pub d: Option<f32>,
}

impl MachinePosition {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            x,
            y,
            z,
            a: None,
            b: None,
            c: None,
            d: None,
        }
    }

    pub fn with_a(mut self, a: f32) -> Self {
        self.a = Some(a);
        self
    }

    pub fn with_b(mut self, b: f32) -> Self {
        self.b = Some(b);
        self
    }

    pub fn with_c(mut self, c: f32) -> Self {
        self.c = Some(c);
        self
    }

    pub fn with_d(mut self, d: f32) -> Self {
        self.d = Some(d);
        self
    }

    pub fn get_axis(&self, axis: char) -> Option<f32> {
        match axis {
            'X' | 'x' => Some(self.x),
            'Y' | 'y' => Some(self.y),
            'Z' | 'z' => Some(self.z),
            'A' | 'a' => self.a,
            'B' | 'b' => self.b,
            'C' | 'c' => self.c,
            'D' | 'd' => self.d,
            _ => None,
        }
    }

    pub fn set_axis(&mut self, axis: char, value: f32) {
        match axis {
            'X' | 'x' => self.x = value,
            'Y' | 'y' => self.y = value,
            'Z' | 'z' => self.z = value,
            'A' | 'a' => self.a = Some(value),
            'B' | 'b' => self.b = Some(value),
            'C' | 'c' => self.c = Some(value),
            'D' | 'd' => self.d = Some(value),
            _ => {}
        }
    }

    pub fn format_position(&self) -> String {
        let mut parts = vec![
            format!("X:{:.3}", self.x),
            format!("Y:{:.3}", self.y),
            format!("Z:{:.3}", self.z),
        ];

        if let Some(a) = self.a {
            parts.push(format!("A:{:.3}", a));
        }
        if let Some(b) = self.b {
            parts.push(format!("B:{:.3}", b));
        }
        if let Some(c) = self.c {
            parts.push(format!("C:{:.3}", c));
        }
        if let Some(d) = self.d {
            parts.push(format!("D:{:.3}", d));
        }

        parts.join(" ")
    }
}

use communication::{CncController, ConnectionState, ControllerType};

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

struct GcodeKitApp {
    selected_tab: Tab,
    controller_type: ControllerType,
    communication: Box<dyn CncController>,
    status_message: String,
    gcode_content: String,
    gcode_filename: String,
    jog_step_size: f32,
    spindle_override: f32,
    feed_override: f32,
    machine_mode: MachineMode,
    console_messages: Vec<String>,
    parsed_paths: Vec<PathSegment>,
    current_position: MachinePosition,
    selected_line: Option<usize>,
    sending_from_line: Option<usize>,
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
    current_tool: i32,
    tool_library: Vec<Tool>,
    keybindings: HashMap<Action, KeyBinding>,
    designer: designer::DesignerState,
    script_content: String,
    job_queue: jobs::JobQueue,
    material_database: materials::MaterialDatabase,
    show_job_creation_dialog: bool,
    new_job_name: String,
    new_job_type: jobs::JobType,
    selected_material: Option<String>,
    current_job_id: Option<String>, // Currently running job ID
}

impl Default for GcodeKitApp {
    fn default() -> Self {
        Self {
            selected_tab: Tab::default(),
            controller_type: ControllerType::Grbl,
            communication: Box::new(communication::GrblCommunication::default()),
            status_message: String::new(),
            gcode_content: String::new(),
            gcode_filename: String::new(),
            jog_step_size: 0.0,
            spindle_override: 0.0,
            feed_override: 0.0,
            machine_mode: MachineMode::default(),
            console_messages: Vec::new(),
            parsed_paths: Vec::new(),
            current_position: MachinePosition::new(0.0, 0.0, 0.0),
            selected_line: None,
            sending_from_line: None,
            shape_width: 0.0,
            shape_height: 0.0,
            shape_radius: 0.0,
            tool_feed_rate: 0.0,
            tool_spindle_speed: 0.0,
            image_resolution: 0.0,
            image_max_power: 0.0,
            box_length: 0.0,
            box_width: 0.0,
            box_height: 0.0,
            tab_size: 0.0,
            jigsaw_pieces: 0,
            jigsaw_complexity: 0,
            current_tool: 0,
            tool_library: vec![
                Tool {
                    name: "End Mill 3mm".to_string(),
                    diameter: 3.0,
                    material: "HSS".to_string(),
                    flute_count: 2,
                    max_rpm: 10000,
                },
                Tool {
                    name: "Drill 2mm".to_string(),
                    diameter: 2.0,
                    material: "HSS".to_string(),
                    flute_count: 1,
                    max_rpm: 5000,
                },
            ],
            keybindings: {
                let mut map = HashMap::new();
                map.insert(Action::OpenFile, KeyBinding { key: egui::Key::O, modifiers: egui::Modifiers::CTRL });
                map.insert(Action::SaveFile, KeyBinding { key: egui::Key::S, modifiers: egui::Modifiers::CTRL });
                map.insert(Action::ExportGcode, KeyBinding { key: egui::Key::E, modifiers: egui::Modifiers::CTRL });
                map.insert(Action::ImportVector, KeyBinding { key: egui::Key::I, modifiers: egui::Modifiers::CTRL });
                map.insert(Action::Undo, KeyBinding { key: egui::Key::Z, modifiers: egui::Modifiers::CTRL });
                map.insert(Action::Redo, KeyBinding { key: egui::Key::Y, modifiers: egui::Modifiers::CTRL });
                map.insert(Action::ZoomIn, KeyBinding { key: egui::Key::Plus, modifiers: egui::Modifiers::CTRL });
                map.insert(Action::ZoomOut, KeyBinding { key: egui::Key::Minus, modifiers: egui::Modifiers::CTRL });
                map.insert(Action::Home, KeyBinding { key: egui::Key::H, modifiers: egui::Modifiers::ALT });
                map.insert(Action::JogXPlus, KeyBinding { key: egui::Key::ArrowRight, modifiers: egui::Modifiers::SHIFT });
                map.insert(Action::JogXMinus, KeyBinding { key: egui::Key::ArrowLeft, modifiers: egui::Modifiers::SHIFT });
                map.insert(Action::JogYPlus, KeyBinding { key: egui::Key::ArrowUp, modifiers: egui::Modifiers::SHIFT });
                map.insert(Action::JogYMinus, KeyBinding { key: egui::Key::ArrowDown, modifiers: egui::Modifiers::SHIFT });
                map.insert(Action::JogZPlus, KeyBinding { key: egui::Key::PageUp, modifiers: egui::Modifiers::SHIFT });
                map.insert(Action::JogZMinus, KeyBinding { key: egui::Key::PageDown, modifiers: egui::Modifiers::SHIFT });
                map.insert(Action::ProbeZ, KeyBinding { key: egui::Key::P, modifiers: egui::Modifiers::ALT });
                map.insert(Action::FeedHold, KeyBinding { key: egui::Key::Space, modifiers: egui::Modifiers::NONE });
                map.insert(Action::Resume, KeyBinding { key: egui::Key::Space, modifiers: egui::Modifiers::SHIFT });
                map.insert(Action::Reset, KeyBinding { key: egui::Key::R, modifiers: egui::Modifiers::ALT });
                map
            },
            designer: designer::DesignerState::default(),
            script_content: String::new(),
            job_queue: jobs::JobQueue::default(),
            material_database: materials::MaterialDatabase::default(),
            show_job_creation_dialog: false,
            new_job_name: String::new(),
            new_job_type: jobs::JobType::GcodeFile,
            selected_material: None,
            current_job_id: None,
        }
    }
}

#[derive(Default, PartialEq, Debug)]
enum Tab {
    #[default]
    Designer,
    GcodeEditor,
    Visualizer3D,
    DeviceConsole,
    JobManager,
    Scripting,
}

#[derive(Clone, Debug, PartialEq, Default)]
enum MoveType {
    #[default]
    Rapid,
    Feed,
    Arc,
}

#[derive(Clone, Debug, Default)]
struct PathSegment {
    start: MachinePosition,
    end: MachinePosition,
    move_type: MoveType,
    line_number: usize,
}

#[derive(Clone, Copy, PartialEq, Debug, Default)]
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
        if let Err(e) = self.communication.connect() {
            let msg = format!("Connection error: {}", e);
            self.status_message = msg.clone();
            self.log_console(&msg);
        } else {
            let msg = "Connected successfully".to_string();
            self.status_message = msg.clone();
            self.log_console(&msg);
        }
    }

    fn disconnect_from_device(&mut self) {
        self.communication.disconnect();
        self.sending_from_line = None; // Clear sending indicator
        let msg = self.communication.get_status_message().to_string();
        self.log_console(&msg);
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
                    self.sending_from_line = None; // Clear sending indicator
                    self.status_message = format!("Loaded {}", self.gcode_filename);
                }
                Err(e) => {
                    self.status_message = format!("Error loading file: {}", e);
                }
            }
        }
    }

    fn send_gcode_to_device(&mut self) {
        if !self.communication.is_connected() {
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

    fn send_gcode_from_line(&mut self, start_line: usize) {
        if !self.communication.is_connected() {
            self.status_message = "Not connected to device".to_string();
            return;
        }

        if self.gcode_content.is_empty() {
            self.status_message = "No G-code loaded".to_string();
            return;
        }

        let lines: Vec<String> = self.gcode_content.lines().map(|s| s.to_string()).collect();
        if start_line >= lines.len() {
            self.status_message = "Invalid line number".to_string();
            return;
        }

        let lines_to_send = &lines[start_line..];
        let mut sent_count = 0;

        for (i, line) in lines_to_send.iter().enumerate() {
            let trimmed = line.trim();
            if !trimmed.is_empty() && !trimmed.starts_with(';') {
                match self.communication.send_gcode_line(trimmed) {
                    Ok(_) => {
                        sent_count += 1;
                        // Update job progress
                        if let Some(job_id) = &self.current_job_id
                            && let Some(job) = self.job_queue.get_job_mut(job_id)
                        {
                            let current_line = start_line + i;
                            job.last_completed_line = Some(current_line);
                            job.update_progress((current_line as f32) / (lines.len() as f32));
                        }
                    }
                    Err(e) => {
                        let error_msg = format!("Error sending line: {}", e);
                        // Interrupt current job on error
                        if let Some(job_id) = &self.current_job_id {
                            if let Some(job) = self.job_queue.get_job_mut(job_id) {
                                let failed_line = start_line + i;
                                job.interrupt(failed_line);
                                self.log_console(&format!(
                                    "Job {} interrupted at line {}",
                                    job_id,
                                    failed_line + 1
                                ));
                            }
                            self.current_job_id = None;
                        }
                        self.handle_communication_error(&error_msg);
                        // Continue with next line if recovery was attempted
                        if self.communication.is_recovering() {
                            continue;
                        } else {
                            self.status_message = error_msg;
                            return;
                        }
                    }
                }
            }
        }

        self.sending_from_line = Some(start_line);
        self.status_message = format!(
            "Sent {} G-code lines from line {}",
            sent_count,
            start_line + 1
        );
        self.log_console(&format!(
            "Sent {} lines starting from line {}",
            sent_count,
            start_line + 1
        ));
    }

    fn optimize_gcode(&mut self) {
        if self.gcode_content.is_empty() {
            self.status_message = "No G-code to optimize".to_string();
            return;
        }

        let original_lines = self.gcode_content.lines().count();
        let mut optimized_lines = Vec::new();

        for line in self.gcode_content.lines() {
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with(';') {
                continue;
            }

            // Remove inline comments
            let line = if let Some(comment_pos) = line.find(';') {
                line[..comment_pos].trim()
            } else {
                line
            };

            if line.is_empty() {
                continue;
            }

            // For now, just keep the line as-is (decimal truncation would be more complex)
            optimized_lines.push(line.to_string());
        }

        self.gcode_content = optimized_lines.join("\n");
        self.parse_gcode(); // Re-parse the optimized G-code

        let optimized_line_count = optimized_lines.len();
        self.status_message = format!(
            "G-code optimized: {} -> {} lines",
            original_lines, optimized_line_count
        );
        self.log_console(&format!(
            "Optimized G-code: removed {} lines",
            original_lines - optimized_line_count
        ));
    }

    fn jog_axis(&mut self, axis: char, distance: f32) {
        self.communication.jog_axis(axis, distance);
        self.status_message = self.communication.get_status_message().to_string();
    }

    fn home_all_axes(&mut self) {
        self.communication.home_all_axes();
        self.status_message = self.communication.get_status_message().to_string();
    }

    fn send_spindle_override(&mut self) {
        self.communication
            .send_spindle_override(self.spindle_override);
        self.status_message = self.communication.get_status_message().to_string();
    }

    fn send_feed_override(&mut self) {
        self.communication.send_feed_override(self.feed_override);
        self.status_message = self.communication.get_status_message().to_string();
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
        let mut current_pos = MachinePosition::new(0.0, 0.0, 0.0);
        let mut current_move_type = MoveType::Rapid;

        for (line_idx, line) in self.gcode_content.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with(';') {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            let mut new_pos = current_pos.clone();
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
                } else if part.len() > 1 {
                    let axis = part.chars().next().unwrap();
                    if let Ok(value) = part[1..].parse::<f32>() {
                        new_pos.set_axis(axis, value);
                    }
                }
            }

            // Check if position changed
            let position_changed = new_pos.x != current_pos.x
                || new_pos.y != current_pos.y
                || new_pos.z != current_pos.z
                || new_pos.a != current_pos.a
                || new_pos.b != current_pos.b
                || new_pos.c != current_pos.c
                || new_pos.d != current_pos.d;

            if position_changed {
                self.parsed_paths.push(PathSegment {
                    start: current_pos,
                    end: new_pos.clone(),
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
            self.gcode_content = format!("{}{}", header, self.gcode_content);
            self.parse_gcode();
            self.status_message = "Toolpath parameters added".to_string();
        } else {
            self.status_message = "No G-code to modify".to_string();
        }
    }

    fn handle_keyboard_shortcuts(&mut self, ctx: &egui::Context) {
        for (action, binding) in &self.keybindings.clone() {
            if ctx.input(|i| i.key_pressed(binding.key) && i.modifiers == binding.modifiers) {
                match action {
                    Action::OpenFile => {
                        self.load_gcode_file();
                    }
                    Action::SaveFile => {
                        self.save_gcode_file();
                    }
                    Action::ExportGcode => {
                        self.export_design_to_gcode();
                    }
                    Action::ImportVector => {
                        self.import_vector_file();
                    }
                    Action::Undo => {
                        self.designer.undo();
                    }
                    Action::Redo => {
                        self.designer.redo();
                    }
                    Action::ZoomIn => {
                        // TODO: Implement zoom
                    }
                    Action::ZoomOut => {
                        // TODO: Implement zoom
                    }
                    Action::Home => {
                        self.send_gcode("G28");
                    }
                    Action::JogXPlus => {
                        self.send_gcode("G91 G0 X10 F1000");
                    }
                    Action::JogXMinus => {
                        self.send_gcode("G91 G0 X-10 F1000");
                    }
                    Action::JogYPlus => {
                        self.send_gcode("G91 G0 Y10 F1000");
                    }
                    Action::JogYMinus => {
                        self.send_gcode("G91 G0 Y-10 F1000");
                    }
                    Action::JogZPlus => {
                        self.send_gcode("G91 G0 Z10 F1000");
                    }
                    Action::JogZMinus => {
                        self.send_gcode("G91 G0 Z-10 F1000");
                    }
                    Action::ProbeZ => {
                        self.send_gcode("G38.2 Z-10 F50");
                    }
                    Action::FeedHold => {
                        self.send_gcode("!");
                    }
                    Action::Resume => {
                        self.send_gcode("~");
                    }
                    Action::Reset => {
                        self.send_gcode("\x18");
                    }
                }
            }
        }
    }

    fn import_vector_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Vector files", &["svg", "dxf"])
            .pick_file()
        {
            let result = if let Some(ext) = path.extension() {
                match ext.to_str().unwrap_or("").to_lowercase().as_str() {
                    "svg" => self.designer.import_svg(&path),
                    "dxf" => self.designer.import_dxf(&path),
                    _ => Err(anyhow::anyhow!("Unsupported file format")),
                }
            } else {
                Err(anyhow::anyhow!("No file extension"))
            };

            match result {
                Ok(()) => {
                    tracing::info!("Successfully imported vector file: {}", path.display());
                    // Optionally export to G-code immediately
                    self.gcode_content = self.designer.export_to_gcode();
                    self.gcode_filename = path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                }
                Err(e) => {
                    tracing::error!("Failed to import vector file: {}", e);
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

    fn send_gcode(&mut self, gcode: &str) {
        // TODO: Implement sending G-code string
        self.status_message = format!("Sending G-code: {}", gcode);
    }

    fn export_design_to_gcode(&mut self) {
        // TODO: Implement design export
        self.status_message = "Exporting design to G-code...".to_string();
    }

    fn import_design_file(&mut self) {
        // TODO: Implement design import
        self.status_message = "Importing design file...".to_string();
    }

    fn run_script(&mut self) {
        // TODO: Implement script running
        self.status_message = "Running script...".to_string();
    }

    fn handle_communication_error(&mut self, error: &str) {
        let timestamp = Utc::now().format("%H:%M:%S");
        println!("[{}] [ERROR] Communication error: {}", timestamp, error);
        self.log_console(&format!("Communication error: {}", error));

        // Attempt recovery
        match self.communication.attempt_recovery(error) {
            Ok(action) => {
                let action_msg = match action {
                    communication::RecoveryAction::Reconnect => {
                        self.log_console("Attempting to reconnect...");
                        println!("[{}] [RECOVERY] Scheduled reconnection attempt", timestamp);
                        "Attempting recovery - reconnecting...".to_string()
                    }
                    communication::RecoveryAction::RetryCommand => {
                        self.log_console("Retrying last command...");
                        println!("[{}] [RECOVERY] Retrying last command", timestamp);
                        "Retrying command...".to_string()
                    }
                    communication::RecoveryAction::ResetController => {
                        self.log_console("Resetting controller...");
                        println!("[{}] [RECOVERY] Resetting controller", timestamp);
                        "Resetting controller...".to_string()
                    }
                    communication::RecoveryAction::SkipCommand => {
                        self.log_console("Skipping failed command...");
                        println!("[{}] [RECOVERY] Skipping failed command", timestamp);
                        "Skipping failed command".to_string()
                    }
                    communication::RecoveryAction::AbortJob => {
                        self.log_console("Aborting current job due to critical error");
                        println!(
                            "[{}] [RECOVERY] Aborting current job due to critical error",
                            timestamp
                        );
                        // Clear current job if aborting
                        self.current_job_id = None;
                        "Critical error - aborting job".to_string()
                    }
                };
                self.status_message = action_msg;
            }
            Err(recovery_error) => {
                println!(
                    "[{}] [RECOVERY] Recovery failed: {}",
                    timestamp, recovery_error
                );
                self.log_console(&format!("Recovery failed: {}", recovery_error));
                self.status_message = format!("Error recovery failed: {}", recovery_error);
                // Clear current job on recovery failure
                self.current_job_id = None;
            }
        }
    }

    fn create_job_from_generated_gcode(&mut self, name: &str, job_type: jobs::JobType) {
        if !self.gcode_content.is_empty() {
            let mut job = jobs::Job::new(name.to_string(), job_type);
            if let Some(material) = &self.selected_material {
                job = job.with_material(material.clone());
            }
            // For generated G-code, we don't have a file path, so we'll store it as content
            // The job system would need to be extended to handle in-memory G-code
            self.job_queue.add_job(job);
        }
    }

    fn start_job(&mut self, job_id: &str) -> Result<(), String> {
        self.job_queue.start_job(job_id)?;
        self.current_job_id = Some(job_id.to_string());
        self.log_console(&format!("Started job: {}", job_id));
        Ok(())
    }

    fn resume_job(&mut self, job_id: &str) -> Result<(), String> {
        // Get the resume line
        let resume_line = {
            let job = self.job_queue.get_job(job_id).ok_or("Job not found")?;
            job.get_resume_line().ok_or("Job cannot be resumed")?
        };

        // Resume the job
        self.job_queue.resume_job(job_id)?;
        self.current_job_id = Some(job_id.to_string());

        // Start sending from resume line
        self.send_gcode_from_line(resume_line);
        self.log_console(&format!(
            "Resumed job {} from line {}",
            job_id,
            resume_line + 1
        ));
        Ok(())
    }

    fn show_job_manager_tab(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.heading("Job Manager");
            ui.separator();

            // Job queue controls
            ui.horizontal(|ui| {
                if ui.button("➕ Add Job").clicked() {
                    self.show_job_creation_dialog = true;
                    self.new_job_name = "New Job".to_string();
                    self.new_job_type = jobs::JobType::GcodeFile;
                    self.selected_material = None;
                }
                if ui.button("🗑️ Clear Completed").clicked() {
                    self.job_queue.clear_completed_jobs();
                }
                ui.label(format!("Jobs: {}", self.job_queue.jobs.len()));
            });

            // Job creation dialog
            if self.show_job_creation_dialog {
                egui::Window::new("Create New Job")
                    .collapsible(false)
                    .resizable(false)
                    .show(ui.ctx(), |ui| {
                        ui.vertical(|ui| {
                            ui.label("Job Name:");
                            ui.text_edit_singleline(&mut self.new_job_name);

                            ui.label("Job Type:");
                            egui::ComboBox::from_label("")
                                .selected_text(format!("{:?}", self.new_job_type))
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut self.new_job_type,
                                        jobs::JobType::GcodeFile,
                                        "G-code File",
                                    );
                                    ui.selectable_value(
                                        &mut self.new_job_type,
                                        jobs::JobType::CAMOperation,
                                        "CAM Operation",
                                    );
                                    ui.selectable_value(
                                        &mut self.new_job_type,
                                        jobs::JobType::Probing,
                                        "Probing",
                                    );
                                    ui.selectable_value(
                                        &mut self.new_job_type,
                                        jobs::JobType::Calibration,
                                        "Calibration",
                                    );
                                    ui.selectable_value(
                                        &mut self.new_job_type,
                                        jobs::JobType::Maintenance,
                                        "Maintenance",
                                    );
                                });

                            ui.label("Material:");
                            let mut material_names: Vec<String> = self
                                .material_database
                                .get_all_materials()
                                .iter()
                                .map(|m| m.name.clone())
                                .collect();
                            material_names.insert(0, "None".to_string());

                            let current_selection = self
                                .selected_material
                                .as_ref()
                                .unwrap_or(&"None".to_string())
                                .clone();

                            egui::ComboBox::from_label("")
                                .selected_text(&current_selection)
                                .show_ui(ui, |ui| {
                                    for material_name in &material_names {
                                        let is_selected = Some(material_name.clone())
                                            == self.selected_material
                                            || (material_name == "None"
                                                && self.selected_material.is_none());
                                        if ui.selectable_label(is_selected, material_name).clicked()
                                        {
                                            if material_name == "None" {
                                                self.selected_material = None;
                                            } else {
                                                self.selected_material =
                                                    Some(material_name.clone());
                                            }
                                        }
                                    }
                                });

                            ui.separator();
                            ui.horizontal(|ui| {
                                if ui.button("Create").clicked() {
                                    let job_name = self.new_job_name.clone();
                                    let job_type = self.new_job_type.clone();
                                    let selected_material = self.selected_material.clone();
                                    let mut job = jobs::Job::new(job_name, job_type);
                                    if let Some(material) = &selected_material {
                                        job = job.with_material(material.clone());
                                    }
                                    self.job_queue.add_job(job);
                                    self.show_job_creation_dialog = false;
                                }
                                if ui.button("Cancel").clicked() {
                                    self.show_job_creation_dialog = false;
                                }
                            });
                        });
                    });
            }

            ui.separator();

            // Job list
            egui::ScrollArea::vertical().show(ui, |ui| {
                let mut jobs_to_start = Vec::new();
                let mut jobs_to_pause = Vec::new();
                let mut jobs_to_resume = Vec::new();
                let mut jobs_to_resume_interrupted = Vec::new(); // For resuming interrupted jobs
                let mut jobs_to_cancel = Vec::new();
                let mut jobs_to_remove = Vec::new();

                // Clone job data for display to avoid borrow issues
                let jobs_data: Vec<_> = self.job_queue.jobs.iter().cloned().collect();

                for job in &jobs_data {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            // Job status indicator
                            let status_color = match job.status {
                                jobs::JobStatus::Pending => egui::Color32::GRAY,
                                jobs::JobStatus::Running => egui::Color32::GREEN,
                                jobs::JobStatus::Paused => egui::Color32::YELLOW,
                                jobs::JobStatus::Completed => egui::Color32::BLUE,
                                jobs::JobStatus::Failed => egui::Color32::RED,
                                jobs::JobStatus::Cancelled => egui::Color32::ORANGE,
                            };
                            ui.colored_label(status_color, "●");

                            ui.label(&job.name);
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    ui.label(format!("{:.1}%", job.progress * 100.0));

                                    // Control buttons - collect actions instead of executing immediately
                                    match job.status {
                                        jobs::JobStatus::Pending => {
                                            if ui.button("▶️ Start").clicked() {
                                                jobs_to_start.push(job.id.clone());
                                            }
                                        }
                                        jobs::JobStatus::Running => {
                                            if ui.button("⏸️ Pause").clicked() {
                                                jobs_to_pause.push(job.id.clone());
                                            }
                                            if ui.button("⏹️ Stop").clicked() {
                                                jobs_to_cancel.push(job.id.clone());
                                            }
                                        }
                                        jobs::JobStatus::Paused => {
                                            if job.can_resume_job() {
                                                if ui.button("🔄 Resume").clicked() {
                                                    jobs_to_resume_interrupted.push(job.id.clone());
                                                }
                                            } else if ui.button("▶️ Resume").clicked() {
                                                jobs_to_resume.push(job.id.clone());
                                            }
                                        }
                                        _ => {
                                            if ui.button("🗑️ Remove").clicked() {
                                                jobs_to_remove.push(job.id.clone());
                                            }
                                        }
                                    }
                                },
                            );
                        });

                        ui.label(format!(
                            "Type: {:?} | Priority: {}",
                            job.job_type, job.priority
                        ));
                        if let Some(material) = &job.material {
                            ui.label(format!("Material: {}", material));
                        }
                        if let Some(tool) = &job.tool {
                            ui.label(format!("Tool: {}", tool));
                        }

                        // Progress bar
                        let progress_bar = egui::ProgressBar::new(job.progress)
                            .show_percentage()
                            .animate(true);
                        ui.add(progress_bar);

                        // Show timing info
                        if let Some(started) = job.started_at {
                            let duration = if let Some(completed) = job.completed_at {
                                completed.signed_duration_since(started)
                            } else {
                                chrono::Utc::now().signed_duration_since(started)
                            };
                            ui.label(format!("Duration: {:.1}s", duration.num_seconds() as f32));
                        }

                        if let Some(error) = &job.error_message {
                            ui.colored_label(egui::Color32::RED, format!("Error: {}", error));
                        }
                    });
                    ui.separator();
                }

                // Execute collected actions
                for job_id in jobs_to_start {
                    let _ = self.start_job(&job_id);
                }
                for job_id in jobs_to_pause {
                    let _ = self.job_queue.pause_job(&job_id);
                }
                for job_id in jobs_to_resume {
                    let _ = self.job_queue.resume_job(&job_id);
                }
                for job_id in jobs_to_resume_interrupted {
                    let _ = self.resume_job(&job_id);
                }
                for job_id in jobs_to_cancel {
                    let _ = self.job_queue.cancel_job(&job_id);
                }
                for job_id in jobs_to_remove {
                    self.job_queue.remove_job(&job_id);
                }
            });
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gcode_app_initialization() {
        let app = GcodeKitApp::default();

        assert_eq!(app.selected_tab, Tab::Designer);
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
    fn test_job_resumption_integration() {
        let mut app = GcodeKitApp::default();

        // Create a job
        let job = jobs::Job::new("Test Job".to_string(), jobs::JobType::GcodeFile);
        app.job_queue.add_job(job);
        let job_id = app.job_queue.jobs[0].id.clone();

        // Start the job
        assert!(app.start_job(&job_id).is_ok());
        assert_eq!(app.current_job_id, Some(job_id.clone()));

        // Simulate sending some G-code lines successfully
        app.gcode_content = "G1 X10\nG1 Y20\nG1 Z30\nG1 X40".to_string();
        let lines: Vec<String> = app.gcode_content.lines().map(|s| s.to_string()).collect();

        // Send first two lines successfully
        for i in 0..2 {
            if let Some(job) = app.job_queue.get_job_mut(&job_id) {
                job.last_completed_line = Some(i);
                job.update_progress((i as f32 + 1.0) / lines.len() as f32);
            }
        }

        // Simulate an error on the third line
        if let Some(job) = app.job_queue.get_job_mut(&job_id) {
            job.interrupt(2); // Interrupt at line 2 (0-indexed)
        }
        app.current_job_id = None;

        // Verify job is interrupted
        let job = app.job_queue.get_job(&job_id).unwrap();
        assert_eq!(job.status, jobs::JobStatus::Paused);
        assert_eq!(job.last_completed_line, Some(2));
        assert!(job.can_resume_job());

        // Test resume functionality
        assert!(app.resume_job(&job_id).is_ok());
        assert_eq!(app.current_job_id, Some(job_id.clone()));

        // Verify job is running again
        let job = app.job_queue.get_job(&job_id).unwrap();
        assert_eq!(job.status, jobs::JobStatus::Running);
        assert_eq!(job.last_completed_line, Some(2)); // Should still have the resume point
    }

    #[test]
    fn test_job_resumption_with_invalid_job() {
        let mut app = GcodeKitApp::default();

        // Try to resume non-existent job
        assert!(app.resume_job("invalid-id").is_err());

        // Create a job but don't interrupt it
        let job = jobs::Job::new("Test Job".to_string(), jobs::JobType::GcodeFile);
        app.job_queue.add_job(job);
        let job_id = app.job_queue.jobs[0].id.clone();

        // Try to resume a job that hasn't been interrupted
        assert!(app.resume_job(&job_id).is_err());
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
        // Handle keyboard shortcuts
        self.handle_keyboard_shortcuts(ctx);

        // Initialize ports on first run
        if self.communication.get_available_ports().is_empty()
            && *self.communication.get_connection_state() == ConnectionState::Disconnected
        {
            self.refresh_ports();
        }

        // Handle recovery operations
        if self.communication.is_recovering() {
            let (should_attempt_reconnect, attempt_count) = {
                let recovery_state = self.communication.get_recovery_state();
                if let Some(last_attempt) = recovery_state.last_reconnect_attempt {
                    let elapsed = last_attempt.elapsed();
                    let config = self.communication.get_recovery_config();
                    (
                        elapsed >= Duration::from_millis(config.reconnect_delay_ms),
                        recovery_state.reconnect_attempts,
                    )
                } else {
                    (false, 0)
                }
            };

            if should_attempt_reconnect {
                // Time to attempt reconnection
                let timestamp = Utc::now().format("%H:%M:%S");
                println!(
                    "[{}] [RECOVERY] Executing scheduled reconnection attempt {}",
                    timestamp, attempt_count
                );
                self.log_console("Executing scheduled reconnection...");
                match self.communication.connect() {
                    Ok(_) => {
                        println!(
                            "[{}] [RECOVERY] Reconnection successful after {} attempts",
                            timestamp, attempt_count
                        );
                        self.log_console("Reconnection successful");
                        self.communication.reset_recovery_state();
                        self.status_message = "Reconnected successfully".to_string();
                    }
                    Err(e) => {
                        let error_msg = format!("Reconnection failed: {}", e);
                        println!(
                            "[{}] [RECOVERY] Reconnection attempt {} failed: {}",
                            timestamp, attempt_count, e
                        );
                        self.log_console(&error_msg);
                        // Try recovery again
                        if let Err(recovery_err) = self.communication.attempt_recovery(&error_msg) {
                            println!(
                                "[{}] [RECOVERY] Recovery failed permanently: {}",
                                timestamp, recovery_err
                            );
                            self.log_console(&format!("Recovery failed: {}", recovery_err));
                            self.status_message =
                                "Recovery failed - manual intervention required".to_string();
                        }
                    }
                }
            }
        }

        // Read responses
        if self.communication.is_connected()
            && let Some(message) = self.communication.read_response()
        {
            if let Some(pos) = self.communication.handle_response(&message) {
                // Position updated
                self.current_position = pos.clone();
                self.log_console(&format!("Position: {}", pos.format_position()));
            } else {
                // Other response, just log
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
                    ui.menu_button("Controller Type", |ui| {
                        if ui
                            .selectable_value(
                                &mut self.controller_type,
                                ControllerType::Grbl,
                                "GRBL",
                            )
                            .clicked()
                        {
                            self.communication =
                                Box::new(communication::GrblCommunication::default());
                            self.refresh_ports();
                        }
                        if ui
                            .selectable_value(
                                &mut self.controller_type,
                                ControllerType::Smoothieware,
                                "Smoothieware",
                            )
                            .clicked()
                        {
                            self.communication =
                                Box::new(communication::SmoothiewareCommunication::default());
                            self.refresh_ports();
                        }
                    });
                    ui.separator();
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
                    ui.separator();
                    if self.controller_type == ControllerType::Grbl {
                        ui.menu_button("Work Coordinate System", |ui| {
                            // This is GRBL-specific, need to handle properly
                            // For now, skip
                        });
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
                    ui.separator();
                    if ui.button("Optimize G-code").clicked() {
                        self.optimize_gcode();
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
                let status_text = match *self.communication.get_connection_state() {
                    ConnectionState::Disconnected => "Disconnected",
                    ConnectionState::Connecting => "Connecting...",
                    ConnectionState::Connected => "Connected",
                    ConnectionState::Error => "Error",
                    ConnectionState::Recovering => "Recovering...",
                };
                ui.colored_label(
                    match *self.communication.get_connection_state() {
                        ConnectionState::Connected => egui::Color32::GREEN,
                        ConnectionState::Error => egui::Color32::RED,
                        ConnectionState::Connecting => egui::Color32::YELLOW,
                        _ => egui::Color32::GRAY,
                    },
                    format!("Status: {}", status_text),
                );

                ui.separator();

                // Device state (locked/alarmed)
                ui.label("State: Idle"); // TODO: Track actual state

                ui.separator();

                // Controller type
                ui.label(format!("Controller: {:?}", self.controller_type));

                ui.separator();

                // Current position
                ui.label(format!(
                    "Position: {}",
                    self.current_position.format_position()
                ));

                ui.separator();

                // Version
                if !self.communication.get_version().is_empty() {
                    ui.label(format!("Version: {}", self.communication.get_version()));
                    ui.separator();
                }

                // Selected port
                if !self.communication.get_selected_port().is_empty() {
                    ui.label(format!("Port: {}", self.communication.get_selected_port()));
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

                widgets::show_connection_widget(ui, self.communication.as_mut());
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
                ui.selectable_value(&mut self.selected_tab, Tab::JobManager, "Job Manager");
            });
            ui.separator();

            match self.selected_tab {
                Tab::GcodeEditor => {
                    if self.gcode_content.is_empty() {
                        ui.centered_and_justified(|ui| {
                            ui.label("No G-code file loaded. Use 'Load File' in the left panel.");
                        });
                    } else {
                        let changed = false;
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            let response = ui.add(
                                egui::TextEdit::multiline(&mut self.gcode_content)
                                    .font(egui::TextStyle::Monospace)
                                    .desired_rows(20)
                                    .layouter(&mut |ui: &egui::Ui, string: &dyn TextBuffer, wrap_width| {
                                        let mut job = LayoutJob::default();
                                        for (i, line) in string.as_str().lines().enumerate() {
                                            // Line number
                                            job.append(&format!("{:05}: ", i + 1), 0.0, TextFormat {
                                                font_id: egui::FontId::monospace(12.0),
                                                color: egui::Color32::DARK_GRAY,
                                                ..Default::default()
                                            });
                                            // Parse line for highlighting
                                            let words: Vec<&str> = line.split_whitespace().collect();
                                            for (j, word) in words.iter().enumerate() {
                                                let color = if word.starts_with('G') && word.len() > 1 && word[1..].chars().all(|c| c.is_ascii_digit()) {
                                                    egui::Color32::BLUE
                                                } else if word.starts_with('M') && word.len() > 1 && word[1..].chars().all(|c| c.is_ascii_digit()) {
                                                    egui::Color32::GREEN
                                                } else if word.starts_with('X') || word.starts_with('Y') || word.starts_with('Z') ||
                                                          word.starts_with('I') || word.starts_with('J') || word.starts_with('K') ||
                                                          word.starts_with('F') || word.starts_with('S') {
                                                    egui::Color32::RED
                                                } else if word.starts_with(';') {
                                                    egui::Color32::GRAY
                                                } else {
                                                    egui::Color32::BLACK
                                                };
                                                job.append(word, 0.0, TextFormat {
                                                    font_id: egui::FontId::monospace(12.0),
                                                    color,
                                                    ..Default::default()
                                                });
                                                if j < words.len() - 1 {
                                                    job.append(" ", 0.0, TextFormat::default());
                                                }
                                            }
                                            job.append("\n", 0.0, TextFormat::default());
                                        }
                                        ui.fonts_mut(|fonts| fonts.layout_job(job))
                                    })
                            );
                            if response.changed() {
                                self.parse_gcode();
                            }
                        });
                    }
                }
                Tab::Visualizer3D => {
                    // Handle keyboard shortcuts
                    if ui.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::R))
                        && let Some(line_number) = self.selected_line
                            && self.communication.is_connected() {
                                self.send_gcode_from_line(line_number);
                            }

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
                            ui.separator();
                            let run_button_enabled = self.selected_line.is_some() && self.communication.is_connected();
                            if ui.add_enabled(run_button_enabled, egui::Button::new("▶️ Run from Selected Line")).clicked()
                                && let Some(line_number) = self.selected_line {
                                    self.send_gcode_from_line(line_number);
                                }
                            ui.separator();
                            ui.label("(Ctrl+R to run from selected line)");
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

                            // Find min/max for scaling (only X and Y for 2D visualization)
                            let mut min_x = f32::INFINITY;
                            let mut max_x = f32::NEG_INFINITY;
                            let mut min_y = f32::INFINITY;
                            let mut max_y = f32::NEG_INFINITY;

                            for segment in &self.parsed_paths {
                                min_x = min_x.min(segment.start.x).min(segment.end.x);
                                max_x = max_x.max(segment.start.x).max(segment.end.x);
                                min_y = min_y.min(segment.start.y).min(segment.end.y);
                                max_y = max_y.max(segment.start.y).max(segment.end.y);
                            }

                            if !self.parsed_paths.is_empty() {
                                let scale_x = bounds.width() / (max_x - min_x).max(1.0);
                                let scale_y = bounds.height() / (max_y - min_y).max(1.0);
                                let scale = scale_x.min(scale_y) * 0.9; // Leave some margin

                                let offset_x = bounds.min.x + (bounds.width() - (max_x - min_x) * scale) / 2.0 - min_x * scale;
                                let offset_y = bounds.min.y + (bounds.height() - (max_y - min_y) * scale) / 2.0 - min_y * scale;

                                for segment in &self.parsed_paths {
                                    let start_pos = egui::pos2(
                                        offset_x + segment.start.x * scale,
                                        offset_y + segment.start.y * scale,
                                    );
                                    let end_pos = egui::pos2(
                                        offset_x + segment.end.x * scale,
                                        offset_y + segment.end.y * scale,
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
                                let current_screen_x = offset_x + self.current_position.x * scale;
                                let current_screen_y = offset_y + self.current_position.y * scale;
                                painter.circle_filled(egui::pos2(current_screen_x, current_screen_y), 5.0, egui::Color32::RED);

                                // Left-click to select segment
                                if response.clicked_by(egui::PointerButton::Primary)
                                    && let Some(click_pos) = response.interact_pointer_pos() {
                                        // Find closest segment to click position
                                        let mut closest_segment = None;
                                        let mut min_distance = f32::INFINITY;

                                        for segment in &self.parsed_paths {
                                            let start_screen = egui::pos2(
                                                offset_x + segment.start.x * scale,
                                                offset_y + segment.start.y * scale,
                                            );
                                            let end_screen = egui::pos2(
                                                offset_x + segment.end.x * scale,
                                                offset_y + segment.end.y * scale,
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

                                // Right-click to jog
                                if response.clicked_by(egui::PointerButton::Secondary) {
                                    if self.communication.is_connected() {
                                        if let Some(click_pos) = response.interact_pointer_pos() {
                                            let gcode_x = (click_pos.x - offset_x) / scale;
                                            let gcode_y = (click_pos.y - offset_y) / scale;
                                            let delta_x = gcode_x - self.current_position.x;
                                            let delta_y = gcode_y - self.current_position.y;
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
                            if let Some(sending_line) = self.sending_from_line {
                                ui.colored_label(egui::Color32::GREEN, format!("Sending from line {}", sending_line + 1));
                            }
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
                Tab::JobManager => {
                    self.show_job_manager_tab(ui);
                }
                Tab::Designer => {
                    if let Some(event) = self.designer.show_ui(ui) {
                        match event {
                            designer::DesignerEvent::ExportGcode => {
                                self.export_design_to_gcode();
                            }
                            designer::DesignerEvent::ImportFile => {
                                self.import_design_file();
                            }
                        }
                    }
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
    }
}
