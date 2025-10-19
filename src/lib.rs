#![allow(dead_code, unused_variables, unused_imports, unused_comparisons)]

pub mod app;
pub mod calibration;
pub mod cam;
pub mod communication;
pub mod designer;
pub mod errors;
pub mod gcode;
pub mod gcodeedit;
pub mod input;
pub mod jobs;
pub mod layout;
pub mod materials;
pub mod ops;
pub mod types;
pub mod ui;
pub mod web_pendant;
pub mod widgets;

pub use crate::designer::Material;
pub use crate::designer::Tool;
pub use app::state::GcodeKitApp;
pub use calibration::{
    BacklashCompensation, CalibrationProcedure, CalibrationProfiles, CalibrationResult,
    HomingConfiguration, MachineCalibration, StepCalibration,
};
pub use cam::CAMOperation;
pub use cam::CAMParameters;
pub use communication::CncController;
pub use designer::DesignerState;
pub use designer::DrawingTool;
pub use designer::Shape;
pub use designer::ToolpathPattern;
pub use gcodeedit::GcodeEditorState;
pub use jobs::Job;
pub use jobs::JobType;
pub use materials::MaterialDatabase;
pub use types::MachineMode;
pub use types::MachinePosition;
pub use types::MoveType;
pub use types::PathSegment;
pub use types::Tab;
