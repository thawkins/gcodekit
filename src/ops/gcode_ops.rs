use crate::GcodeKitApp;

impl GcodeKitApp {
    /// Helper function to sync G-code content to the enhanced editor
    pub(crate) fn sync_gcode_to_editor(&mut self) {
        let content = self.gcode.gcode_content.clone();
        self.gcode_editor.buffer.set_content(&content);
        self.gcode_editor.gcode_content = content;
        self.gcode_editor.gcode_filename = self.gcode.gcode_filename.clone();
        self.gcode_editor.on_buffer_change();
        self.gcode_editor.selected_line = Some(0);
        self.gcode_editor.virtualized_state = Default::default();
        // Parse gcode and store in editor
        self.gcode_editor.parsed_paths = crate::gcode::parse_gcode(&self.gcode.gcode_content);
    }

    /// Parses the currently loaded G-code content and extracts path segments.
    /// Identifies move commands (G0, G1, G2, G3) and creates PathSegment objects
    /// for visualization and analysis. Handles absolute/incremental positioning modes.
    pub fn parse_gcode(&mut self) {
        self.gcode_editor.parsed_paths = crate::gcode::parse_gcode(&self.gcode.gcode_content);
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
        self.sync_gcode_to_editor();
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
        self.sync_gcode_to_editor();
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
        self.sync_gcode_to_editor();
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
            self.sync_gcode_to_editor();
            self.parse_gcode();
            self.machine.status_message = "Toolpath parameters added".to_string();
        } else {
            self.machine.status_message = "No G-code to modify".to_string();
        }
    }

    /// Initiates sending the currently loaded G-code to the connected device.
    /// Currently a placeholder - full implementation with queuing is TODO.
    pub fn send_gcode(&mut self, content: &str) {
        self.log_console(&format!(
            "send_gcode: Called with content length = {}, contains_newlines = {}",
            content.len(),
            content.contains('\n')
        ));
        if !content.is_empty() {
            let preview = if content.len() > 50 {
                &content[..50]
            } else {
                content
            };
            self.log_console(&format!("send_gcode: Content preview: {:?}", preview));
        }
        self.machine.status_message = "Sending G-code to device...".to_string();
        if content.contains('\n') {
            // Multi-line content (file)
            self.log_console(
                "send_gcode: Multi-line content detected, calling send_gcode_to_device",
            );
            self.gcode.gcode_content = content.to_string();
            self.sync_gcode_to_editor();
            self.send_gcode_to_device();
        } else {
            // Single command
            self.log_console(&format!(
                "send_gcode: Single command '{}' detected",
                content
            ));
            if let Err(e) = self.machine.communication.send_gcode_line(content) {
                self.machine.status_message = format!("Error sending G-code: {}", e);
            } else {
                self.machine.status_message = "G-code command sent".to_string();
            }
        }
    }

    /// Stops sending G-code to the device and resets sending state.
    pub fn stop_sending_gcode(&mut self) {
        self.log_console("stop_sending_gcode: Stopping G-code transmission");
        
        // Send emergency stop to device
        self.machine.communication.emergency_stop();
        
        // Reset sending flags
        self.gcode.is_sending = false;
        self.gcode.current_line_sending = 0;
        
        self.machine.status_message = "G-code transmission stopped".to_string();
        self.log_console("stop_sending_gcode: G-code transmission halted");
    }

    fn send_gcode_to_device(&mut self) {
        self.log_console("send_gcode_to_device: Starting G-code send process");

        let connection_state = self.machine.communication.get_connection_state().clone();
        self.log_console(&format!(
            "send_gcode_to_device: Connection state = {:?}",
            connection_state
        ));
        self.log_console(&format!(
            "send_gcode_to_device: G-code content length = {}",
            self.gcode.gcode_content.len()
        ));

        if connection_state != crate::communication::ConnectionState::Connected {
            self.machine.status_message = "Not connected to device".to_string();
            self.log_console("send_gcode_to_device: Not connected to device, aborting");
            return;
        }

        if self.gcode.gcode_content.is_empty() {
            self.machine.status_message = "No G-code loaded".to_string();
            self.log_console("send_gcode_to_device: No G-code loaded, aborting");
            return;
        }

        self.log_console(&format!(
            "send_gcode_to_device: G-code content length = {}",
            self.gcode.gcode_content.len()
        ));

        // Mark as sending
        self.gcode.is_sending = true;

        // Send each line to the device sequentially with delay, like gcode-send
        let content = self.gcode.gcode_content.clone(); // Clone to avoid borrowing issues
        let lines: Vec<&str> = content.lines().collect();
        let mut sent_count = 0;
        let error_count = 0;

        self.log_console(&format!(
            "send_gcode_to_device: Total lines = {}",
            lines.len()
        ));

        // Reset progress
        self.gcode_editor.sending_progress = 0.0;
        self.gcode.current_line_sending = 0;

        for (line_idx, line) in lines.iter().enumerate() {
            // Check if we should stop sending
            if !self.gcode.is_sending {
                self.log_console("send_gcode_to_device: Transmission stopped by user");
                break;
            }

            let trimmed = line.trim();
            if !trimmed.is_empty() && !trimmed.starts_with(';') {
                // Remove comments (after ; or ( )
                let command = trimmed
                    .split(';')
                    .next()
                    .unwrap_or(trimmed)
                    .split('(')
                    .next()
                    .unwrap_or(trimmed)
                    .trim();

                if !command.is_empty() {
                    self.log_console(&format!(
                        "send_gcode_to_device: Sending command {}: '{}'",
                        line_idx + 1,
                        command
                    ));
                    // Send directly without queuing, like gcode-send
                    self.machine
                        .communication
                        .send_raw_command(&format!("{}\r\n", command));
                    sent_count += 1;
                    self.gcode.current_line_sending = line_idx + 1;
                }
            }

            // Update progress
            self.gcode_editor.sending_progress = ((line_idx + 1) as f32) / (lines.len() as f32);

            // Small delay between commands
            std::thread::sleep(std::time::Duration::from_millis(50));
        }

        // Mark as finished sending
        self.gcode.is_sending = false;

        // Report final status
        if error_count == 0 {
            self.machine.status_message = format!(
                "G-code queued successfully ({} commands sent sequentially)",
                sent_count
            );
            self.gcode_editor.sending_progress = 1.0;
        } else if sent_count > 0 {
            self.machine.status_message = format!(
                "G-code partially queued: {} commands sent, {} errors. Check console for details.",
                sent_count, error_count
            );
            self.gcode_editor.sending_progress =
                (sent_count as f32) / (sent_count + error_count) as f32;
        } else {
            self.machine.status_message = format!(
                "Failed to send any G-code commands ({} errors)",
                error_count
            );
            self.gcode_editor.sending_progress = 0.0;
        }
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
                        if let Some(job_id) = &self.job.current_job_id {
                            if let Some(job) = self.job.job_queue.get_job_mut(job_id) {
                                let current_line = start_line + i;
                                job.last_completed_line = Some(current_line);
                                job.update_progress((current_line as f32) / (lines.len() as f32));
                            }
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

        self.gcode_editor.sending_from_line = Some(start_line);
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

    /// Generates G-code for image engraving with grayscale intensity mapping.
    /// Converts loaded bitmap to G-code using vectorization and intensity-based power control.
    pub fn generate_image_engraving(&mut self) {
        self.log_console("generate_image_engraving: Starting image to G-code conversion");

        // Check if image is loaded
        if self.cam.image_path.is_none() {
            self.machine.status_message = "No image loaded. Click 'Load Image' first.".to_string();
            self.log_console("generate_image_engraving: No image loaded, aborting");
            return;
        }

        let image_path = self.cam.image_path.as_ref().unwrap().clone();

        // Load the image
        match image::open(&image_path) {
            Ok(img) => {
                self.log_console(&format!(
                    "generate_image_engraving: Loaded image {}x{}",
                    img.width(),
                    img.height()
                ));

                // Convert to grayscale
                let gray = img.to_luma8();

                // Vectorize using bitmap processor
                let contours = crate::designer::bitmap_processing::BitmapProcessor::vectorize_bitmap(
                    &gray,
                    &self.cam.vectorization_config,
                );

                self.log_console(&format!(
                    "generate_image_engraving: Generated {} contours",
                    contours.len()
                ));

                // Convert contours to G-code
                let gcode = self.contours_to_gcode(&contours, &image_path);
                
                self.gcode.gcode_content = gcode;
                self.gcode.gcode_filename = "image_engraving.gcode".to_string();
                self.sync_gcode_to_editor();
                
                self.machine.status_message = format!(
                    "Image engraving G-code generated ({} contours)",
                    contours.len()
                );
                self.log_console("generate_image_engraving: Conversion complete");
            }
            Err(e) => {
                self.machine.status_message = format!("Error loading image: {}", e);
                self.log_console(&format!("generate_image_engraving: Error: {}", e));
            }
        }
    }

    /// Converts vectorized contours to G-code for laser engraving.
    /// Uses raster scanning with intensity-based power control for grayscale effects.
    fn contours_to_gcode(&self, contours: &[Vec<(f32, f32)>], image_path: &str) -> String {
        let mut gcode = String::new();

        // Header
        gcode.push_str("; Image Engraving G-code\n");
        gcode.push_str(&format!("; Source: {}\n", image_path));
        gcode.push_str(&format!("; Resolution: {} dpi\n", self.cam.image_resolution));
        gcode.push_str(&format!("; Max Power: {}%\n", self.cam.image_max_power));
        gcode.push_str(&format!("; Contours: {}\n", contours.len()));
        gcode.push_str("; Generated by gcodekit\n\n");

        // Machine setup
        gcode.push_str("G90 ; Absolute positioning\n");
        gcode.push_str("G21 ; Metric units\n");
        gcode.push_str("M3 S0 ; Spindle/laser off\n");
        gcode.push_str("\n");

        // Calculate scaling factors
        let mm_per_inch = 25.4;
        let scale = mm_per_inch / self.cam.image_resolution;

        // Process each contour
        for (contour_idx, contour) in contours.iter().enumerate() {
            if contour.len() < 2 {
                continue;
            }

            gcode.push_str(&format!("; Contour {}\n", contour_idx + 1));

            // Move to start of contour
            let first = contour[0];
            let x = first.0 * scale;
            let y = first.1 * scale;
            gcode.push_str(&format!("G0 X{:.3} Y{:.3} ; Move to start\n", x, y));

            // Engrave the contour
            gcode.push_str("M3 S1000 ; Laser on\n");
            
            for point in contour.iter().skip(1) {
                let x = point.0 * scale;
                let y = point.1 * scale;
                let feed = self.cam.tool_feed_rate.max(10.0).min(1000.0);
                gcode.push_str(&format!("G1 X{:.3} Y{:.3} F{:.0}\n", x, y, feed));
            }

            gcode.push_str("M5 ; Laser off\n\n");
        }

        // End program
        gcode.push_str("M5 ; Laser off\n");
        gcode.push_str("G0 X0 Y0 ; Move to origin\n");
        gcode.push_str("M30 ; End program\n");

        gcode
    }

    /// Generates G-code for cutting a tabbed box with interlocking tabs.
    /// Creates flat pattern with tabs for assembly into 3D box.
    pub fn generate_tabbed_box(&mut self) {
        self.log_console("generate_tabbed_box: Starting tabbed box generation");

        let length = self.cam.box_length.max(10.0);
        let width = self.cam.box_width.max(10.0);
        let height = self.cam.box_height.max(10.0);
        let tab_size = self.cam.tab_size.max(2.0).min(length / 4.0);
        let feed = self.cam.tool_feed_rate.max(10.0).min(1000.0);

        let mut gcode = String::new();
        gcode.push_str("; Tabbed Box G-code\n");
        gcode.push_str(&format!("; Dimensions: {}x{}x{}mm\n", length, width, height));
        gcode.push_str(&format!("; Tab size: {}mm\n", tab_size));
        gcode.push_str("; Generated by gcodekit\n\n");

        gcode.push_str("G90 ; Absolute positioning\n");
        gcode.push_str("G21 ; Metric units\n");
        gcode.push_str("M3 S10000 ; Spindle on\n\n");

        // Generate base pattern with 6 faces
        let faces = vec![
            ("Front (length x height)", 0.0, 0.0, length, height),
            ("Back (length x height)", length + 20.0, 0.0, length, height),
            ("Left (width x height)", 0.0, height + 20.0, width, height),
            ("Right (width x height)", width + 20.0, height + 20.0, width, height),
            ("Bottom (length x width)", 0.0, height * 2.0 + 40.0, length, width),
            ("Top (length x width)", length + 20.0, height * 2.0 + 40.0, length, width),
        ];

        for (name, start_x, start_y, face_length, face_width) in faces {
            gcode.push_str(&format!("; --- {} ---\n", name));
            gcode.push_str(&self.generate_box_face(
                start_x, start_y, face_length, face_width, tab_size, feed,
            ));
        }

        gcode.push_str("\nM5 ; Spindle off\n");
        gcode.push_str("G0 X0 Y0 ; Move to origin\n");
        gcode.push_str("M30 ; End program\n");

        self.gcode.gcode_content = gcode;
        self.gcode.gcode_filename = "tabbed_box.gcode".to_string();
        self.sync_gcode_to_editor();
        self.machine.status_message = "Tabbed box G-code generated".to_string();
        self.log_console("generate_tabbed_box: Generation complete");
    }

    /// Generate a single box face with tabs
    fn generate_box_face(&self, x: f32, y: f32, length: f32, width: f32, tab_size: f32, feed: f32) -> String {
        let mut gcode = String::new();

        // Move to start corner
        gcode.push_str(&format!("G0 X{:.3} Y{:.3}\n", x, y));
        gcode.push_str("G1 Z-1 F100 ; Plunge\n");

        // Top edge with tabs
        let num_tabs = ((length / tab_size) / 2.0).ceil() as i32;
        for i in 0..num_tabs * 2 {
            let tab_x = x + (i as f32 * tab_size / 2.0);
            if i % 2 == 0 {
                // Tab (raised)
                gcode.push_str(&format!("G1 X{:.3} Y{:.3} F{:.0}\n", tab_x, y, feed));
            } else {
                // Gap (cut)
                gcode.push_str(&format!("G1 X{:.3} Y{:.3} F{:.0}\n", tab_x, y - tab_size / 3.0, feed));
            }
        }

        // Complete the rectangle
        gcode.push_str(&format!("G1 X{:.3} Y{:.3} F{:.0}\n", x + length, y, feed));
        gcode.push_str(&format!("G1 X{:.3} Y{:.3} F{:.0}\n", x + length, y + width, feed));
        gcode.push_str(&format!("G1 X{:.3} Y{:.3} F{:.0}\n", x, y + width, feed));
        gcode.push_str(&format!("G1 X{:.3} Y{:.3} F{:.0}\n", x, y, feed));

        gcode.push_str("G0 Z0 ; Retract\n\n");
        gcode
    }

    /// Generates G-code for cutting jigsaw puzzle pieces.
    /// Creates interlocking puzzle pieces with wave pattern edges.
    pub fn generate_jigsaw(&mut self) {
        self.log_console("generate_jigsaw: Starting jigsaw generation");

        let pieces = (self.cam.jigsaw_pieces as f32).max(4.0).sqrt() as i32;
        let complexity = (self.cam.jigsaw_complexity as f32).max(1.0).min(5.0) as i32;
        
        // Material dimensions
        let material_width = 100.0;
        let material_height = 100.0;
        let piece_width = material_width / pieces as f32;
        let piece_height = material_height / pieces as f32;
        let wave_height = piece_width / (complexity as f32 + 2.0);
        let feed = self.cam.tool_feed_rate.max(10.0).min(1000.0);

        let mut gcode = String::new();
        gcode.push_str("; Jigsaw Puzzle G-code\n");
        gcode.push_str(&format!("; Grid: {}x{} ({} pieces)\n", pieces, pieces, pieces * pieces));
        gcode.push_str(&format!("; Complexity: {}\n", complexity));
        gcode.push_str("; Generated by gcodekit\n\n");

        gcode.push_str("G90 ; Absolute positioning\n");
        gcode.push_str("G21 ; Metric units\n");
        gcode.push_str("M3 S12000 ; Spindle on\n\n");

        // Generate puzzle pieces
        for row in 0..pieces {
            for col in 0..pieces {
                let piece_x = col as f32 * piece_width;
                let piece_y = row as f32 * piece_height;
                
                gcode.push_str(&format!("; Piece ({},{})\n", col, row));
                gcode.push_str(&self.generate_puzzle_piece(
                    piece_x, piece_y, piece_width, piece_height, wave_height, 
                    complexity, feed, col, row, pieces
                ));
            }
        }

        gcode.push_str("\nM5 ; Spindle off\n");
        gcode.push_str("G0 X0 Y0 ; Move to origin\n");
        gcode.push_str("M30 ; End program\n");

        self.gcode.gcode_content = gcode;
        self.gcode.gcode_filename = "jigsaw_puzzle.gcode".to_string();
        self.sync_gcode_to_editor();
        self.machine.status_message = format!(
            "Jigsaw G-code generated ({}x{} grid, {} pieces)", 
            pieces, pieces, pieces * pieces
        );
        self.log_console("generate_jigsaw: Generation complete");
    }

    /// Generate a single jigsaw puzzle piece with interlocking edges
    fn generate_puzzle_piece(
        &self, x: f32, y: f32, width: f32, height: f32, wave_height: f32,
        complexity: i32, feed: f32, col: i32, row: i32, grid_size: i32
    ) -> String {
        let mut gcode = String::new();

        gcode.push_str(&format!("G0 X{:.3} Y{:.3}\n", x, y));
        gcode.push_str("G1 Z-1 F100 ; Plunge\n");

        // Bottom edge (may have interlocks)
        let bottom_has_lock = row < grid_size - 1;
        gcode.push_str(&self.generate_puzzle_edge(x, y, x + width, y, wave_height, complexity, feed, bottom_has_lock));

        // Right edge
        let right_has_lock = col < grid_size - 1;
        gcode.push_str(&self.generate_puzzle_edge(x + width, y, x + width, y + height, wave_height, complexity, feed, right_has_lock));

        // Top edge
        let top_has_lock = row > 0;
        gcode.push_str(&self.generate_puzzle_edge(x + width, y + height, x, y + height, wave_height, complexity, feed, top_has_lock));

        // Left edge
        let left_has_lock = col > 0;
        gcode.push_str(&self.generate_puzzle_edge(x, y + height, x, y, wave_height, complexity, feed, left_has_lock));

        gcode.push_str("G0 Z0 ; Retract\n\n");
        gcode
    }

    /// Generate a puzzle edge with optional wave pattern
    fn generate_puzzle_edge(
        &self, x1: f32, y1: f32, x2: f32, y2: f32, wave_height: f32, 
        complexity: i32, feed: f32, has_lock: bool
    ) -> String {
        let mut gcode = String::new();

        let dx = x2 - x1;
        let dy = y2 - y1;
        let edge_length = (dx * dx + dy * dy).sqrt();
        let steps = (edge_length / 5.0).max(4.0) as i32;
        let step_size = edge_length / steps as f32;

        if !has_lock {
            // Straight edge
            gcode.push_str(&format!("G1 X{:.3} Y{:.3} F{:.0}\n", x2, y2, feed));
        } else {
            // Wavy edge with bumps for interlocking
            for i in 0..=steps {
                let t = i as f32 / steps as f32;
                let base_x = x1 + dx * t;
                let base_y = y1 + dy * t;

                // Add perpendicular offset (wave)
                let perp_factor = if i % 2 == 0 { 1.0 } else { -1.0 };
                let offset = wave_height * perp_factor * (std::f32::consts::PI * i as f32 / steps as f32).sin();

                // Perpendicular direction
                let perp_x = -dy.signum() * offset;
                let perp_y = dx.signum() * offset;

                gcode.push_str(&format!("G1 X{:.3} Y{:.3} F{:.0}\n", base_x + perp_x, base_y + perp_y, feed));
            }
        }

        gcode
    }
}
