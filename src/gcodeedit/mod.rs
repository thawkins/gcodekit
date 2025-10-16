// G-code editor module.
pub mod autocomplete;
pub mod config;
pub mod editor;
pub mod find_replace;
pub mod rules;
pub mod tokenizer;
pub mod virtualized_view;
pub mod vocabulary;

// Re-export main types
pub use editor::TextBufferCore;
pub use rules::{Diagnostic, RuleSet, Severity};
pub use tokenizer::{LineSyntax, TokenizerService};

// This module provides functionality for editing, searching, and optimizing
// G-code files with syntax highlighting and post-processing capabilities.

use eframe::egui;
use egui::text::{LayoutJob, TextFormat};
use egui::text_edit::TextBuffer;
use std::path::PathBuf;

use crate::communication::CncController;
use crate::types::{MachinePosition, MoveType, PathSegment};

#[derive(Clone, Debug)]
pub struct GcodeEditorState {
    pub gcode_content: String,
    pub buffer: crate::gcodeedit::editor::TextBufferCore,
    pub tokenizer: crate::gcodeedit::tokenizer::TokenizerService,
    pub last_parsed: Vec<crate::gcodeedit::tokenizer::LineSyntax>,
    pub gcode_filename: String,
    pub current_file_path: Option<PathBuf>,
    pub search_query: String,
    pub search_results: Vec<usize>,
    pub current_search_index: usize,
    pub selected_line: Option<usize>,
    // Validation ruleset and diagnostics
    pub rules: crate::gcodeedit::rules::RuleSet,
    pub diagnostics: Vec<crate::gcodeedit::rules::Diagnostic>,
    /// Content version for tracking changes and cache invalidation
    content_version: u64,
    // Auto-completion state
    pub autocomplete: crate::gcodeedit::autocomplete::AutoCompleter,
    pub show_autocomplete: bool,
    pub autocomplete_suggestions: Vec<crate::gcodeedit::autocomplete::CompletionItem>,
    pub autocomplete_selected: usize,
    // Virtualization and performance
    pub virtualized_state: crate::gcodeedit::virtualized_view::VirtualizedState,
    pub virtualized_config: crate::gcodeedit::virtualized_view::VirtualizedConfig,
    pub fold_manager: crate::gcodeedit::virtualized_view::FoldManager,
    pub performance_metrics: crate::gcodeedit::virtualized_view::PerformanceMetrics,
    pub enable_virtualization: bool,
    // Find and Replace
    pub find_replace: crate::gcodeedit::find_replace::FindReplace,
    pub show_find_replace: bool,
    pub show_replace: bool,
    // Help and shortcuts
    pub show_shortcuts_help: bool,
    // Visualization and sending state (moved from GcodeState)
    pub parsed_paths: Vec<PathSegment>,
    pub sending_from_line: Option<usize>,
    pub sending_progress: f32, // 0.0 to 1.0, progress of current send operation
    // Cached galley row height for gutter alignment
    pub(crate) cached_row_height: Option<f32>,
}

impl Default for GcodeEditorState {
    fn default() -> Self {
        Self {
            gcode_content: String::new(),
            buffer: crate::gcodeedit::editor::TextBufferCore::new(),
            tokenizer: crate::gcodeedit::tokenizer::TokenizerService::new(100),
            last_parsed: Vec::new(),
            gcode_filename: String::new(),
            current_file_path: None,
            search_query: String::new(),
            search_results: Vec::new(),
            current_search_index: 0,
            selected_line: None,
            rules: crate::gcodeedit::rules::RuleSet::new_default(),
            diagnostics: Vec::new(),
            content_version: 0,
            autocomplete: crate::gcodeedit::autocomplete::AutoCompleter::new("1.1"),
            show_autocomplete: false,
            autocomplete_suggestions: Vec::new(),
            autocomplete_selected: 0,
            virtualized_state: crate::gcodeedit::virtualized_view::VirtualizedState::default(),
            virtualized_config: crate::gcodeedit::virtualized_view::VirtualizedConfig::default(),
            fold_manager: crate::gcodeedit::virtualized_view::FoldManager::new(),
            performance_metrics: crate::gcodeedit::virtualized_view::PerformanceMetrics::default(),
            enable_virtualization: true, // Enable by default for performance
            find_replace: crate::gcodeedit::find_replace::FindReplace::new(),
            show_find_replace: false,
            show_replace: false,
            show_shortcuts_help: false,
            parsed_paths: Vec::new(),
            sending_from_line: None,
            sending_progress: 0.0,
            cached_row_height: None,
        }
    }
}

