//! G-code editor module.
//!
//! This module provides functionality for editing, searching, and optimizing
//! G-code files with syntax highlighting and post-processing capabilities.

use eframe::egui;
use egui::text::{LayoutJob, TextFormat};
use egui::text_edit::TextBuffer;
use std::path::PathBuf;

use crate::communication::CncController;
use crate::{MachinePosition, MoveType, PathSegment};

#[derive(Clone, Debug, Default)]
pub struct GcodeEditorState {
    pub gcode_content: String,
    pub gcode_filename: String,
    pub current_file_path: Option<PathBuf>,
    pub search_query: String,
    pub search_results: Vec<usize>,
    pub current_search_index: usize,
    pub selected_line: Option<usize>,
}

impl GcodeEditorState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load_gcode_file(&mut self) -> Result<(), String> {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("G-code files", &["gcode", "nc", "txt"])
            .pick_file()
        {
            match std::fs::read_to_string(&path) {
                Ok(content) => {
                    self.gcode_content = content;
                    self.current_file_path = Some(path.clone());
                    self.gcode_filename = path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    Ok(())
                }
                Err(e) => Err(format!("Error loading file: {}", e)),
            }
        } else {
            Err("No file selected".to_string())
        }
    }

    pub fn save_gcode_file(&mut self) -> Result<(), String> {
        if let Some(path) = &self.current_file_path {
            match std::fs::write(path, &self.gcode_content) {
                Ok(_) => Ok(()),
                Err(e) => Err(format!("Error saving file: {}", e)),
            }
        } else {
            self.save_gcode_file_as()
        }
    }

    pub fn save_gcode_file_as(&mut self) -> Result<(), String> {
        if self.gcode_content.is_empty() {
            return Err("No G-code to save".to_string());
        }

        if let Some(path) = rfd::FileDialog::new()
            .add_filter("G-code files", &["gcode", "nc", "txt"])
            .set_file_name(&self.gcode_filename)
            .save_file()
        {
            match std::fs::write(&path, &self.gcode_content) {
                Ok(_) => {
                    self.current_file_path = Some(path.clone());
                    self.gcode_filename = path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    Ok(())
                }
                Err(e) => Err(format!("Error saving file: {}", e)),
            }
        } else {
            Err("Save cancelled".to_string())
        }
    }

    pub fn optimize_gcode(&mut self) -> String {
        if self.gcode_content.is_empty() {
            return "No G-code to optimize".to_string();
        }

        let original_size = self.gcode_content.len();
        let original_lines = self.gcode_content.lines().count();
        let mut optimized_lines = Vec::new();
        let mut current_pos = MachinePosition::new(0.0, 0.0, 0.0);

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

            // Check if this is an arc command (G2/G3)
            if self.is_arc_command(line) {
                // Convert arc to line segments
                let arc_lines = self.convert_arc_to_lines(line, &current_pos);
                for arc_line in arc_lines {
                    optimized_lines.push(arc_line.clone());
                    // Update current position for next command
                    if let Some(new_pos) = self.extract_position_from_line(&arc_line) {
                        current_pos = new_pos;
                    }
                }
            } else {
                // Parse and optimize the regular G-code command
                let optimized_line = self.optimize_gcode_line(line);
                optimized_lines.push(optimized_line.clone());

                // Update current position
                if let Some(new_pos) = self.extract_position_from_line(&optimized_line) {
                    current_pos = new_pos;
                }
            }
        }

        self.gcode_content = optimized_lines.join("\n");

        let optimized_size = self.gcode_content.len();
        let optimized_line_count = optimized_lines.len();
        let size_reduction = if original_size > 0 && optimized_size <= original_size {
            ((original_size - optimized_size) as f32 / original_size as f32 * 100.0) as i32
        } else {
            0
        };

        format!(
            "G-code optimized: {} -> {} lines, {} -> {} bytes ({}% reduction)",
            original_lines, optimized_line_count, original_size, optimized_size, size_reduction
        )
    }

    fn optimize_gcode_line(&self, line: &str) -> String {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            return String::new();
        }

        let mut optimized_parts = Vec::new();

        for part in parts {
            if part.len() > 1 {
                let first_char = part.chars().next().unwrap();

                // Handle parameters with decimal truncation
                if first_char.is_ascii_alphabetic() && part.len() > 1
                    && let Ok(value) = part[1..].parse::<f32>() {
                        // Truncate to 3 decimal places for most parameters
                        let truncated = if first_char == 'F' {
                            // Feed rates: truncate to 1 decimal place
                            format!("{:.1}", value)
                        } else {
                            // Coordinates and other parameters: truncate to 3 decimal places
                            format!("{:.3}", value)
                        };

                        // Remove trailing zeros and decimal point if not needed
                        let clean_value = if truncated.contains('.') {
                            truncated
                                .trim_end_matches('0')
                                .trim_end_matches('.')
                                .to_string()
                        } else {
                            truncated
                        };

                        optimized_parts.push(format!("{}{}", first_char, clean_value));
                        continue;
                    }
            }

            // Keep other parts as-is
            optimized_parts.push(part.to_string());
        }

        optimized_parts.join(" ")
    }

    fn is_arc_command(&self, line: &str) -> bool {
        line.split_whitespace()
            .any(|part| part == "G2" || part == "G3")
    }

    fn convert_arc_to_lines(&self, arc_line: &str, current_pos: &MachinePosition) -> Vec<String> {
        let parts: Vec<&str> = arc_line.split_whitespace().collect();
        let mut params = std::collections::HashMap::new();

        // Parse parameters
        for part in &parts {
            if part.len() > 1 {
                let first_char = part.chars().next().unwrap();
                if first_char.is_ascii_alphabetic()
                    && let Ok(value) = part[1..].parse::<f32>() {
                        params.insert(first_char, value);
                    }
            }
        }

        // Extract arc parameters
        let end_x = params.get(&'X').copied().unwrap_or(current_pos.x);
        let end_y = params.get(&'Y').copied().unwrap_or(current_pos.y);
        let center_x = params.get(&'I').copied().unwrap_or(0.0);
        let center_y = params.get(&'J').copied().unwrap_or(0.0);
        let feed_rate = params.get(&'F').copied();

        // Determine arc direction (G2 = clockwise, G3 = counterclockwise)
        let is_clockwise = parts.contains(&"G2");

        // Calculate arc center in absolute coordinates
        let arc_center_x = current_pos.x + center_x;
        let arc_center_y = current_pos.y + center_y;

        // Calculate radius
        let radius = ((current_pos.x - arc_center_x).powi(2)
            + (current_pos.y - arc_center_y).powi(2))
        .sqrt();

        // Calculate start and end angles
        let start_angle = (current_pos.y - arc_center_y).atan2(current_pos.x - arc_center_x);
        let end_angle = (end_y - arc_center_y).atan2(end_x - arc_center_x);

        // Calculate angular difference
        let mut delta_angle = if is_clockwise {
            start_angle - end_angle
        } else {
            end_angle - start_angle
        };

        // Normalize angle to be between -π and π
        while delta_angle > std::f32::consts::PI {
            delta_angle -= 2.0 * std::f32::consts::PI;
        }
        while delta_angle < -std::f32::consts::PI {
            delta_angle += 2.0 * std::f32::consts::PI;
        }

        // For clockwise arcs, delta_angle should be negative
        if is_clockwise && delta_angle > 0.0 {
            delta_angle -= 2.0 * std::f32::consts::PI;
        } else if !is_clockwise && delta_angle < 0.0 {
            delta_angle += 2.0 * std::f32::consts::PI;
        }

        // Number of segments (adjust based on arc length and desired precision)
        let arc_length = radius * delta_angle.abs();
        let num_segments = (arc_length / 1.0).max(4.0).min(100.0) as usize; // At least 4 segments, max 100

        let angle_step = delta_angle / num_segments as f32;

        let mut lines = Vec::new();
        let mut current_angle = start_angle;

        for _i in 1..=num_segments {
            current_angle += angle_step;

            let x = arc_center_x + radius * current_angle.cos();
            let y = arc_center_y + radius * current_angle.sin();

            // Create G1 command
            let mut g1_parts = vec!["G1".to_string()];

            // Add coordinates with 3 decimal precision
            g1_parts.push(format!("X{:.3}", x));
            g1_parts.push(format!("Y{:.3}", y));

            // Add feed rate if present
            if let Some(f) = feed_rate {
                g1_parts.push(format!("F{:.1}", f));
            }

            lines.push(g1_parts.join(" "));
        }

        lines
    }

    fn extract_position_from_line(&self, line: &str) -> Option<MachinePosition> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        let mut pos = MachinePosition::new(0.0, 0.0, 0.0);

        for part in parts {
            if part.len() > 1 {
                let first_char = part.chars().next().unwrap();
                if let Ok(value) = part[1..].parse::<f32>() {
                    match first_char {
                        'X' => pos.x = value,
                        'Y' => pos.y = value,
                        'Z' => pos.z = value,
                        'A' => pos.a = Some(value),
                        'B' => pos.b = Some(value),
                        'C' => pos.c = Some(value),
                        'D' => pos.d = Some(value),
                        _ => {}
                    }
                }
            }
        }

        Some(pos)
    }

    pub fn parse_gcode(&self) -> Vec<PathSegment> {
        let mut parsed_paths = Vec::new();
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
                parsed_paths.push(PathSegment {
                    start: current_pos,
                    end: new_pos.clone(),
                    move_type: move_type.clone(),
                    line_number: line_idx,
                });
                current_pos = new_pos;
            }
            current_move_type = move_type;
        }

        parsed_paths
    }

    pub fn perform_search(&mut self) {
        if self.search_query.is_empty() {
            self.search_results.clear();
            self.current_search_index = 0;
            return;
        }

        self.search_results.clear();
        let query = self.search_query.to_lowercase();

        for (line_num, line) in self.gcode_content.lines().enumerate() {
            if line.to_lowercase().contains(&query) {
                self.search_results.push(line_num);
            }
        }

        self.current_search_index = 0;
        if !self.search_results.is_empty() {
            self.selected_line = Some(self.search_results[0]);
        }
    }

    pub fn search_next(&mut self) -> bool {
        if self.search_results.is_empty() {
            return false;
        }

        self.current_search_index = (self.current_search_index + 1) % self.search_results.len();
        self.selected_line = Some(self.search_results[self.current_search_index]);
        true
    }

    pub fn search_prev(&mut self) -> bool {
        if self.search_results.is_empty() {
            return false;
        }

        if self.current_search_index == 0 {
            self.current_search_index = self.search_results.len() - 1;
        } else {
            self.current_search_index -= 1;
        }
        self.selected_line = Some(self.search_results[self.current_search_index]);
        true
    }

    pub fn send_gcode_from_line(
        &self,
        start_line: usize,
        communication: &mut Box<dyn CncController>,
    ) -> Result<String, String> {
        if !communication.is_connected() {
            return Err("Not connected to device".to_string());
        }

        if self.gcode_content.is_empty() {
            return Err("No G-code loaded".to_string());
        }

        let lines: Vec<String> = self.gcode_content.lines().map(|s| s.to_string()).collect();
        if start_line >= lines.len() {
            return Err("Invalid line number".to_string());
        }

        let lines_to_send = &lines[start_line..];
        let mut sent_count = 0;

        for line in lines_to_send.iter() {
            let trimmed = line.trim();
            if !trimmed.is_empty() && !trimmed.starts_with(';') {
                match communication.send_gcode_line(trimmed) {
                    Ok(_) => {
                        sent_count += 1;
                        // Note: Job progress updates are handled in the main app
                    }
                    Err(e) => {
                        return Err(format!("Error sending line: {}", e));
                    }
                }
            }
        }

        Ok(format!(
            "Sent {} G-code lines from line {}",
            sent_count,
            start_line + 1
        ))
    }

    pub fn show_ui(
        &mut self,
        ui: &mut egui::Ui,
        _parsed_paths: &[PathSegment],
        postprocessor_manager: &mut crate::postprocessor::PostProcessorManager,
    ) -> Option<usize> {
        if self.gcode_content.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label("No G-code file loaded. Use 'Load File' in the left panel.");
            });
            return None;
        }

        // Search controls
        ui.horizontal(|ui| {
            ui.label("Search:");
            let search_response = ui.text_edit_singleline(&mut self.search_query);
            if ui.button("🔍 Find").clicked()
                || (search_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))
            {
                self.perform_search();
            }
            let _ = ui
                .add_enabled(
                    !self.search_results.is_empty(),
                    egui::Button::new("⬆️ Prev"),
                )
                .clicked()
                && self.search_prev();
            let _ = ui
                .add_enabled(
                    !self.search_results.is_empty(),
                    egui::Button::new("⬇️ Next"),
                )
                .clicked()
                && self.search_next();
            if !self.search_results.is_empty() {
                ui.label(format!(
                    "{}/{}",
                    self.current_search_index + 1,
                    self.search_results.len()
                ));
            }
        });

        let mut changed = false;

        // Post-processing controls
        ui.horizontal(|ui| {
            ui.label("Post-Processor:");

            // Controller selection dropdown
            let current_controller = postprocessor_manager.get_current_processor();
            let available_controllers = postprocessor_manager.get_available_processors();

            egui::ComboBox::from_label("")
                .selected_text(current_controller.as_str())
                .show_ui(ui, |ui| {
                    for controller in &available_controllers {
                        let selected = *controller == current_controller;
                        if ui.selectable_label(selected, controller.as_str()).clicked() {
                            postprocessor_manager.set_target_controller(*controller);
                        }
                    }
                });

            // Post-process button
            if ui.button("🔄 Post-Process").clicked() {
                // Parse current G-code
                let commands = crate::postprocessor::parse_gcode(&self.gcode_content);

                // Apply post-processing
                let processed_commands = postprocessor_manager.process_gcode(&commands);

                // Convert back to G-code string
                self.gcode_content = crate::postprocessor::commands_to_gcode(&processed_commands);

                // Re-parse for visualization
                changed = true;
            }

            // Show current processor info
            if let Some((name, description)) =
                postprocessor_manager.get_processor_info(current_controller)
            {
                ui.separator();
                ui.label(format!("{}: {}", name, description));
            }
        });

        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            let response = ui.add(
                egui::TextEdit::multiline(&mut self.gcode_content)
                    .font(egui::TextStyle::Monospace)
                    .desired_rows(20)
                    .layouter(&mut |ui: &egui::Ui, string: &dyn TextBuffer, _wrap_width| {
                        let mut job = LayoutJob::default();
                        let search_query_lower = self.search_query.to_lowercase();
                        let has_search =
                            !self.search_query.is_empty() && !self.search_results.is_empty();

                        // Helper function for syntax highlighting
                        let append_with_syntax_highlight = |job: &mut LayoutJob, text: &str| {
                            let words: Vec<&str> = text.split_whitespace().collect();
                            for (j, word) in words.iter().enumerate() {
                                let color = if word.starts_with('G')
                                    && word.len() > 1
                                    && word[1..].chars().all(|c| c.is_ascii_digit())
                                {
                                    egui::Color32::BLUE
                                } else if word.starts_with('M')
                                    && word.len() > 1
                                    && word[1..].chars().all(|c| c.is_ascii_digit())
                                {
                                    egui::Color32::GREEN
                                } else if word.starts_with('X')
                                    || word.starts_with('Y')
                                    || word.starts_with('Z')
                                    || word.starts_with('A')
                                    || word.starts_with('B')
                                    || word.starts_with('C')
                                    || word.starts_with('U')
                                    || word.starts_with('V')
                                    || word.starts_with('W')
                                    || word.starts_with('I')
                                    || word.starts_with('J')
                                    || word.starts_with('K')
                                    || word.starts_with('R')
                                    || word.starts_with('F')
                                    || word.starts_with('S')
                                    || word.starts_with('T')
                                    || word.starts_with('H')
                                    || word.starts_with('D')
                                    || word.starts_with('P')
                                    || word.starts_with('Q')
                                    || word.starts_with('L')
                                    || word.starts_with('N')
                                    || word.starts_with('O')
                                {
                                    // Check if parameter has valid numeric value
                                    if word.len() > 1 {
                                        let value_part = &word[1..];
                                        if value_part.parse::<f32>().is_err()
                                            && !value_part.contains('.')
                                            && !value_part.starts_with('-')
                                        {
                                            egui::Color32::from_rgb(255, 165, 0) // Orange for invalid parameter values
                                        } else {
                                            egui::Color32::RED
                                        }
                                    } else {
                                        egui::Color32::RED
                                    }
                                } else if word.starts_with(';') {
                                    egui::Color32::GRAY
                                } else if word
                                    .chars()
                                    .next()
                                    .is_some_and(|c| c.is_ascii_alphabetic())
                                {
                                    // Word starts with letter but not a recognized command or parameter
                                    egui::Color32::from_rgb(255, 165, 0) // Orange for unrecognized commands
                                } else {
                                    egui::Color32::BLACK
                                };
                                job.append(
                                    word,
                                    0.0,
                                    TextFormat {
                                        font_id: egui::FontId::monospace(12.0),
                                        color,
                                        ..Default::default()
                                    },
                                );
                                if j < words.len() - 1 {
                                    job.append(" ", 0.0, TextFormat::default());
                                }
                            }
                        };

                        for (i, line) in string.as_str().lines().enumerate() {
                            // Line number
                            job.append(
                                &format!("{:05}: ", i + 1),
                                0.0,
                                TextFormat {
                                    font_id: egui::FontId::monospace(12.0),
                                    color: egui::Color32::DARK_GRAY,
                                    ..Default::default()
                                },
                            );

                            // Check if this line is the current search result
                            let is_current_search_line = has_search
                                && !self.search_results.is_empty()
                                && i == self.search_results[self.current_search_index];

                            if is_current_search_line {
                                // Process line with current search result highlighting
                                let line_lower = line.to_lowercase();
                                let mut pos = 0;
                                while let Some(match_start) =
                                    line_lower[pos..].find(&search_query_lower)
                                {
                                    let match_start = pos + match_start;
                                    let match_end = match_start + search_query_lower.len();

                                    // Add text before match with normal syntax highlighting
                                    if match_start > pos {
                                        append_with_syntax_highlight(
                                            &mut job,
                                            &line[pos..match_start],
                                        );
                                    }

                                    // Add matched text with yellow background (current search result)
                                    job.append(
                                        &line[match_start..match_end],
                                        0.0,
                                        TextFormat {
                                            font_id: egui::FontId::monospace(12.0),
                                            color: egui::Color32::BLACK,
                                            background: egui::Color32::YELLOW,
                                            ..Default::default()
                                        },
                                    );

                                    pos = match_end;
                                }

                                // Add remaining text
                                if pos < line.len() {
                                    append_with_syntax_highlight(&mut job, &line[pos..]);
                                }
                            } else if has_search
                                && line.to_lowercase().contains(&search_query_lower)
                            {
                                // Line has search matches but is not current result - highlight with lighter color
                                let line_lower = line.to_lowercase();
                                let mut pos = 0;
                                while let Some(match_start) =
                                    line_lower[pos..].find(&search_query_lower)
                                {
                                    let match_start = pos + match_start;
                                    let match_end = match_start + search_query_lower.len();

                                    // Add text before match with normal syntax highlighting
                                    if match_start > pos {
                                        append_with_syntax_highlight(
                                            &mut job,
                                            &line[pos..match_start],
                                        );
                                    }

                                    // Add matched text with light yellow background (other search results)
                                    job.append(
                                        &line[match_start..match_end],
                                        0.0,
                                        TextFormat {
                                            font_id: egui::FontId::monospace(12.0),
                                            color: egui::Color32::BLACK,
                                            background: egui::Color32::from_rgb(255, 255, 200), // Light yellow
                                            ..Default::default()
                                        },
                                    );

                                    pos = match_end;
                                }

                                // Add remaining text
                                if pos < line.len() {
                                    append_with_syntax_highlight(&mut job, &line[pos..]);
                                }
                            } else {
                                // No search match, use normal syntax highlighting
                                append_with_syntax_highlight(&mut job, line);
                            }

                            job.append("\n", 0.0, TextFormat::default());
                        }
                        ui.fonts_mut(|fonts| fonts.layout_job(job))
                    }),
            );
            if response.changed() {
                changed = true;
            }
        });

        if changed {
            Some(0) // Signal that parsing should be updated
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gcode_editor_new() {
        let editor = GcodeEditorState::new();
        assert!(editor.gcode_content.is_empty());
        assert!(editor.gcode_filename.is_empty());
        assert!(editor.current_file_path.is_none());
        assert!(editor.search_query.is_empty());
        assert!(editor.search_results.is_empty());
        assert_eq!(editor.current_search_index, 0);
        assert!(editor.selected_line.is_none());
    }

    #[test]
    fn test_parse_empty_gcode() {
        let editor = GcodeEditorState {
            gcode_content: String::new(),
            ..Default::default()
        };
        let paths = editor.parse_gcode();
        assert!(paths.is_empty());
    }

    #[test]
    fn test_parse_simple_gcode() {
        let editor = GcodeEditorState {
            gcode_content: "G0 X10 Y20\nG1 X30 Y40 F100".to_string(),
            ..Default::default()
        };
        let paths = editor.parse_gcode();

        assert_eq!(paths.len(), 2);

        // First segment: G0 X10 Y20
        assert_eq!(paths[0].start.x, 0.0);
        assert_eq!(paths[0].start.y, 0.0);
        assert_eq!(paths[0].end.x, 10.0);
        assert_eq!(paths[0].end.y, 20.0);
        assert_eq!(paths[0].move_type, crate::MoveType::Rapid);
        assert_eq!(paths[0].line_number, 0);

        // Second segment: G1 X30 Y40 F100
        assert_eq!(paths[1].start.x, 10.0);
        assert_eq!(paths[1].start.y, 20.0);
        assert_eq!(paths[1].end.x, 30.0);
        assert_eq!(paths[1].end.y, 40.0);
        assert_eq!(paths[1].move_type, crate::MoveType::Feed);
        assert_eq!(paths[1].line_number, 1);
    }

    #[test]
    fn test_parse_gcode_with_comments() {
        let editor = GcodeEditorState {
            gcode_content:
                "; This is a comment\nG0 X10 Y20 ; inline comment\n; Another comment\nG1 X30 Y40"
                    .to_string(),
            ..Default::default()
        };
        let paths = editor.parse_gcode();

        assert_eq!(paths.len(), 2);
        assert_eq!(paths[0].line_number, 1); // Line 1 (0-indexed)
        assert_eq!(paths[1].line_number, 3); // Line 3 (0-indexed)
    }

    #[test]
    fn test_parse_gcode_with_arcs() {
        let editor = GcodeEditorState {
            gcode_content: "G0 X10 Y10\nG2 X20 Y20 I5 J5\nG3 X30 Y30 I10 J10".to_string(),
            ..Default::default()
        };
        let paths = editor.parse_gcode();

        assert_eq!(paths.len(), 3);
        assert_eq!(paths[0].move_type, crate::MoveType::Rapid);
        assert_eq!(paths[1].move_type, crate::MoveType::Arc);
        assert_eq!(paths[2].move_type, crate::MoveType::Arc);
    }

    #[test]
    fn test_parse_gcode_multiple_axes() {
        let editor = GcodeEditorState {
            gcode_content: "G0 X10 Y20 Z30 A45 B90".to_string(),
            ..Default::default()
        };
        let paths = editor.parse_gcode();

        assert_eq!(paths.len(), 1);
        let segment = &paths[0];
        assert_eq!(segment.end.x, 10.0);
        assert_eq!(segment.end.y, 20.0);
        assert_eq!(segment.end.z, 30.0);
        assert_eq!(segment.end.a, Some(45.0));
        assert_eq!(segment.end.b, Some(90.0));
    }

    #[test]
    fn test_optimize_gcode_empty() {
        let mut editor = GcodeEditorState::new();
        let result = editor.optimize_gcode();
        assert_eq!(result, "No G-code to optimize");
    }

    #[test]
    fn test_optimize_gcode_with_comments() {
        let mut editor = GcodeEditorState {
            gcode_content: "; Header comment\nG0 X10 ; move to start\n; inline comment\nG1 X20 Y30 F100\n; footer comment".to_string(),
            ..Default::default()
        };

        let result = editor.optimize_gcode();
        assert_eq!(
            result,
            "G-code optimized: 5 -> 2 lines, 89 -> 22 bytes (75% reduction)"
        );
        assert_eq!(editor.gcode_content, "G0 X10\nG1 X20 Y30 F100");
    }

    #[test]
    fn test_optimize_gcode_remove_empty_lines() {
        let mut editor = GcodeEditorState {
            gcode_content: "G0 X10\n\nG1 X20\n  \nG1 X40 Y50".to_string(),
            ..Default::default()
        };

        let result = editor.optimize_gcode();
        assert_eq!(
            result,
            "G-code optimized: 5 -> 3 lines, 33 -> 29 bytes (12% reduction)"
        );
        assert_eq!(editor.gcode_content, "G0 X10\nG1 X20\nG1 X40 Y50");
    }

    #[test]
    fn test_search_empty_query() {
        let mut editor = GcodeEditorState {
            gcode_content: "G0 X10\nG1 X20".to_string(),
            ..Default::default()
        };

        editor.search_query = String::new();
        editor.perform_search();

        assert!(editor.search_results.is_empty());
        assert_eq!(editor.current_search_index, 0);
        assert!(editor.selected_line.is_none());
    }

    #[test]
    fn test_search_single_result() {
        let mut editor = GcodeEditorState {
            gcode_content: "G0 X10\nG1 X20 Y30\nG2 X40 Y50 I10 J10".to_string(),
            ..Default::default()
        };

        editor.search_query = "X20".to_string();
        editor.perform_search();

        assert_eq!(editor.search_results.len(), 1);
        assert_eq!(editor.search_results[0], 1);
        assert_eq!(editor.current_search_index, 0);
        assert_eq!(editor.selected_line, Some(1));
    }

    #[test]
    fn test_search_multiple_results() {
        let mut editor = GcodeEditorState {
            gcode_content: "G0 X10\nG1 X20\nG2 X30".to_string(),
            ..Default::default()
        };

        editor.search_query = "X".to_string();
        editor.perform_search();

        assert_eq!(editor.search_results.len(), 3);
        assert_eq!(editor.search_results, vec![0, 1, 2]);
        assert_eq!(editor.selected_line, Some(0));
    }

    #[test]
    fn test_search_case_insensitive() {
        let mut editor = GcodeEditorState {
            gcode_content: "g0 x10\nG1 X20".to_string(),
            ..Default::default()
        };

        editor.search_query = "G0".to_string();
        editor.perform_search();

        assert_eq!(editor.search_results.len(), 1);
        assert_eq!(editor.search_results[0], 0);
    }

    #[test]
    fn test_search_next() {
        let mut editor = GcodeEditorState {
            gcode_content: "G0 X10\nG1 X20\nG2 X30".to_string(),
            search_results: vec![0, 1, 2],
            current_search_index: 0,
            ..Default::default()
        };

        assert!(editor.search_next());
        assert_eq!(editor.current_search_index, 1);
        assert_eq!(editor.selected_line, Some(1));

        assert!(editor.search_next());
        assert_eq!(editor.current_search_index, 2);
        assert_eq!(editor.selected_line, Some(2));

        assert!(editor.search_next()); // Wrap around
        assert_eq!(editor.current_search_index, 0);
        assert_eq!(editor.selected_line, Some(0));
    }

    #[test]
    fn test_search_prev() {
        let mut editor = GcodeEditorState {
            gcode_content: "G0 X10\nG1 X20\nG2 X30".to_string(),
            search_results: vec![0, 1, 2],
            current_search_index: 2,
            ..Default::default()
        };

        assert!(editor.search_prev());
        assert_eq!(editor.current_search_index, 1);
        assert_eq!(editor.selected_line, Some(1));

        assert!(editor.search_prev());
        assert_eq!(editor.current_search_index, 0);
        assert_eq!(editor.selected_line, Some(0));

        assert!(editor.search_prev()); // Wrap around
        assert_eq!(editor.current_search_index, 2);
        assert_eq!(editor.selected_line, Some(2));
    }

    #[test]
    fn test_search_next_empty_results() {
        let mut editor = GcodeEditorState::new();
        assert!(!editor.search_next());
    }
}
