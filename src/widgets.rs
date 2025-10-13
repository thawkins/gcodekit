pub mod calibration;
pub mod cam_operations;
pub mod connection;
pub mod gcode_loading;
pub mod job_scheduling;
pub mod jog;
pub mod machine_control;
pub mod overrides;
pub mod safety;
pub mod tool_management;

// Re-export the main widget functions for easy access
pub use calibration::show_calibration_widget;
pub use cam_operations::show_cam_operations_widget;
pub use connection::show_connection_widget;
pub use gcode_loading::show_gcode_loading_widget;
pub use job_scheduling::JobSchedulingWidget;
pub use jog::show_jog_widget;
pub use machine_control::show_machine_control_widget;
pub use overrides::show_overrides_widget;
pub use safety::show_safety_widget;
pub use tool_management::show_tool_management_widget;
