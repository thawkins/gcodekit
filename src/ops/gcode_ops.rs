use crate::GcodeKitApp;

impl GcodeKitApp {
    /// Parses the currently loaded G-code content and extracts path segments.
    /// Identifies move commands (G0, G1, G2, G3) and creates PathSegment objects
    /// for visualization and analysis. Handles absolute/incremental positioning modes.
    pub fn parse_gcode(&mut self) {
        self.gcode.parsed_paths = crate::gcode::parse_gcode(&self.gcode.gcode_content);
    }

    /// Optimizes the currently loaded G-code by removing comments, empty lines,
    /// and redundant commands. Currently performs basic cleanup operations.
    pub fn optimize_gcode(&mut self) {
        if self.gcode.gcode_content.is_empty() {
            self.machine.status_message = "No G-code to optimize".to_string();
            return;
        }

        let original_lines = self.gcode.gcode_content.lines().count();
        let mut optimized_lines = Vec::new();

        for line in self.gcode.gcode_content.lines() {
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

        self.gcode.gcode_content = optimized_lines.join("\n");
        self.parse_gcode(); // Re-parse the optimized G-code

        let optimized_line_count = optimized_lines.len();
        self.machine.status_message = format!(
            "G-code optimized: {} -> {} lines",
            original_lines, optimized_line_count
        );
        // TODO: log_console needs to be accessible
        // self.log_console(&format!(
        //     "Optimized G-code: removed {} lines",
        //     original_lines - optimized_line_count
        // ));
    }

    /// Generates G-code for cutting a rectangular shape.
    /// Creates a simple rectangle path using the configured dimensions and feed rates.
    pub fn generate_rectangle(&mut self) {
        let gcode = format!(
            "G21 ; Set units to mm\n\
             G90 ; Absolute positioning\n\
             G0 X0 Y0 ; Go to origin\n\
             G1 X{} Y0 F{} ; Bottom edge\n\
             G1 X{} Y{} F{} ; Right edge\n\
             G1 X0 Y{} F{} ; Top edge\n\
             G1 X0 Y0 F{} ; Left edge\n\
             M30 ; End program\n",
            self.cam.shape_width,
            self.cam.tool_feed_rate,
            self.cam.shape_width,
            self.cam.shape_height,
            self.cam.tool_feed_rate,
            self.cam.shape_height,
            self.cam.tool_feed_rate,
            self.cam.tool_feed_rate
        );
        self.gcode.gcode_content = gcode;
        self.gcode.gcode_filename = "generated_rectangle.gcode".to_string();
        self.parse_gcode();
        self.machine.status_message = "Rectangle G-code generated".to_string();
    }

    /// Generates G-code for cutting a circular shape.
    /// Creates a clockwise circle using G2 command with configured radius and feed rate.
    pub fn generate_circle(&mut self) {
        let gcode = format!(
            "G21 ; Set units to mm\n\
             G90 ; Absolute positioning\n\
             G0 X{} Y{} ; Go to circle center\n\
             G2 I-{} J-{} F{} ; Clockwise circle\n\
             M30 ; End program\n",
            self.cam.shape_radius,
            self.cam.shape_radius,
            self.cam.shape_radius,
            self.cam.shape_radius,
            self.cam.tool_feed_rate
        );
        self.gcode.gcode_content = gcode;
        self.gcode.gcode_filename = "generated_circle.gcode".to_string();
        self.parse_gcode();
        self.machine.status_message = "Circle G-code generated".to_string();
    }

    /// Adds toolpath parameters (spindle speed, feed rate) to existing G-code.
    /// Prepends M3 (spindle on) and G1 F (feed rate) commands to the current G-code.
    pub fn generate_toolpath(&mut self) {
        // For now, just add toolpath parameters to existing G-code
        if !self.gcode.gcode_content.is_empty() {
            let header = format!(
                "G21 ; Set units to mm\n\
                 M3 S{} ; Spindle on\n\
                 G1 F{} ; Set feed rate\n",
                self.cam.tool_spindle_speed, self.cam.tool_feed_rate
            );
            self.gcode.gcode_content = format!("{}{}", header, self.gcode.gcode_content);
            self.parse_gcode();
            self.machine.status_message = "Toolpath parameters added".to_string();
        } else {
            self.machine.status_message = "No G-code to modify".to_string();
        }
    }

    /// Sends a raw G-code command string to the device.
    /// Currently a placeholder - full implementation is TODO.
    ///
    /// # Arguments
    /// * `gcode` - The G-code command string to send
    pub fn send_gcode(&mut self, gcode: &str) {
        // TODO: Implement sending G-code string
        self.machine.status_message = format!("Sending G-code: {}", gcode);
    }

    /// Sends G-code lines starting from a specified line number to the device.
    /// Handles job progress tracking, error recovery, and communication failures.
    ///
    /// # Arguments
    /// * `start_line` - The zero-based line number to start sending from
    pub fn send_gcode_from_line(&mut self, start_line: usize) {
        if !self.machine.communication.is_connected() {
            self.machine.status_message = "Not connected to device".to_string();
            return;
        }

        if self.gcode.gcode_content.is_empty() {
            self.machine.status_message = "No G-code loaded".to_string();
            return;
        }

        let lines: Vec<String> = self
            .gcode
            .gcode_content
            .lines()
            .map(|s| s.to_string())
            .collect();
        if start_line >= lines.len() {
            self.machine.status_message = "Invalid line number".to_string();
            return;
        }

        let lines_to_send = &lines[start_line..];
        let mut sent_count = 0;

        for (i, line) in lines_to_send.iter().enumerate() {
            let trimmed = line.trim();
            if !trimmed.is_empty() && !trimmed.starts_with(';') {
                match self.machine.communication.send_gcode_line(trimmed) {
                    Ok(_) => {
                        sent_count += 1;
                        // Update job progress
                        if let Some(job_id) = &self.job.current_job_id
                            && let Some(job) = self.job.job_queue.get_job_mut(job_id)
                        {
                            let current_line = start_line + i;
                            job.last_completed_line = Some(current_line);
                            job.update_progress((current_line as f32) / (lines.len() as f32));
                        }
                    }
                    Err(e) => {
                        let error_msg = format!("Error sending line: {}", e);
                        // Interrupt current job on error
                        if let Some(job_id) = &self.job.current_job_id {
                            if let Some(job) = self.job.job_queue.get_job_mut(job_id) {
                                let failed_line = start_line + i;
                                job.interrupt(failed_line);
                                // TODO: log_console
                                // self.log_console(&format!(
                                //     "Job {} interrupted at line {}",
                                //     job_id,
                                //     failed_line + 1
                                // ));
                            }
                            self.job.current_job_id = None;
                        }
                        // TODO: handle_communication_error
                        // self.handle_communication_error(&error_msg);
                        // Continue with next line if recovery was attempted
                        if self.machine.communication.is_recovering() {
                            continue;
                        } else {
                            self.machine.status_message = error_msg;
                            return;
                        }
                    }
                }
            }
        }

        self.gcode.sending_from_line = Some(start_line);
        self.machine.status_message = format!(
            "Sent {} G-code lines from line {}",
            sent_count,
            start_line + 1
        );
        // TODO: log_console
        // self.log_console(&format!(
        //     "Sent {} lines starting from line {}",
        //     sent_count,
        //     start_line + 1
        // ));
    }

    /// Generates placeholder G-code for image engraving.
    /// Currently creates a basic template with engraving parameters.
    /// Full image-to-G-code conversion is TODO.
    pub fn generate_image_engraving(&mut self) {
        // TODO: Implement image to G-code conversion
        let gcode = format!(
            "; Image engraving G-code\n\
             ; Resolution: {} dpi\n\
             ; Max Power: {}%\n\
             ; TODO: Implement actual image processing\n\
             M30 ; End program\n",
            self.cam.image_resolution, self.cam.image_max_power
        );
        self.gcode.gcode_content = gcode;
        self.gcode.gcode_filename = "image_engraving.gcode".to_string();
        self.machine.status_message = "Image engraving G-code generated (placeholder)".to_string();
    }

    /// Generates placeholder G-code for creating a tabbed box.
    /// Currently creates a basic template with box dimensions.
    /// Full box cutting path generation is TODO.
    pub fn generate_tabbed_box(&mut self) {
        // TODO: Implement actual tabbed box generation
        let gcode = format!(
            "; Tabbed box G-code\n\
             ; Dimensions: {}x{}x{}mm\n\
             ; Tab size: {}mm\n\
             ; TODO: Implement actual box cutting paths\n\
             M30 ; End program\n",
            self.cam.box_length, self.cam.box_width, self.cam.box_height, self.cam.tab_size
        );
        self.gcode.gcode_content = gcode;
        self.gcode.gcode_filename = "tabbed_box.gcode".to_string();
        self.machine.status_message = "Tabbed box G-code generated (placeholder)".to_string();
    }

    /// Generates placeholder G-code for creating jigsaw puzzle pieces.
    /// Currently creates a basic template with puzzle parameters.
    /// Full puzzle piece cutting path generation is TODO.
    pub fn generate_jigsaw(&mut self) {
        // TODO: Implement actual jigsaw generation
        let gcode = format!(
            "; Jigsaw puzzle G-code\n\
             ; Pieces: {}\n\
             ; Complexity: {}\n\
             ; TODO: Implement actual puzzle piece cutting\n\
             M30 ; End program\n",
            self.cam.jigsaw_pieces, self.cam.jigsaw_complexity
        );
        self.gcode.gcode_content = gcode;
        self.gcode.gcode_filename = "jigsaw_puzzle.gcode".to_string();
        self.machine.status_message = "Jigsaw G-code generated (placeholder)".to_string();
    }
}
