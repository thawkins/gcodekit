pub mod calibration;
pub mod cam_operations;
pub mod connection;
pub mod error_recovery;
pub mod gcode_loading;
pub mod job_scheduling;
pub mod jog;
pub mod machine_control;
pub mod overrides;
pub mod safety;
pub mod settings_panel;
pub mod tool_management;

// Re-export the main widget functions for easy access
pub use connection::show_connection_widget;
pub use error_recovery::show_error_recovery_widget;
pub use jog::show_jog_widget;
pub use overrides::show_overrides_widget;
pub use settings_panel::{draw_settings_dialogs, draw_settings_panel, SettingsUiState};
