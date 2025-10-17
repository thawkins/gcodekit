use gcodekit::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gcode_app_initialization() {
        let app = GcodeKitApp::default();

        assert_eq!(app.ui.selected_tab, crate::types::Tab::GcodeEditor);
        assert!(app.gcode.gcode_content.is_empty());
        assert!(app.gcode.gcode_filename.is_empty());
        assert_eq!(app.machine.jog_step_size, 1.0);
        assert_eq!(app.machine.spindle_override, 1.0);
        assert_eq!(app.machine.feed_override, 1.0);
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
        assert!(app
            .gcode
            .gcode_content
            .contains("G90 ; Absolute positioning"));
        assert!(app.gcode.gcode_content.contains("G0 X0 Y0 ; Go to origin"));
        assert!(app
            .gcode
            .gcode_content
            .contains("G1 X100 Y0 F500 ; Bottom edge"));
        assert!(app
            .gcode
            .gcode_content
            .contains("G1 X100 Y50 F500 ; Right edge"));
        assert!(app
            .gcode
            .gcode_content
            .contains("G1 X0 Y50 F500 ; Top edge"));
        assert!(app
            .gcode
            .gcode_content
            .contains("G1 X0 Y0 F500 ; Left edge"));
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
        assert!(app
            .gcode
            .gcode_content
            .contains("G90 ; Absolute positioning"));
        assert!(app
            .gcode
            .gcode_content
            .contains("G0 X25 Y25 ; Go to circle center"));
        assert!(app
            .gcode
            .gcode_content
            .contains("G2 I-25 J-25 F300 ; Clockwise circle"));
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
        assert!(app
            .gcode
            .gcode_content
            .contains("; TODO: Implement actual image processing"));
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
        assert!(app
            .gcode
            .gcode_content
            .contains("; Dimensions: 100x80x50mm"));
        assert!(app.gcode.gcode_content.contains("; Tab size: 10mm"));
        assert!(app
            .gcode
            .gcode_content
            .contains("; TODO: Implement actual box cutting paths"));
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
        assert!(app
            .gcode
            .gcode_content
            .contains("; TODO: Implement actual puzzle piece cutting"));
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

    #[test]
    fn test_gcode_batching_logic() {
        let mut app = GcodeKitApp::default();

        // Load the large G-code file (131 lines)
        let large_gcode = std::fs::read_to_string("assets/gcode/square_15mm.gcode")
            .expect("Failed to read test G-code file");
        app.gcode.gcode_content = large_gcode.clone();

        // Count total lines and commands
        let lines: Vec<&str> = app.gcode.gcode_content.lines().collect();
        let total_lines = lines.len();
        let commands: Vec<&str> = lines
            .iter()
            .filter(|line| {
                let trimmed = line.trim();
                !trimmed.is_empty() && !trimmed.starts_with(';')
            })
            .copied()
            .collect();
        let total_commands = commands.len();

        // Verify we have the expected number of lines
        assert_eq!(total_lines, 131, "G-code file should have 131 lines");

        // Calculate expected batches (10 commands per batch)
        let batch_size = 10;
        let expected_batches = total_commands.div_ceil(batch_size);

        // Verify batching calculation
        assert_eq!(
            total_commands, 85,
            "Should have 85 commands after filtering comments and empty lines"
        );
        assert_eq!(
            expected_batches, 9,
            "Should require 9 batches for 85 commands"
        );

        // Test that the batching logic would work (without actually sending)
        let mut batch_count = 0;
        for batch in commands.chunks(batch_size) {
            batch_count += 1;
            assert!(
                batch.len() <= batch_size,
                "Batch size should not exceed {}",
                batch_size
            );
        }
        assert_eq!(
            batch_count, expected_batches,
            "Should create correct number of batches"
        );
    }

    #[test]
    fn test_send_gcode_with_debug_logging() {
        let mut app = GcodeKitApp::default();

        // Load a small G-code file
        let small_gcode = "G21 ; Set units to mm\n\
                           G90 ; Absolute positioning\n\
                           G0 X0 Y0 ; Go to origin\n\
                           G1 X100 Y0 F500 ; Bottom edge\n\
                           M30 ; End program\n";
        app.gcode.gcode_content = small_gcode.to_string();

        // Test that send_gcode detects multi-line content
        // Note: This will fail at connection check since we're not connected
        app.send_gcode(small_gcode);

        // Verify the content was set
        assert_eq!(app.gcode.gcode_content, small_gcode);

        // Check that it detected multi-line and tried to send
        // The status should indicate connection failure
        assert!(app.machine.status_message.contains("Not connected"));

        // Check console messages for our debug logs
        let console_messages = &app.machine.console_messages;
        assert!(!console_messages.is_empty());

        // Find the send_gcode debug message
        let send_gcode_msg = console_messages
            .iter()
            .find(|msg| msg.contains("send_gcode: Called with content"));
        assert!(
            send_gcode_msg.is_some(),
            "send_gcode debug message not found"
        );

        // Verify it detected newlines
        assert!(send_gcode_msg.unwrap().contains("contains_newlines = true"));
    }

    #[test]
    fn test_larger_gcode_file_batching() {
        let mut app = GcodeKitApp::default();

        // Create a larger G-code file by duplicating the square pattern
        let base_gcode = std::fs::read_to_string("assets/gcode/square_15mm.gcode")
            .expect("Failed to read test G-code file");

        // Duplicate the content 5 times to create a larger file
        let mut large_gcode = String::new();
        for _ in 0..5 {
            large_gcode.push_str(&base_gcode);
            large_gcode.push('\n');
        }
        app.gcode.gcode_content = large_gcode.clone();

        // Count total lines and commands
        let lines: Vec<&str> = app.gcode.gcode_content.lines().collect();
        let total_lines = lines.len();
        let commands: Vec<&str> = lines
            .iter()
            .filter(|line| {
                let trimmed = line.trim();
                !trimmed.is_empty() && !trimmed.starts_with(';')
            })
            .copied()
            .collect();
        let total_commands = commands.len();

        // Verify we have the expected scale (5x the original)
        assert_eq!(
            total_lines,
            130 * 5 + 5,
            "Should have 5x lines plus separators"
        );
        assert_eq!(total_commands, 85 * 5, "Should have 5x commands");

        // Calculate expected batches (10 commands per batch)
        let batch_size = 10;
        let expected_batches = total_commands.div_ceil(batch_size);

        // Verify batching calculation for larger file
        assert_eq!(
            expected_batches, 43,
            "Should require 43 batches for 425 commands"
        );

        // Test that the batching logic would work (without actually sending)
        let mut total_batch_commands = 0;
        for batch in commands.chunks(batch_size) {
            assert!(
                batch.len() <= batch_size,
                "Batch size should not exceed {}",
                batch_size
            );
            total_batch_commands += batch.len();
        }
        assert_eq!(
            total_batch_commands, total_commands,
            "All commands should be batched"
        );
    }
}
