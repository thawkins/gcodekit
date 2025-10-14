use std::collections::HashMap;

use crate::communication::{ConnectionState, ControllerType};
use crate::designer::bitmap_processing::VectorizationConfig;
use crate::designer::parametric_design::ParametricConfig;
use crate::designer::{DesignerState, Tool};
use crate::input::{Action, KeyBinding, create_default_keybindings};
use crate::jobs::{JobQueue, JobType};
use crate::materials::MaterialDatabase;
use crate::materials::MaterialType;
use crate::types::{MachineMode, MachinePosition, PathSegment, Tab};

// UI State - Ephemeral state for UI rendering and interaction
#[derive(Debug, Clone)]
pub struct UiState {
    pub selected_tab: Tab,
    pub show_job_creation_dialog: bool,
    pub new_job_name: String,
    pub new_job_type: JobType,
    pub new_job_file_path: String,
    pub selected_material: Option<String>,
    pub show_add_material_dialog: bool,
    pub new_material_name: String,
    pub new_material_type: MaterialType,
    pub new_material_density: f32,
    pub new_material_hardness: f32,
    pub new_material_cutting_speed: f32,
    pub new_material_feed_rate: f32,
    pub new_material_spindle_speed: f32,
    pub new_material_tool_material: String,
    pub new_material_tool_coating: String,
    pub new_material_chip_load_min: f32,
    pub new_material_chip_load_max: f32,
    pub new_material_notes: String,
    pub show_left_panel: bool,
    pub show_right_panel: bool,
    pub left_panel_width: f32,
    pub right_panel_width: f32,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            selected_tab: Tab::default(),
            show_job_creation_dialog: false,
            new_job_name: String::new(),
            new_job_type: JobType::GcodeFile,
            new_job_file_path: String::new(),
            selected_material: None,
            show_add_material_dialog: false,
            new_material_name: String::new(),
            new_material_type: MaterialType::Wood,
            new_material_density: 0.0,
            new_material_hardness: 0.0,
            new_material_cutting_speed: 0.0,
            new_material_feed_rate: 0.0,
            new_material_spindle_speed: 0.0,
            new_material_tool_material: String::new(),
            new_material_tool_coating: String::new(),
            new_material_chip_load_min: 0.0,
            new_material_chip_load_max: 0.0,
            new_material_notes: String::new(),
            show_left_panel: true,
            show_right_panel: true,
            left_panel_width: 200.0,
            right_panel_width: 250.0,
        }
    }
}

// CAM State
#[derive(Debug, Clone)]
pub struct CamState {
    pub shape_width: f32,
    pub shape_height: f32,
    pub shape_radius: f32,
    pub tool_feed_rate: f32,
    pub tool_spindle_speed: f32,
    pub image_resolution: f32,
    pub image_max_power: f32,
    pub box_length: f32,
    pub box_width: f32,
    pub box_height: f32,
    pub tab_size: f32,
    pub jigsaw_pieces: i32,
    pub jigsaw_complexity: i32,
    pub current_tool: i32,
    pub tool_library: Vec<Tool>,
    pub vectorization_config: VectorizationConfig,
    pub parametric_shape_type: String,
    pub parametric_config: ParametricConfig,
}

impl Default for CamState {
    fn default() -> Self {
        Self {
            shape_width: 100.0,
            shape_height: 50.0,
            shape_radius: 25.0,
            tool_feed_rate: 100.0,
            tool_spindle_speed: 10000.0,
            image_resolution: 300.0,
            image_max_power: 100.0,
            box_length: 100.0,
            box_width: 50.0,
            box_height: 30.0,
            tab_size: 10.0,
            jigsaw_pieces: 50,
            jigsaw_complexity: 3,
            current_tool: 0,
            tool_library: vec![
                Tool {
                    name: "End Mill 3mm".to_string(),
                    diameter: 3.0,
                    length: 40.0,
                    material: "HSS".to_string(),
                    flute_count: 2,
                    max_rpm: 10000,
                    tool_number: 1,
                    length_offset: 1.0,
                    wear_offset: 0.0,
                },
                Tool {
                    name: "Drill 2mm".to_string(),
                    diameter: 2.0,
                    length: 50.0,
                    material: "HSS".to_string(),
                    flute_count: 1,
                    max_rpm: 5000,
                    tool_number: 2,
                    length_offset: 2.0,
                    wear_offset: 0.0,
                },
            ],
            vectorization_config: VectorizationConfig::default(),
            parametric_shape_type: "Custom".to_string(),
            parametric_config: ParametricConfig::default(),
        }
    }
}

