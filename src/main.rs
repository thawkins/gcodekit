//! # gcodekit
//!
//! A comprehensive GUI application for CNC machine control, G-code generation,
//! and CAM operations. Built with Rust and egui.
//!
//! ## Features
//!
//! - CNC machine communication and control
//! - G-code editing and visualization
//! - CAM operations for shape generation
//! - Material management
//! - Job queuing and management
//! - Real-time machine monitoring

use chrono::Utc;
use eframe::egui;
use std::time::Duration;
use tracing::{debug, info};

mod app;
pub mod cam;
mod communication;
mod designer;
mod errors;
mod firmware;
mod gcode;
mod input;
mod jobs;
mod layout;
mod materials;
mod ops;
mod types;
mod ui;
mod web_pendant;
mod widgets;

use crate::types::{MachineMode, MachinePosition};
use app::GcodeKitApp;

use communication::{ConnectionState, ControllerType};

fn main() -> Result<(), eframe::Error> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Create default communication
    let communication: Box<dyn crate::communication::CncController> = match ControllerType::Grbl {
        ControllerType::Grbl => Box::new(crate::communication::GrblCommunication::default()),
        _ => Box::new(crate::communication::GrblCommunication::default()), // Default to Grbl
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_maximized(true)
            .with_title("gcodekit"),
        ..Default::default()
    };

    eframe::run_native(
        "gcodekit",
        options,
        Box::new(|_cc| {
            let mut app = GcodeKitApp::default();
            app.machine.communication = communication;
            Ok(Box::new(app))
        }),
    )
}

impl GcodeKitApp {
    /// Refreshes the list of available serial ports for CNC controller communication.
    /// This method queries the system for all available serial ports and updates
    /// the communication module's port list.
    fn refresh_ports(&mut self) {
        self.machine.communication.refresh_ports();
        self.machine.available_ports = self.machine.communication.get_available_ports().clone();
    }

    /// Attempts to establish a connection to the selected CNC controller device.
    /// Uses the currently selected port and controller type to initialize communication.
    /// Updates status messages and logs connection results.
    fn connect_to_device(&mut self) {
        if let Err(e) = self.machine.communication.connect() {
            let msg = format!("Connection error: {}", e);
            self.machine.status_message = msg.clone();
            self.machine.connection_state = ConnectionState::Error;
            self.log_console(&msg);
        } else {
            self.machine.connection_state = ConnectionState::Connected;
            let msg = "Connected successfully".to_string();
            self.machine.status_message = msg.clone();
            self.log_console(&msg);
        }
    }

    /// Disconnects from the currently connected CNC controller device.
    /// Clears any active sending indicators and logs the disconnection status.
    fn disconnect_from_device(&mut self) {
        self.machine.communication.disconnect();
        self.machine.connection_state = ConnectionState::Disconnected;
        self.gcode.sending_from_line = None; // Clear sending indicator
        let msg = self.machine.communication.get_status_message().to_string();
        self.log_console(&msg);
    }

    /// Moves the specified machine axis by the given distance.
    /// Sends a jog command to the CNC controller.
    ///
    /// # Arguments
    /// * `axis` - The axis to move ('X', 'Y', 'Z', etc.)
    /// * `distance` - The distance to move in machine units
    fn jog_axis(&mut self, axis: char, distance: f32) {
        self.machine.communication.jog_axis(axis, distance);
        self.machine.status_message = self.machine.communication.get_status_message().to_string();
    }

    /// Homes all machine axes to their reference positions.
    /// Sends a homing command to the CNC controller.
    fn home_all_axes(&mut self) {
        self.machine.communication.home_all_axes();
        self.machine.status_message = self.machine.communication.get_status_message().to_string();
    }

    /// Sends the current spindle speed override value to the CNC controller.
    /// Updates the controller's spindle speed multiplier.
    fn send_spindle_override(&mut self) {
        self.machine
            .communication
            .send_spindle_override(self.machine.spindle_override);
        self.machine.status_message = self.machine.communication.get_status_message().to_string();
    }