impl GcodeEditorState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Return the effective content: prefer buffer content if non-empty, otherwise fall back to gcode_content
    pub fn content(&self) -> String {
        let buf = self.buffer.get_content();
        if !buf.is_empty() {
            buf
        } else {
            self.gcode_content.clone()
        }
    }

    /// Update rule configuration and re-validate
    pub fn update_rules(&mut self, new_rules: crate::gcodeedit::rules::RuleSet) {
        self.rules = new_rules;
        // Clear cache since rules changed
        self.rules.clear_cache();
        // Re-validate with new rules
        self.on_buffer_change();
    }

    /// Enable a specific rule by ID and re-validate
    pub fn enable_rule(&mut self, rule_id: &str) {
        self.rules.enable_rule(rule_id);
        self.rules.clear_cache();
        self.on_buffer_change();
    }

    /// Disable a specific rule by ID and re-validate
    pub fn disable_rule(&mut self, rule_id: &str) {
        self.rules.disable_rule(rule_id);
        self.rules.clear_cache();
        self.on_buffer_change();
    }

    /// Trigger autocomplete at current cursor position
    pub fn trigger_autocomplete(&mut self) {
        let cursor = self.buffer.cursor;
        let lines: Vec<String> = self.content().lines().map(|s| s.to_string()).collect();

        if cursor.line < lines.len() {
            let line = &lines[cursor.line];
            let context = crate::gcodeedit::autocomplete::CompletionContext {
                line_before_cursor: line[..cursor.col.min(line.len())].to_string(),
                line_after_cursor: line[cursor.col.min(line.len())..].to_string(),
                recent_commands: self.get_recent_commands(),
                modal_state: crate::gcodeedit::autocomplete::ModalState::default(),
            };

            self.autocomplete_suggestions = self
                .autocomplete
                .get_suggestions(line, cursor.col, &context);

            if !self.autocomplete_suggestions.is_empty() {
                self.show_autocomplete = true;
                self.autocomplete_selected = 0;
            } else {
                self.show_autocomplete = false;
            }
        }
    }

    /// Accept the currently selected autocomplete suggestion
    pub fn accept_autocomplete(&mut self) {
        if self.show_autocomplete
            && self.autocomplete_selected < self.autocomplete_suggestions.len()
        {
            let suggestion = &self.autocomplete_suggestions[self.autocomplete_selected];
            let cursor = self.buffer.cursor;
            let lines: Vec<String> = self.content().lines().map(|s| s.to_string()).collect();

            if cursor.line < lines.len() {
                let line = &lines[cursor.line];

                // Find the start of the current word to replace
                let mut start_col = cursor.col;
                let before = &line[..cursor.col.min(line.len())];
                for (i, c) in before.chars().rev().enumerate() {
                    if c.is_whitespace() {
                        start_col = cursor.col - i;
                        break;
                    }
                    if i == before.len() - 1 {
                        start_col = 0;
                    }
                }

                // Delete current word and insert suggestion
                let start = crate::gcodeedit::editor::Cursor::new(cursor.line, start_col);
                self.buffer.delete_range(start, cursor);
                self.buffer.insert_text(&suggestion.insert_text);
            }

            self.show_autocomplete = false;
            self.on_buffer_change();
        }
    }

    /// Move autocomplete selection up
    pub fn autocomplete_prev(&mut self) {
        if self.show_autocomplete && !self.autocomplete_suggestions.is_empty() {
            if self.autocomplete_selected > 0 {
                self.autocomplete_selected -= 1;
            } else {
                self.autocomplete_selected = self.autocomplete_suggestions.len() - 1;
            }
        }
    }

    /// Move autocomplete selection down
    pub fn autocomplete_next(&mut self) {
        if self.show_autocomplete && !self.autocomplete_suggestions.is_empty() {
            self.autocomplete_selected =
                (self.autocomplete_selected + 1) % self.autocomplete_suggestions.len();
        }
    }

    /// Cancel autocomplete
    pub fn cancel_autocomplete(&mut self) {
        self.show_autocomplete = false;
        self.autocomplete_suggestions.clear();
        self.autocomplete_selected = 0;
    }

    /// Get recently used commands from the file for context-aware suggestions
    fn get_recent_commands(&self) -> Vec<String> {
        let mut commands = Vec::new();
        let lines: Vec<String> = self.content().lines().map(|s| s.to_string()).collect();

        for line in lines.iter().rev().take(20) {
            let trimmed = line.trim();
            if let Some(first) = trimmed.split_whitespace().next() {
                if (first.starts_with('G') || first.starts_with('M'))
                    && !commands.contains(&first.to_string()) {
                        commands.push(first.to_string());
                    }
            }
        }

        commands
    }

    /// Detect and update fold regions
    pub fn detect_folds(&mut self) {
        let lines: Vec<String> = self.content().lines().map(|s| s.to_string()).collect();
        self.fold_manager.detect_folds(&lines);
    }

    /// Toggle fold at a specific line
    pub fn toggle_fold_at_line(&mut self, line: usize) {
        self.fold_manager.toggle_fold_at(line);
    }

    /// Expand all folds
    pub fn expand_all_folds(&mut self) {
        for region in &mut self.fold_manager.regions {
            region.is_folded = false;
        }
    }

    /// Collapse all folds
    pub fn collapse_all_folds(&mut self) {
        for region in &mut self.fold_manager.regions {
            region.is_folded = true;
        }
    }

    /// Scroll to a specific line (for virtualized rendering)
    pub fn scroll_to_line(&mut self, line: usize) {
        self.virtualized_state
            .scroll_to_line(line, &self.virtualized_config);
    }

    pub fn on_buffer_change(&mut self) {
        // Increment content version to track changes
        self.content_version = self.content_version.wrapping_add(1);

        // Submit to tokenizer service for debounced background parsing
        let content = self.content();
        self.tokenizer.submit_content(&content);

        // Run immediate synchronous parse for near-instant diagnostics
        let parsed = crate::gcodeedit::tokenizer::parse_content_sync(&content);

        // Use incremental validation with caching
        self.diagnostics = self
            .rules
            .validate_parsed(&parsed, Some(self.content_version));

        // Update last_parsed snapshot
        self.last_parsed = parsed;
    }

    pub fn load_gcode_file(&mut self) -> Result<(), String> {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("G-code files", &["gcode", "nc", "txt"])
            .pick_file()
        {
            match std::fs::read_to_string(&path) {
                Ok(content) => {
                    self.gcode_content = content.clone();
                    self.buffer.set_content(&content);
                    // submit to tokenizer immediately
                    self.tokenizer.submit_content(&content);
                    self.current_file_path = Some(path.clone());
                    self.gcode_filename = path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    // Trigger validation after loading
                    self.on_buffer_change();
                    // Detect fold regions
                    self.detect_folds();
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
            match std::fs::write(path, self.buffer.get_content()) {
                Ok(_) => Ok(()),
                Err(e) => Err(format!("Error saving file: {}", e)),
            }
        } else {
            self.save_gcode_file_as()
        }
    }

    pub fn save_gcode_file_as(&mut self) -> Result<(), String> {
        if self.buffer.get_content().is_empty() {
            return Err("No G-code to save".to_string());
        }

        if let Some(path) = rfd::FileDialog::new()
            .add_filter("G-code files", &["gcode", "nc", "txt"])
            .set_file_name(&self.gcode_filename)
            .save_file()
        {
            match std::fs::write(&path, self.buffer.get_content()) {
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
        if self.content().is_empty() {
            return "No G-code to optimize".to_string();
        }

        let original_size = self.content().len();
        let original_lines = self.content().lines().count();
        let mut optimized_lines = Vec::new();
        let mut current_pos = MachinePosition::new(0.0, 0.0, 0.0);

        for line in self.content().lines() {
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

        let new_content = optimized_lines.join("\n");
        self.buffer.set_content(&new_content);
        self.gcode_content = self.buffer.get_content();
        // Re-validate optimized content
        self.on_buffer_change();

        let optimized_size = self.content().len();
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
                let first_char = match part.chars().next() {
                    Some(c) => c,
                    None => continue,
                };

                // Handle parameters with decimal truncation
                if first_char.is_ascii_alphabetic() && part.len() > 1 {
                    if let Ok(value) = part[1..].parse::<f32>() {
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
                let first_char = match part.chars().next() {
                    Some(c) => c,
                    None => continue,
                };
                if first_char.is_ascii_alphabetic() {
                    if let Ok(value) = part[1..].parse::<f32>() {
                        params.insert(first_char, value);
                    }
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
        let num_segments = (arc_length / 1.0).clamp(4.0, 100.0) as usize; // At least 4 segments, max 100

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
                let first_char = match part.chars().next() {
                    Some(c) => c,
                    None => continue,
                };
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

        for (line_idx, line) in self.content().lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with(';') {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            let mut new_pos = current_pos.clone();
            let mut move_type = current_move_type.clone();

            for part in parts {
                if let Some(stripped) = part.strip_prefix('G') {
                    if let Ok(code) = stripped.parse::<u32>() {
                        match code {
                            0 => move_type = MoveType::Rapid,
                            1 => move_type = MoveType::Feed,
                            2 | 3 => move_type = MoveType::Arc,
                            _ => {}
                        }
                    }
                } else if part.len() > 1 {
                    let axis = match part.chars().next() {
                        Some(c) => c,
                        None => continue,
                    };
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

        for (line_num, line) in self.content().lines().enumerate() {
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

    /// Move selection to the next diagnostic (by line). Returns true if moved.
    pub fn next_diagnostic(&mut self) -> bool {
        let mut lines: Vec<usize> = self.diagnostics.iter().map(|d| d.line).collect();
        lines.sort_unstable();
        lines.dedup();
        if lines.is_empty() {
            return false;
        }
        let cur = self.selected_line.unwrap_or(usize::MAX);
        for l in &lines {
            if *l > cur {
                self.selected_line = Some(*l);
                return true;
            }
        }
        // wrap
        self.selected_line = Some(lines[0]);
        true
    }

    /// Move selection to the previous diagnostic (by line). Returns true if moved.
    pub fn prev_diagnostic(&mut self) -> bool {
        let mut lines: Vec<usize> = self.diagnostics.iter().map(|d| d.line).collect();
        lines.sort_unstable();
        lines.dedup();
        if lines.is_empty() {
            return false;
        }
        let cur = self.selected_line.unwrap_or(usize::MAX);
        for l in lines.iter().rev() {
            if *l < cur {
                self.selected_line = Some(*l);
                return true;
            }
        }
        // wrap
        self.selected_line = Some(*lines.last().unwrap());
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

        if self.content().is_empty() {
            return Err("No G-code loaded".to_string());
        }

        let lines: Vec<String> = self.content().lines().map(|s| s.to_string()).collect();
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

    pub fn show_ui(&mut self, ui: &mut egui::Ui, _parsed_paths: &[PathSegment]) -> Option<usize> {
        if self.buffer.get_content().is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label("No G-code file loaded. Use 'Load File' in the left panel.");
            });
            return None;
        }

        // Keyboard shortcuts for Find/Replace
        if ui.input(|i| i.key_pressed(egui::Key::F) && i.modifiers.ctrl) {
            self.show_find_replace = true;
            self.show_replace = false;
        }
        if ui.input(|i| i.key_pressed(egui::Key::H) && i.modifiers.ctrl) {
            self.show_find_replace = true;
            self.show_replace = true;
        }
        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.show_find_replace = false;
        }

        // Additional keyboard shortcuts
        // Undo/Redo
        if ui.input(|i| i.key_pressed(egui::Key::Z) && i.modifiers.ctrl && !i.modifiers.shift) {
            self.buffer.undo();
            self.on_buffer_change();
        }
        if ui.input(|i| (i.key_pressed(egui::Key::Z) && i.modifiers.ctrl && i.modifiers.shift) 
                        || (i.key_pressed(egui::Key::Y) && i.modifiers.ctrl)) {
            self.buffer.redo();
            self.on_buffer_change();
        }

        // Save
        if ui.input(|i| i.key_pressed(egui::Key::S) && i.modifiers.ctrl) {
            if let Err(e) = self.save_gcode_file() {
                eprintln!("Failed to save: {}", e);
            }
        }

        // Toggle comment (Ctrl+/)
        if ui.input(|i| i.key_pressed(egui::Key::Slash) && i.modifiers.ctrl) {
            if let Some(line_num) = self.selected_line {
                let lines: Vec<String> = self.content().lines().map(|s| s.to_string()).collect();
                if line_num < lines.len() {
                    let mut new_lines = lines.clone();
                    let line = &new_lines[line_num];
                    
                    // Toggle comment
                    if line.trim_start().starts_with(';') {
                        // Remove comment
                        new_lines[line_num] = line.trim_start().trim_start_matches(';').trim_start().to_string();
                    } else {
                        // Add comment
                        new_lines[line_num] = format!("; {}", line);
                    }
                    
                    self.buffer.set_content(&new_lines.join("\n"));
                    self.on_buffer_change();
                }
            }
        }

        // Fold/Unfold (Ctrl+])
        if ui.input(|i| i.key_pressed(egui::Key::CloseBracket) && i.modifiers.ctrl) {
            if let Some(line_num) = self.selected_line {
                self.toggle_fold_at_line(line_num);
            }
        }

        // Expand all folds (Ctrl+Shift+])
        if ui.input(|i| i.key_pressed(egui::Key::CloseBracket) && i.modifiers.ctrl && i.modifiers.shift) {
            self.expand_all_folds();
        }

        // Collapse all folds (Ctrl+Shift+[)
        if ui.input(|i| i.key_pressed(egui::Key::OpenBracket) && i.modifiers.ctrl && i.modifiers.shift) {
            self.collapse_all_folds();
        }

        // Go to line (Ctrl+L)
        // TODO: Add goto line dialog

        // Select all (Ctrl+A)
        if ui.input(|i| i.key_pressed(egui::Key::A) && i.modifiers.ctrl) {
            // TODO: Implement select all
        }

        // Copy (Ctrl+C) - handled by egui
        // Cut (Ctrl+X) - handled by egui
        // Paste (Ctrl+V) - handled by egui

        // Show shortcuts help (F1 or Ctrl+?)
        if ui.input(|i| i.key_pressed(egui::Key::F1) 
                        || (i.key_pressed(egui::Key::Slash) && i.modifiers.ctrl && i.modifiers.shift)) {
            self.show_shortcuts_help = !self.show_shortcuts_help;
        }

        // Keyboard shortcuts help dialog
        if self.show_shortcuts_help {
            egui::Window::new("⌨️ Keyboard Shortcuts")
                .open(&mut self.show_shortcuts_help)
                .resizable(false)
                .default_width(500.0)
                .show(ui.ctx(), |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.heading("Navigation");
                        ui.label("F7 / F8 - Navigate diagnostics");
                        ui.label("Ctrl+G - Jump to next diagnostic");
                        ui.add_space(10.0);

                        ui.heading("Editing");
                        ui.label("Ctrl+Z - Undo");
                        ui.label("Ctrl+Y / Ctrl+Shift+Z - Redo");
                        ui.label("Ctrl+/ - Toggle comment on selected line");
                        ui.label("Ctrl+S - Save file");
                        ui.label("Ctrl+A - Select all");
                        ui.label("Ctrl+C - Copy");
                        ui.label("Ctrl+X - Cut");
                        ui.label("Ctrl+V - Paste");
                        ui.add_space(10.0);

                        ui.heading("Search & Replace");
                        ui.label("Ctrl+F - Open find dialog");
                        ui.label("Ctrl+H - Open find/replace dialog");
                        ui.label("Esc - Close find/replace");
                        ui.add_space(10.0);

                        ui.heading("Code Folding");
                        ui.label("Ctrl+] - Toggle fold at selected line");
                        ui.label("Ctrl+Shift+] - Expand all folds");
                        ui.label("Ctrl+Shift+[ - Collapse all folds");
                        ui.add_space(10.0);

                        ui.heading("Auto-completion");
                        ui.label("Ctrl+Space - Trigger auto-completion");
                        ui.label("↑↓ - Navigate suggestions");
                        ui.label("Enter - Accept suggestion");
                        ui.label("Esc - Cancel auto-completion");
                        ui.add_space(10.0);

                        ui.heading("Help");
                        ui.label("F1 - Show/hide this help dialog");
                    });
                });
        }

        // Find/Replace panel
        if self.show_find_replace {
            let mut window_open = true;
            let mut needs_find = false;
            let mut needs_replace_current = false;
            let mut needs_replace_all = false;
            let mut nav_prev = false;
            let mut nav_next = false;
            
            egui::Window::new("Find and Replace")
                .open(&mut window_open)
                .resizable(false)
                .default_width(500.0)
                .show(ui.ctx(), |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Find:");
                        let find_response = ui.text_edit_singleline(&mut self.find_replace.query);
                        
                        if find_response.changed() || ui.button("🔍 Find All").clicked() {
                            needs_find = true;
                        }
                    });

                    if self.show_replace {
                        ui.horizontal(|ui| {
                            ui.label("Replace:");
                            ui.text_edit_singleline(&mut self.find_replace.replace_text);
                        });
                    }

                    ui.horizontal(|ui| {
                        ui.checkbox(&mut self.find_replace.options.case_sensitive, "Match case");
                        ui.checkbox(&mut self.find_replace.options.whole_word, "Whole word");
                        ui.checkbox(&mut self.find_replace.options.use_regex, "Use regex");
                        ui.checkbox(&mut self.find_replace.options.wrap_around, "Wrap around");
                    });

                    ui.separator();

                    ui.horizontal(|ui| {
                        if ui.button("⬆️ Previous").clicked() {
                            nav_prev = true;
                        }

                        if ui.button("⬇️ Next").clicked() {
                            nav_next = true;
                        }

                        if !self.find_replace.matches.is_empty() {
                            ui.label(format!(
                                "{} / {}",
                                self.find_replace.current_match_number(),
                                self.find_replace.match_count()
                            ));
                        }
                    });

                    if self.show_replace {
                        ui.horizontal(|ui| {
                            if ui.button("Replace").clicked() && !self.find_replace.matches.is_empty() {
                                needs_replace_current = true;
                            }

                            if ui.button("Replace All").clicked() && !self.find_replace.matches.is_empty() {
                                needs_replace_all = true;
                            }
                        });
                    }

                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.label("💡 Tip:");
                        ui.label("Ctrl+F to find, Ctrl+H to replace, Esc to close");
                    });
                });

            self.show_find_replace = window_open;

            // Handle actions outside the window closure
            if needs_find {
                let content = self.content();
                let count = self.find_replace.find(&content);
                if count > 0 {
                    if let Some(first_match) = self.find_replace.current() {
                        self.selected_line = Some(first_match.line);
                    }
                }
            }

            if nav_prev {
                if let Some(mat) = self.find_replace.prev_match() {
                    let line = mat.line;
                    self.selected_line = Some(line);
                    self.scroll_to_line(line);
                }
            }

            if nav_next {
                if let Some(mat) = self.find_replace.next_match() {
                    let line = mat.line;
                    self.selected_line = Some(line);
                    self.scroll_to_line(line);
                }
            }

            if needs_replace_current {
                let content = self.content();
                let new_content = self.find_replace.replace_current(&content);
                self.buffer.set_content(&new_content);
                self.on_buffer_change();
                
                // Re-run find to update matches
                let content = self.content();
                self.find_replace.find(&content);
            }

            if needs_replace_all {
                let content = self.content();
                let (_new_content, _count) = self.find_replace.replace_all(&content);
                self.buffer.set_content(&_new_content);
                self.on_buffer_change();
            }
        }

        // Legacy search controls (kept for compatibility, can be removed later)
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

            // Diagnostic navigation shortcuts
            if ui.button("⏭️ Next Diag").clicked() {
                self.next_diagnostic();
            }
            if ui.button("⏮️ Prev Diag").clicked() {
                self.prev_diagnostic();
            }

            // Keyboard shortcuts
            if ui.input(|i| i.key_pressed(egui::Key::F8)) {
                self.next_diagnostic();
            }
            if ui.input(|i| i.key_pressed(egui::Key::F7)) {
                self.prev_diagnostic();
            }
            // Jump to next diagnostic from current position (Ctrl+G)
            if ui.input(|i| i.key_pressed(egui::Key::G) && i.modifiers.ctrl) {
                self.next_diagnostic();
            }
        });

        // Autocomplete keyboard shortcuts (outside text edit to capture globally)
        if ui.input(|i| i.key_pressed(egui::Key::Space) && i.modifiers.ctrl) {
            self.trigger_autocomplete();
        }

        if self.show_autocomplete {
            if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                self.autocomplete_prev();
            }
            if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                self.autocomplete_next();
            }
            if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                self.accept_autocomplete();
            }
            if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                self.cancel_autocomplete();
            }
        }

        // Performance monitoring
        let render_start = std::time::Instant::now();

        // Determine if we should use virtualization
        let content_clone = self.content();
        let lines: Vec<String> = content_clone.lines().map(|s| s.to_string()).collect();
        let total_lines = lines.len();
        let use_virtualization = self.enable_virtualization && total_lines > self.virtualized_config.max_rendered_lines;

        // Add performance info header
        ui.horizontal(|ui| {
            ui.label(format!("Lines: {}", total_lines));
            if use_virtualization {
                ui.colored_label(egui::Color32::GREEN, "⚡ Virtualized");
                ui.label(format!(
                    "Rendering: {}-{}",
                    self.virtualized_state.first_visible_line,
                    self.virtualized_state.last_visible_line
                ));
            }
            
            // Fold controls
            if ui.button("📁 Detect Folds").clicked() {
                self.detect_folds();
            }
            if ui.button("➕ Expand All").clicked() {
                self.expand_all_folds();
            }
            if ui.button("➖ Collapse All").clicked() {
                self.collapse_all_folds();
            }
            
            // Performance metrics
            if !self.performance_metrics.summary().is_empty() {
                ui.label(format!("⏱️ {}", self.performance_metrics.summary()));
            }
        });

        ui.separator();

        egui::ScrollArea::vertical()
            .id_salt("gcode_editor_scroll")
            .show(ui, |ui| {
            // For now, render all non-folded lines
            // TODO: Implement true virtualization with scroll tracking
            
            // Calculate visible lines (accounting for folds)
            let visible_lines: Vec<usize> = if use_virtualization && total_lines > 1000 {
                // Use virtualized range
                let range = self.virtualized_state.visible_range();
                range.filter(|&line| !self.fold_manager.is_line_folded(line)).collect()
            } else {
                // Render all lines (accounting for folds)
                (0..total_lines)
                    .filter(|&line| !self.fold_manager.is_line_folded(line))
                    .collect()
            };

            ui.horizontal(|ui| {
                // Use cached row height from the galley if available, otherwise estimate
                let font_id = egui::TextStyle::Monospace.resolve(ui.style());
                let row_height = self.cached_row_height.unwrap_or(font_id.size * 1.45);
                
                // Gutter column with clickable markers and fold indicators
                ui.vertical(|g| {
                    // Remove all spacing to let text height control layout
                    g.spacing_mut().item_spacing.y = 0.0;
                    g.spacing_mut().button_padding = egui::vec2(4.0, 0.0);
                    
                    for &i in &visible_lines {
                        // Use allocate_ui to have full control over the row
                        let (rect, response) = g.allocate_exact_size(
                            egui::vec2(80.0, row_height),
                            egui::Sense::click()
                        );
                        
                        // Check if click is on fold icon area (first ~20 pixels)
                        let has_fold = self.fold_manager.get_region_at(i).is_some();
                        if response.clicked() {
                            if has_fold && response.interact_pointer_pos().map_or(false, |pos| {
                                pos.x - rect.left() < 20.0
                            }) {
                                self.toggle_fold_at_line(i);
                            } else {
                                self.selected_line = Some(i);
                            }
                        }
                        
                        // Fold indicator
                        let fold_icon = if let Some(region) = self.fold_manager.get_region_at(i) {
                            if region.is_folded { "▶ " } else { "▼ " }
                        } else {
                            "  "
                        };
                        
                        // Diagnostic icon
                        let icon = if self.diagnostics.iter().any(|d| {
                            d.line == i && d.severity == crate::gcodeedit::rules::Severity::Error
                        }) {
                            "❗"
                        } else if self.diagnostics.iter().any(|d| {
                            d.line == i && d.severity == crate::gcodeedit::rules::Severity::Warn
                        }) {
                            "⚠️"
                        } else if self.diagnostics.iter().any(|d| {
                            d.line == i && d.severity == crate::gcodeedit::rules::Severity::Info
                        }) {
                            "ℹ️"
                        } else {
                            " "
                        };
                        
                        // Draw background for selected line
                        if self.selected_line == Some(i) {
                            g.painter().rect_filled(
                                rect,
                                0.0,
                                g.visuals().selection.bg_fill
                            );
                        }
                        
                        // Draw the gutter text
                        let text = format!("{}{} {:05}", fold_icon, icon, i + 1);
                        let text_color = if self.selected_line == Some(i) {
                            g.visuals().strong_text_color()
                        } else {
                            g.visuals().text_color()
                        };
                        
                        g.painter().text(
                            rect.left_top() + egui::vec2(2.0, 0.0),
                            egui::Align2::LEFT_TOP,
                            text,
                            font_id.clone(),
                            text_color
                        );
                        
                        // Attach hover UI to show diagnostics
                        let diags: Vec<_> =
                            self.diagnostics.iter().filter(|d| d.line == i).collect();
                        if !diags.is_empty() {
                            response.on_hover_ui(|ui| {
                                ui.vertical(|ui| {
                                    for d in diags.iter() {
                                        let sev = match d.severity {
                                            crate::gcodeedit::rules::Severity::Error => "Error",
                                            crate::gcodeedit::rules::Severity::Warn => "Warn",
                                            crate::gcodeedit::rules::Severity::Info => "Info",
                                        };
                                        ui.label(format!("[{}] {}", sev, d.message));
                                    }
                                });
                            });
                        }
                    }
                });

                // Editor column
                let _response = ui.add(
                    egui::TextEdit::multiline(&mut self.buffer.get_content())
                        .font(egui::TextStyle::Monospace)
                        .desired_rows(20)
                        .layouter(&mut |ui: &egui::Ui, string: &dyn TextBuffer, _wrap_width| {
                            let mut job = LayoutJob::default();
                            let search_query_lower = self.search_query.to_lowercase();
                            let has_search =
                                !self.search_query.is_empty() && !self.search_results.is_empty();

                            // Get theme-aware colors
                            let is_dark = ui.visuals().dark_mode;
                            let text_color = ui.visuals().text_color();
                            let bg_color = ui.visuals().extreme_bg_color;
                            
                            // Syntax highlighting colors that work in both themes
                            let g_code_color = if is_dark {
                                egui::Color32::from_rgb(100, 150, 255)  // Light blue for dark
                            } else {
                                egui::Color32::from_rgb(0, 0, 200)      // Dark blue for light
                            };
                            
                            let m_code_color = if is_dark {
                                egui::Color32::from_rgb(100, 255, 150)  // Light green for dark
                            } else {
                                egui::Color32::from_rgb(0, 150, 0)      // Dark green for light
                            };
                            
                            let param_color = if is_dark {
                                egui::Color32::from_rgb(255, 120, 120)  // Light red for dark
                            } else {
                                egui::Color32::from_rgb(200, 0, 0)      // Dark red for light
                            };
                            
                            let comment_color = if is_dark {
                                egui::Color32::from_rgb(120, 120, 120)  // Light gray for dark
                            } else {
                                egui::Color32::from_rgb(100, 100, 100)  // Dark gray for light
                            };
                            
                            let keyword_color = if is_dark {
                                egui::Color32::from_rgb(255, 200, 100)  // Orange for dark
                            } else {
                                egui::Color32::from_rgb(200, 120, 0)    // Dark orange for light
                            };

                            // Helper function for syntax highlighting
                            let append_with_syntax_highlight =
                                |job: &mut LayoutJob, text: &str, line_bg: egui::Color32| {
                                    let words: Vec<&str> = text.split_whitespace().collect();
                                    for (j, word) in words.iter().enumerate() {
                                        let color = if word.starts_with('G')
                                            && word.len() > 1
                                            && word[1..].chars().all(|c| c.is_ascii_digit())
                                        {
                                            g_code_color
                                        } else if word.starts_with('M')
                                            && word.len() > 1
                                            && word[1..].chars().all(|c| c.is_ascii_digit())
                                        {
                                            m_code_color
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
                                            if word.len() > 1 {
                                                let value_part = &word[1..];
                                                if value_part.parse::<f32>().is_err()
                                                    && !value_part.contains('.')
                                                    && !value_part.starts_with('-')
                                                {
                                                    keyword_color
                                                } else {
                                                    param_color
                                                }
                                            } else {
                                                param_color
                                            }
                                        } else if word.starts_with(';') {
                                            comment_color
                                        } else if word
                                            .chars()
                                            .next()
                                            .is_some_and(|c| c.is_ascii_alphabetic())
                                        {
                                            keyword_color
                                        } else {
                                            text_color
                                        };
                                        job.append(
                                            word,
                                            0.0,
                                            TextFormat {
                                                font_id: egui::FontId::monospace(12.0),
                                                color,
                                                background: line_bg,
                                                ..Default::default()
                                            },
                                        );
                                        if j < words.len() - 1 {
                                            job.append(" ", 0.0, TextFormat {
                                                background: line_bg,
                                                ..Default::default()
                                            });
                                        }
                                    }
                                };

                            for (i, line) in string.as_str().lines().enumerate() {
                                // Determine line background color based on diagnostics
                                let mut line_bg = bg_color;
                                for d in &self.diagnostics {
                                    if d.line == i {
                                        line_bg = match d.severity {
                                            crate::gcodeedit::rules::Severity::Error => {
                                                if is_dark {
                                                    egui::Color32::from_rgb(80, 30, 30)   // Dark red bg
                                                } else {
                                                    egui::Color32::from_rgb(255, 200, 200) // Light red bg
                                                }
                                            }
                                            crate::gcodeedit::rules::Severity::Warn => {
                                                if is_dark {
                                                    egui::Color32::from_rgb(80, 60, 20)    // Dark yellow bg
                                                } else {
                                                    egui::Color32::from_rgb(255, 230, 180)  // Light yellow bg
                                                }
                                            }
                                            crate::gcodeedit::rules::Severity::Info => {
                                                if is_dark {
                                                    egui::Color32::from_rgb(20, 40, 60)    // Dark blue bg
                                                } else {
                                                    egui::Color32::from_rgb(230, 240, 255) // Light blue bg
                                                }
                                            }
                                        };
                                        break;
                                    }
                                }

                                // Check if this line is the current search result
                                let is_current_search_line = has_search
                                    && !self.search_results.is_empty()
                                    && i == self.search_results[self.current_search_index];

                                if is_current_search_line {
                                    let line_lower = line.to_lowercase();
                                    let mut pos = 0;
                                    while let Some(match_start) =
                                        line_lower[pos..].find(&search_query_lower)
                                    {
                                        let match_start = pos + match_start;
                                        let match_end = match_start + search_query_lower.len();

                                        if match_start > pos {
                                            append_with_syntax_highlight(
                                                &mut job,
                                                &line[pos..match_start],
                                                line_bg,
                                            );
                                        }

                                        job.append(
                                            &line[match_start..match_end],
                                            0.0,
                                            TextFormat {
                                                font_id: egui::FontId::monospace(12.0),
                                                color: egui::Color32::BLACK,
                                                background: if is_dark {
                                                    egui::Color32::from_rgb(180, 180, 0)  // Darker yellow
                                                } else {
                                                    egui::Color32::YELLOW                  // Bright yellow
                                                },
                                                ..Default::default()
                                            },
                                        );

                                        pos = match_end;
                                    }

                                    if pos < line.len() {
                                        append_with_syntax_highlight(
                                            &mut job,
                                            &line[pos..],
                                            line_bg,
                                        );
                                    }
                                } else if has_search
                                    && line.to_lowercase().contains(&search_query_lower)
                                {
                                    let line_lower = line.to_lowercase();
                                    let mut pos = 0;
                                    while let Some(match_start) =
                                        line_lower[pos..].find(&search_query_lower)
                                    {
                                        let match_start = pos + match_start;
                                        let match_end = match_start + search_query_lower.len();

                                        if match_start > pos {
                                            append_with_syntax_highlight(
                                                &mut job,
                                                &line[pos..match_start],
                                                line_bg,
                                            );
                                        }

                                        job.append(
                                            &line[match_start..match_end],
                                            0.0,
                                            TextFormat {
                                                font_id: egui::FontId::monospace(12.0),
                                                color: egui::Color32::BLACK,
                                                background: if is_dark {
                                                    egui::Color32::from_rgb(160, 160, 0)  // Darker yellow
                                                } else {
                                                    egui::Color32::from_rgb(255, 255, 200) // Light yellow
                                                },
                                                ..Default::default()
                                            },
                                        );

                                        pos = match_end;
                                    }

                                    if pos < line.len() {
                                        append_with_syntax_highlight(
                                            &mut job,
                                            &line[pos..],
                                            line_bg,
                                        );
                                    }
                                } else {
                                    append_with_syntax_highlight(&mut job, line, line_bg);
                                }

                                job.append("\n", 0.0, TextFormat::default());
                            }
                            let galley = ui.fonts_mut(|fonts| fonts.layout_job(job));
                            
                            // Cache the row height for gutter alignment
                            // Calculate from total galley height / number of rows
                            if !galley.rows.is_empty() {
                                self.cached_row_height = Some(galley.rect.height() / galley.rows.len() as f32);
                            }
                            
                            galley
                        }),
                );
            });

            // Diagnostics pane below editor
            if let Some(sel) = self.selected_line {
                let diags: Vec<_> = self.diagnostics.iter().filter(|d| d.line == sel).collect();
                if !diags.is_empty() {
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.colored_label(
                            egui::Color32::LIGHT_RED,
                            format!("Diagnostics for line {}:", sel + 1),
                        );
                    });
                    for d in diags {
                        ui.label(format!(
                            "- [{}] {}",
                            match d.severity {
                                crate::gcodeedit::rules::Severity::Error => "Error",
                                crate::gcodeedit::rules::Severity::Warn => "Warn",
                                crate::gcodeedit::rules::Severity::Info => "Info",
                            },
                            d.message
                        ));
                    }
                }
            }
        });

        // Update performance metrics
        let render_time = render_start.elapsed();
        let lines_to_track = if use_virtualization { 
            self.virtualized_state.last_visible_line - self.virtualized_state.first_visible_line
        } else {
            total_lines
        };
        self.performance_metrics.update(
            lines_to_track,
            render_time.as_micros() as u64,
            total_lines,
        );

        // Render autocomplete popup
        if self.show_autocomplete && !self.autocomplete_suggestions.is_empty() {
            let suggestions_clone = self.autocomplete_suggestions.clone();
            let selected_idx = self.autocomplete_selected;

            egui::Window::new("Autocomplete")
                .title_bar(false)
                .resizable(false)
                .collapsible(false)
                .fixed_pos(ui.cursor().min + egui::vec2(200.0, 0.0))
                .show(ui.ctx(), |ui| {
                    ui.set_max_width(400.0);
                    ui.set_max_height(300.0);

                    ui.label("Suggestions (↑↓ to navigate, Enter to accept, Esc to cancel):");
                    ui.separator();

                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for (idx, suggestion) in suggestions_clone.iter().enumerate() {
                            let is_selected = idx == selected_idx;
                            let label_text =
                                format!("{} - {}", suggestion.label, suggestion.detail);

                            let response = ui.selectable_label(is_selected, label_text);

                            if response.clicked() {
                                self.autocomplete_selected = idx;
                                self.accept_autocomplete();
                            }

                            if is_selected {
                                response.scroll_to_me(Some(egui::Align::Center));
                            }
                        }
                    });

                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.label("Tip: Press");
                        ui.monospace("Ctrl+Space");
                        ui.label("to show completions");
                    });
                });
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::MoveType;

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
        assert_eq!(paths[0].move_type, MoveType::Rapid);
        assert_eq!(paths[0].line_number, 0);

        // Second segment: G1 X30 Y40 F100
        assert_eq!(paths[1].start.x, 10.0);
        assert_eq!(paths[1].start.y, 20.0);
        assert_eq!(paths[1].end.x, 30.0);
        assert_eq!(paths[1].end.y, 40.0);
        assert_eq!(paths[1].move_type, MoveType::Feed);
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
        assert_eq!(paths[0].move_type, MoveType::Rapid);
        assert_eq!(paths[1].move_type, MoveType::Arc);
        assert_eq!(paths[2].move_type, MoveType::Arc);
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
            "G-code optimized: 5 -> 3 lines, 28 -> 24 bytes (14% reduction)"
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