// Job State
#[derive(Debug, Clone)]
pub struct JobState {
    pub job_queue: JobQueue,
    pub current_job_id: Option<String>,
}

impl Default for JobState {
    fn default() -> Self {
        Self {
            job_queue: JobQueue::default(),
            current_job_id: None,
        }
    }
}

// G-code State
#[derive(Debug, Clone)]
pub struct GcodeState {
    pub gcode_content: String,
    pub gcode_filename: String,
    pub parsed_paths: Vec<PathSegment>,
    pub selected_line: Option<usize>,
    pub sending_from_line: Option<usize>,
    pub sending_progress: f32, // 0.0 to 1.0, progress of current send operation
}

impl Default for GcodeState {
    fn default() -> Self {
        Self {
            gcode_content: String::new(),
            gcode_filename: String::new(),
            parsed_paths: Vec::new(),
            selected_line: None,
            sending_from_line: None,
            sending_progress: 0.0,
        }
    }
}

// Machine State
pub struct MachineState {
    pub controller_type: ControllerType,
    pub communication: Box<dyn crate::communication::CncController>,
    pub status_message: String,
    pub jog_step_size: f32,
    pub spindle_override: f32,
    pub feed_override: f32,
    pub machine_mode: MachineMode,
    pub console_messages: Vec<String>,
    pub current_position: MachinePosition,
    pub soft_limits_enabled: bool,
    pub available_ports: Vec<String>,
    pub connection_state: ConnectionState,
    pub selected_port: String,
}

impl Default for MachineState {
    fn default() -> Self {
        Self {
            controller_type: ControllerType::Grbl,
            communication: Box::new(crate::communication::GrblCommunication::default()),
            status_message: String::new(),
            jog_step_size: 1.0,
            spindle_override: 1.0,
            feed_override: 1.0,
            machine_mode: MachineMode::default(),
            console_messages: Vec::new(),
            current_position: MachinePosition::new(0.0, 0.0, 0.0),
            soft_limits_enabled: true,
            available_ports: Vec::new(),
            connection_state: ConnectionState::Disconnected,
            selected_port: String::new(),
        }
    }
}

pub struct GcodeKitApp {
    pub ui: UiState,
    pub cam: CamState,
    pub job: JobState,
    pub gcode: GcodeState,
    pub machine: MachineState,
    pub keybindings: HashMap<Action, KeyBinding>,
    pub designer: DesignerState,
    pub script_content: String,
    pub material_database: MaterialDatabase,
}

impl Default for GcodeKitApp {
    fn default() -> Self {
        Self {
            ui: UiState::default(),
            cam: CamState::default(),
            job: JobState::default(),
            gcode: GcodeState::default(),
            machine: MachineState::default(),
            keybindings: create_default_keybindings(),
            designer: DesignerState::default(),
            script_content: String::new(),
            material_database: MaterialDatabase::default(),
        }
    }
}

impl GcodeKitApp {
    /// Executes a user-defined script for automation.
    /// Currently a placeholder - full implementation is TODO.
    pub fn run_script(&mut self) {
        // TODO: Implement script running
        self.machine.status_message = "Running script...".to_string();
    }
    /// Logs a message to the console with a timestamp.
    /// Maintains a rolling buffer of the last 1000 messages.
    ///
    /// # Arguments
    /// * `message` - The message to log
    pub fn log_console(&mut self, message: &str) {
        let timestamp = chrono::Utc::now().format("%H:%M:%S");
        self.machine
            .console_messages
            .push(format!("[{}] {}", timestamp, message));

        // Keep only last 1000 messages
        if self.machine.console_messages.len() > 1000 {
            self.machine.console_messages.remove(0);
        }
    }
}