    /// Sends the current feed rate override value to the CNC controller.
    /// Updates the controller's feed rate multiplier and logs the change.
    fn send_feed_override(&mut self) {
        self.machine
            .communication
            .send_feed_override(self.machine.feed_override);
        self.machine.status_message = self.machine.communication.get_status_message().to_string();
        let message = self.machine.status_message.clone();
        self.log_console(&message);
    }

    /// Logs a message to the console with a timestamp.
    /// Maintains a rolling buffer of the last 1000 messages.
    ///
    /// # Arguments
    /// * `message` - The message to log

    /// Executes a user-defined script for automation.
    /// Currently a placeholder - full implementation is TODO.
    /// Handles communication errors by attempting recovery strategies.
    /// Logs errors and initiates recovery actions like reconnection or command retry.
    ///
    /// # Arguments
    /// * `error` - The error message describing the communication failure
    fn handle_communication_error(&mut self, error: &str) {
        let timestamp = Utc::now().format("%H:%M:%S").to_string();
        tracing::error!(timestamp, error, "Communication error");
        self.log_console(&format!("Communication error: {}", error));

        // Attempt recovery
        match self.machine.communication.attempt_recovery(error) {
            Ok(action) => {
                let action_msg = match action {
                    communication::RecoveryAction::Reconnect => {
                        self.log_console("Attempting to reconnect...");
                        tracing::info!(timestamp, "Scheduled reconnection attempt");
                        "Attempting recovery - reconnecting...".to_string()
                    }
                    communication::RecoveryAction::RetryCommand => {
                        self.log_console("Retrying last command...");
                        tracing::info!(timestamp, "Retrying last command");
                        "Retrying command...".to_string()
                    }
                    communication::RecoveryAction::ResetController => {
                        self.log_console("Resetting controller...");
                        tracing::info!(timestamp, "Resetting controller");
                        "Resetting controller...".to_string()
                    }
                    communication::RecoveryAction::SkipCommand => {
                        self.log_console("Skipping failed command...");
                        tracing::info!(timestamp, "Skipping failed command");
                        "Skipping failed command".to_string()
                    }
                    communication::RecoveryAction::AbortJob => {
                        self.log_console("Aborting current job due to critical error");
                        tracing::info!(timestamp, "Aborting current job due to critical error");
                        // Clear current job if aborting
                        self.job.current_job_id = None;
                        "Critical error - aborting job".to_string()
                    }
                };
                self.machine.status_message = action_msg;
            }
            Err(recovery_error) => {
                tracing::error!(timestamp, recovery_error, "Recovery failed");
                self.log_console(&format!("Recovery failed: {}", recovery_error));
                self.machine.status_message = format!("Error recovery failed: {}", recovery_error);
                // Clear current job on recovery failure
                self.job.current_job_id = None;
            }
        }
    }

