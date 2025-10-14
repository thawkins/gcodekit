pub mod designer;
pub mod device_console;
pub mod gcode_editor;
pub mod job_manager;
pub mod visualizer_3d;

// Re-export the tab functions for easy access
pub use designer::show_designer_tab;
pub use device_console::show_device_console_tab;
pub use gcode_editor::show_gcode_editor_tab;
pub use job_manager::show_job_manager_tab;
pub use visualizer_3d::show_visualizer_3d_tab;