    /// Handles automatic recovery operations for communication failures.
    fn handle_recovery_operations(&mut self) {
        if self.machine.communication.is_recovering() {
            let (should_attempt_reconnect, attempt_count) = {
                let recovery_state = self.machine.communication.get_recovery_state();
                if let Some(last_attempt) = recovery_state.last_reconnect_attempt {
                    let elapsed = last_attempt.elapsed();
                    let config = self.machine.communication.get_recovery_config();
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
                info!(
                    "[{}] [RECOVERY] Executing scheduled reconnection attempt {}",
                    timestamp, attempt_count
                );
                self.log_console("Executing scheduled reconnection...");
                match self.machine.communication.connect() {
                    Ok(_) => {
                        info!(
                            "[{}] [RECOVERY] Executing scheduled reconnection attempt {}",
                            timestamp, attempt_count
                        );
                        self.log_console("Reconnection successful");
                        self.machine.communication.reset_recovery_state();
                        self.machine.status_message = "Reconnected successfully".to_string();
                        self.machine.connection_state = ConnectionState::Connected;
                    }
                    Err(e) => {
                        let error_msg = format!("Reconnection failed: {}", e);
                        info!(
                            "[{}] [RECOVERY] Reconnection successful after {} attempts",
                            timestamp, attempt_count
                        );
                        self.log_console(&error_msg);
                        // Try recovery again
                        if let Err(recovery_err) =
                            self.machine.communication.attempt_recovery(&error_msg)
                        {
                            info!(
                                "[{}] [RECOVERY] Recovery failed permanently: {}",
                                timestamp, recovery_err
                            );
                            self.log_console(&format!("Recovery failed: {}", recovery_err));
                            self.machine.status_message =
                                "Recovery failed - manual intervention required".to_string();
                        }
                    }
                }
            }
        }
    }

    /// Handles incoming communication responses from the CNC controller.
    fn handle_communication_responses(&mut self) {
        if *self.machine.communication.get_connection_state() == ConnectionState::Connected
            && let Some(message) = self.machine.communication.read_response()
        {
            debug!("Device response: {}", message.trim());
            if let Some(pos) = self.machine.communication.handle_response(&message) {
                // Position updated
                self.machine.current_position = pos.clone();
                self.log_console(&format!("Position: {}", pos.format()));
            } else {
                // Other response, just log if not "ok"
                if message.trim() != "ok" {
                    self.log_console(&format!("Recv: {}", message));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gcode_app_initialization() {
        let app = GcodeKitApp::default();

        assert_eq!(app.ui.selected_tab, crate::types::Tab::Designer);
        assert!(app.gcode.gcode_content.is_empty());
        assert!(app.gcode.gcode_filename.is_empty());
        assert_eq!(app.machine.jog_step_size, 0.0); // Default f32 is 0.0
        assert_eq!(app.machine.spindle_override, 0.0);
        assert_eq!(app.machine.feed_override, 0.0);
        assert_eq!(app.machine.machine_mode, MachineMode::CNC);
        assert!(app.machine.console_messages.is_empty());
        assert_eq!(app.machine.status_message, String::new());
    }

    #[test]
    fn test_generate_rectangle_gcode() {
        let mut app = GcodeKitApp::default();
        app.cam.shape_width = 100.0;
        app.cam.shape_height = 50.0;
        app.cam.tool_feed_rate = 500.0;

        app.generate_rectangle();

        assert!(app.gcode.gcode_content.contains("G21 ; Set units to mm"));
        assert!(
            app.gcode
                .gcode_content
                .contains("G90 ; Absolute positioning")
        );
        assert!(app.gcode.gcode_content.contains("G0 X0 Y0 ; Go to origin"));
        assert!(
            app.gcode
                .gcode_content
                .contains("G1 X100 Y0 F500 ; Bottom edge")
        );
        assert!(
            app.gcode
                .gcode_content
                .contains("G1 X100 Y50 F500 ; Right edge")
        );
        assert!(
            app.gcode
                .gcode_content
                .contains("G1 X0 Y50 F500 ; Top edge")
        );
        assert!(
            app.gcode
                .gcode_content
                .contains("G1 X0 Y0 F500 ; Left edge")
        );
        assert!(app.gcode.gcode_content.contains("M30 ; End program"));
        assert_eq!(app.gcode.gcode_filename, "generated_rectangle.gcode");
        assert_eq!(
            app.machine.status_message,
            "Rectangle G-code generated".to_string()
        );
    }

    #[test]
    fn test_generate_circle_gcode() {
        let mut app = GcodeKitApp::default();
        app.cam.shape_radius = 25.0;
        app.cam.tool_feed_rate = 300.0;

        app.generate_circle();

        assert!(app.gcode.gcode_content.contains("G21 ; Set units to mm"));
        assert!(
            app.gcode
                .gcode_content
                .contains("G90 ; Absolute positioning")
        );
        assert!(
            app.gcode
                .gcode_content
                .contains("G0 X25 Y25 ; Go to circle center")
        );
        assert!(
            app.gcode
                .gcode_content
                .contains("G2 I-25 J-25 F300 ; Clockwise circle")
        );
        assert!(app.gcode.gcode_content.contains("M30 ; End program"));
        assert_eq!(app.gcode.gcode_filename, "generated_circle.gcode");
        assert_eq!(
            app.machine.status_message,
            "Circle G-code generated".to_string()
        );
    }

    #[test]
    fn test_generate_toolpath_with_existing_gcode() {
        let mut app = GcodeKitApp::default();
        app.gcode.gcode_content = "G1 X10 Y10\nG1 X20 Y20".to_string();
        app.cam.tool_spindle_speed = 1000.0;
        app.cam.tool_feed_rate = 400.0;

        app.generate_toolpath();

        assert!(app.gcode.gcode_content.contains("G21 ; Set units to mm"));
        assert!(app.gcode.gcode_content.contains("M3 S1000 ; Spindle on"));
        assert!(app.gcode.gcode_content.contains("G1 F400 ; Set feed rate"));
        assert!(app.gcode.gcode_content.contains("G1 X10 Y10"));
        assert!(app.gcode.gcode_content.contains("G1 X20 Y20"));
        assert_eq!(
            app.machine.status_message,
            "Toolpath parameters added".to_string()
        );
    }

    #[test]
    fn test_generate_toolpath_without_gcode() {
        let mut app = GcodeKitApp::default();
        // gcode_content is empty by default
        app.cam.tool_spindle_speed = 1000.0;
        app.cam.tool_feed_rate = 400.0;

        app.generate_toolpath();

        assert_eq!(
            app.machine.status_message,
            "No G-code to modify".to_string()
        );
        assert!(app.gcode.gcode_content.is_empty());
    }

    #[test]
    fn test_log_console_functionality() {
        let mut app = GcodeKitApp::default();

        app.log_console("Test message");

        assert_eq!(app.machine.console_messages.len(), 1);
        assert!(app.machine.console_messages[0].contains("Test message"));
        assert!(app.machine.console_messages[0].contains("[")); // Should contain timestamp
        assert!(app.machine.console_messages[0].contains("]"));
    }

    #[test]
    fn test_console_message_limit() {
        let mut app = GcodeKitApp::default();

        // Add more than 1000 messages
        for i in 0..1010 {
            app.log_console(&format!("Message {}", i));
        }

        // Should only keep the last 1000 messages
        assert_eq!(app.machine.console_messages.len(), 1000);
        assert!(app.machine.console_messages[0].contains("Message 10")); // First message should be removed
        assert!(app.machine.console_messages[999].contains("Message 1009")); // Last message should be kept
    }

    #[test]
    fn test_job_resumption_integration() {
        let mut app = GcodeKitApp::default();

        // Create a job
        let job = jobs::Job::new("Test Job".to_string(), jobs::JobType::GcodeFile);
        app.job.job_queue.add_job(job);
        let job_id = app.job.job_queue.jobs[0].id.clone();

        // Start the job
        assert!(app.start_job(&job_id).is_ok());
        assert_eq!(app.job.current_job_id, Some(job_id.clone()));

        // Simulate sending some G-code lines successfully
        app.gcode.gcode_content = "G1 X10\nG1 Y20\nG1 Z30\nG1 X40".to_string();
        let lines: Vec<String> = app
            .gcode
            .gcode_content
            .lines()
            .map(|s| s.to_string())
            .collect();

        // Send first two lines successfully
        for i in 0..2 {
            if let Some(job) = app.job.job_queue.get_job_mut(&job_id) {
                job.last_completed_line = Some(i);
                job.update_progress((i as f32 + 1.0) / lines.len() as f32);
            }
        }

        // Simulate an error on the third line
        if let Some(job) = app.job.job_queue.get_job_mut(&job_id) {
            job.interrupt(2); // Interrupt at line 2 (0-indexed)
        }
        app.job.current_job_id = None;

        // Verify job is interrupted
        let job = app.job.job_queue.get_job(&job_id).unwrap();
        assert_eq!(job.status, jobs::JobStatus::Paused);
        assert_eq!(job.last_completed_line, Some(2));
        assert!(job.can_resume_job());

        // Test resume functionality
        assert!(app.resume_job(&job_id).is_ok());
        assert_eq!(app.job.current_job_id, Some(job_id.clone()));

        // Verify job is running again
        let job = app.job.job_queue.get_job(&job_id).unwrap();
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
        app.job.job_queue.add_job(job);
        let job_id = app.job.job_queue.jobs[0].id.clone();

        // Try to resume a job that hasn't been interrupted
        assert!(app.resume_job(&job_id).is_err());
    }

    #[test]
    fn test_generate_image_engraving_placeholder() {
        let mut app = GcodeKitApp::default();
        app.cam.image_resolution = 300.0;
        app.cam.image_max_power = 80.0;

        app.generate_image_engraving();

        assert!(app.gcode.gcode_content.contains("; Image engraving G-code"));
        assert!(app.gcode.gcode_content.contains("; Resolution: 300 dpi"));
        assert!(app.gcode.gcode_content.contains("; Max Power: 80%"));
        assert!(
            app.gcode
                .gcode_content
                .contains("; TODO: Implement actual image processing")
        );
        assert!(app.gcode.gcode_content.contains("M30 ; End program"));
        assert_eq!(app.gcode.gcode_filename, "image_engraving.gcode");
        assert_eq!(
            app.machine.status_message,
            "Image engraving G-code generated (placeholder)".to_string()
        );
    }

    #[test]
    fn test_generate_tabbed_box_placeholder() {
        let mut app = GcodeKitApp::default();
        app.cam.box_length = 100.0;
        app.cam.box_width = 80.0;
        app.cam.box_height = 50.0;
        app.cam.tab_size = 10.0;

        app.generate_tabbed_box();

        assert!(app.gcode.gcode_content.contains("; Tabbed box G-code"));
        assert!(
            app.gcode
                .gcode_content
                .contains("; Dimensions: 100x80x50mm")
        );
        assert!(app.gcode.gcode_content.contains("; Tab size: 10mm"));
        assert!(
            app.gcode
                .gcode_content
                .contains("; TODO: Implement actual box cutting paths")
        );
        assert!(app.gcode.gcode_content.contains("M30 ; End program"));
        assert_eq!(app.gcode.gcode_filename, "tabbed_box.gcode");
        assert_eq!(
            app.machine.status_message,
            "Tabbed box G-code generated (placeholder)".to_string()
        );
    }

    #[test]
    fn test_generate_jigsaw_placeholder() {
        let mut app = GcodeKitApp::default();
        app.cam.jigsaw_pieces = 50;
        app.cam.jigsaw_complexity = 3;

        app.generate_jigsaw();

        assert!(app.gcode.gcode_content.contains("; Jigsaw puzzle G-code"));
        assert!(app.gcode.gcode_content.contains("; Pieces: 50"));
        assert!(app.gcode.gcode_content.contains("; Complexity: 3"));
        assert!(
            app.gcode
                .gcode_content
                .contains("; TODO: Implement actual puzzle piece cutting")
        );
        assert!(app.gcode.gcode_content.contains("M30 ; End program"));
        assert_eq!(app.gcode.gcode_filename, "jigsaw_puzzle.gcode");
        assert_eq!(
            app.machine.status_message,
            "Jigsaw G-code generated (placeholder)".to_string()
        );
    }

    #[test]
    fn test_reset_add_material_dialog() {
        let mut app = GcodeKitApp::default();

        // Set some values
        app.ui.new_material_name = "Test Material".to_string();
        app.ui.new_material_type = materials::MaterialType::Metal;
        app.ui.new_material_density = 7800.0;
        app.ui.new_material_hardness = 200.0;
        app.ui.new_material_cutting_speed = 100.0;
        app.ui.new_material_feed_rate = 500.0;
        app.ui.new_material_spindle_speed = 2000.0;
        app.ui.new_material_tool_material = "Carbide".to_string();
        app.ui.new_material_tool_coating = "TiN".to_string();
        app.ui.new_material_chip_load_min = 0.05;
        app.ui.new_material_chip_load_max = 0.15;
        app.ui.new_material_notes = "Test notes".to_string();

        // Reset
        app.reset_add_material_dialog();

        // Check all fields are reset
        assert!(app.ui.new_material_name.is_empty());
        assert_eq!(app.ui.new_material_type, materials::MaterialType::Wood);
        assert_eq!(app.ui.new_material_density, 0.0);
        assert_eq!(app.ui.new_material_hardness, 0.0);
        assert_eq!(app.ui.new_material_cutting_speed, 0.0);
        assert_eq!(app.ui.new_material_feed_rate, 0.0);
        assert_eq!(app.ui.new_material_spindle_speed, 0.0);
        assert!(app.ui.new_material_tool_material.is_empty());
        assert!(app.ui.new_material_tool_coating.is_empty());
        assert_eq!(app.ui.new_material_chip_load_min, 0.0);
        assert_eq!(app.ui.new_material_chip_load_max, 0.0);
        assert!(app.ui.new_material_notes.is_empty());
    }

    #[test]
    fn test_material_database_operations() {
        let mut app = GcodeKitApp::default();

        // Create a material
        let material = materials::MaterialProperties::new(
            "Test Wood",
            materials::MaterialType::Wood,
            materials::MaterialSubtype::Custom,
        )
        .with_density(600.0)
        .with_hardness(50.0);

        // Add to database
        app.material_database.add_material(material);

        // Check it was added
        let material = app.material_database.get_material("Test Wood");
        assert!(material.is_some());
        let material = material.unwrap();
        assert_eq!(material.name, "Test Wood");
        assert_eq!(material.material_type, materials::MaterialType::Wood);
        assert_eq!(material.density, 600.0);
        assert_eq!(material.hardness, 50.0);
    }

    #[test]
    fn test_job_creation_with_material() {
        let mut app = GcodeKitApp::default();

        // Add a material
        let material = materials::MaterialProperties::new(
            "Test Material",
            materials::MaterialType::Wood,
            materials::MaterialSubtype::Custom,
        );
        app.material_database.add_material(material);

        // Create job with material
        app.ui.new_job_name = "Test Job".to_string();
        app.ui.new_job_type = jobs::JobType::GcodeFile;
        app.ui.selected_material = Some("Test Material".to_string());

        let job = jobs::Job::new(app.ui.new_job_name.clone(), app.ui.new_job_type.clone())
            .with_material(app.ui.selected_material.clone().unwrap());

        app.job.job_queue.add_job(job);

        // Check job was created with material
        let jobs = app.job.job_queue.jobs;
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].name, "Test Job");
        assert_eq!(jobs[0].job_type, jobs::JobType::GcodeFile);
        assert_eq!(jobs[0].material, app.ui.selected_material);
    }
}

impl eframe::App for GcodeKitApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle keyboard shortcuts
        self.handle_keyboard_shortcuts(ctx);

        // Initialize ports on first run
        if self.machine.available_ports.is_empty()
            && *self.machine.communication.get_connection_state() == ConnectionState::Disconnected
        {
            self.machine.communication.refresh_ports();
            self.machine.available_ports = self.machine.communication.get_available_ports().clone();
        }

        // Handle recovery operations
        self.handle_recovery_operations();

        // Read responses
        self.handle_communication_responses();

        ui::panels::render_panels(self, ctx);
    }
}
