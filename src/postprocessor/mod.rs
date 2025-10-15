//! G-code postprocessing and controller-specific formatting.
//!
//! This module handles parsing, processing, and formatting G-code for different
//! CNC controllers, including postprocessing transformations and optimizations.

use crate::types::{MoveType, PathSegment};
use std::collections::HashMap;

/// Represents a parsed G-code command with its parameters
#[derive(Debug, Clone, PartialEq)]
pub struct GcodeCommand {
    pub command: String,
    pub parameters: HashMap<String, f32>,
    pub comment: Option<String>,
    pub line_number: usize,
}

impl GcodeCommand {
    pub fn new(command: String, line_number: usize) -> Self {
        Self {
            command,
            parameters: HashMap::new(),
            comment: None,
            line_number,
        }
    }

    pub fn with_parameter(mut self, key: String, value: f32) -> Self {
        self.parameters.insert(key, value);
        self
    }

    pub fn with_comment(mut self, comment: String) -> Self {
        self.comment = Some(comment);
        self
    }

    pub fn get_parameter(&self, key: &str) -> Option<f32> {
        self.parameters.get(key).copied()
    }

    pub fn has_parameter(&self, key: &str) -> bool {
        self.parameters.contains_key(key)
    }

    /// Convert back to G-code string
    pub fn to_gcode(&self) -> String {
        let mut result = self.command.clone();

        // Sort parameters for consistent output
        let mut sorted_params: Vec<_> = self.parameters.iter().collect();
        sorted_params.sort_by_key(|(k, _)| *k);

        for (key, value) in sorted_params {
            result.push_str(&format!(" {}{}", key, value));
        }

        if let Some(comment) = &self.comment {
            result.push_str(&format!(" ; {}", comment));
        }

        result
    }
}

/// Supported controller types for post-processing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ControllerType {
    Grbl,
    Smoothieware,
}

impl ControllerType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "grbl" => Some(Self::Grbl),
            "smoothieware" => Some(Self::Smoothieware),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Grbl => "GRBL",
            Self::Smoothieware => "Smoothieware",
        }
    }
}

/// Post-processor trait for converting G-code between controller formats
pub trait PostProcessor {
    /// Process a single G-code command for the target controller
    fn process_command(&self, command: &GcodeCommand) -> Vec<GcodeCommand>;

    /// Get the target controller type
    fn target_controller(&self) -> ControllerType;

    /// Get post-processor name
    fn name(&self) -> &'static str;

    /// Get post-processor description
    fn description(&self) -> &'static str;
}

/// Standard post-processor that passes commands through unchanged
pub struct StandardPostProcessor {
    target: ControllerType,
}

impl StandardPostProcessor {
    pub fn new(target: ControllerType) -> Self {
        Self { target }
    }
}

impl PostProcessor for StandardPostProcessor {
    fn process_command(&self, command: &GcodeCommand) -> Vec<GcodeCommand> {
        vec![command.clone()]
    }

    fn target_controller(&self) -> ControllerType {
        self.target
    }

    fn name(&self) -> &'static str {
        "Standard"
    }

    fn description(&self) -> &'static str {
        "Standard post-processor that passes G-code unchanged"
    }
}

/// GRBL-specific post-processor
pub struct GrblPostProcessor;

impl Default for GrblPostProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl GrblPostProcessor {
    pub fn new() -> Self {
        Self
    }
}

impl PostProcessor for GrblPostProcessor {
    fn process_command(&self, command: &GcodeCommand) -> Vec<GcodeCommand> {
        let mut processed = command.clone();

        // GRBL-specific modifications
        match command.command.as_str() {
            // Convert G28 (home) to GRBL format if needed
            "G28" => {
                // GRBL supports G28, but ensure no parameters that aren't supported
                if command.has_parameter("X")
                    || command.has_parameter("Y")
                    || command.has_parameter("Z")
                    || command.has_parameter("A")
                    || command.has_parameter("B")
                    || command.has_parameter("C")
                {
                    // GRBL G28 homes all axes or specified axes
                    processed = processed;
                }
            }
            // Ensure feed rates are within GRBL limits (if known)
            "G1" | "G2" | "G3" => {
                if let Some(_f) = command.get_parameter("F") {
                    // GRBL has no hard feed rate limit, but we could add validation here
                    processed = processed;
                }
            }
            _ => {}
        }

        vec![processed]
    }

    fn target_controller(&self) -> ControllerType {
        ControllerType::Grbl
    }

    fn name(&self) -> &'static str {
        "GRBL"
    }

    fn description(&self) -> &'static str {
        "Post-processor optimized for GRBL controllers"
    }
}

/// Smoothieware-specific post-processor
pub struct SmoothiewarePostProcessor;

impl Default for SmoothiewarePostProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl SmoothiewarePostProcessor {
    pub fn new() -> Self {
        Self
    }
}

impl PostProcessor for SmoothiewarePostProcessor {
    fn process_command(&self, command: &GcodeCommand) -> Vec<GcodeCommand> {
        let mut processed = command.clone();

        // Smoothieware-specific modifications
        match command.command.as_str() {
            // Smoothieware uses different homing commands
            "G28" => {
                // Convert to Smoothieware homing format if needed
                processed = processed;
            }
            // Smoothieware supports additional parameters
            _ => {}
        }

        vec![processed]
    }

    fn target_controller(&self) -> ControllerType {
        ControllerType::Smoothieware
    }

    fn name(&self) -> &'static str {
        "Smoothieware"
    }

    fn description(&self) -> &'static str {
        "Post-processor optimized for Smoothieware controllers"
    }
}

/// Main post-processor manager
pub struct PostProcessorManager {
    processors: HashMap<ControllerType, Box<dyn PostProcessor>>,
    current_processor: ControllerType,
}

impl PostProcessorManager {
    pub fn new() -> Self {
        let mut processors: HashMap<ControllerType, Box<dyn PostProcessor>> = HashMap::new();

        // Register all post-processors
        processors.insert(ControllerType::Grbl, Box::new(GrblPostProcessor::new()));
        processors.insert(
            ControllerType::Smoothieware,
            Box::new(SmoothiewarePostProcessor::new()),
        );

        Self {
            processors,
            current_processor: ControllerType::Grbl, // Default to GRBL
        }
    }

    pub fn set_target_controller(&mut self, controller: ControllerType) {
        self.current_processor = controller;
    }

    pub fn get_current_processor(&self) -> ControllerType {
        self.current_processor
    }

    pub fn get_available_processors(&self) -> Vec<ControllerType> {
        self.processors.keys().cloned().collect()
    }

    pub fn process_gcode(&self, commands: &[GcodeCommand]) -> Vec<GcodeCommand> {
        if let Some(processor) = self.processors.get(&self.current_processor) {
            commands
                .iter()
                .flat_map(|cmd| processor.process_command(cmd))
                .collect()
        } else {
            // Fallback to standard processor
            let standard = StandardPostProcessor::new(self.current_processor);
            commands
                .iter()
                .flat_map(|cmd| standard.process_command(cmd))
                .collect()
        }
    }

    pub fn get_processor_info(&self, controller: ControllerType) -> Option<(&str, &str)> {
        self.processors
            .get(&controller)
            .map(|p| (p.name(), p.description()))
    }
}

impl Default for PostProcessorManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse a line of G-code into a GcodeCommand
pub fn parse_gcode_line(line: &str, line_number: usize) -> Option<GcodeCommand> {
    let line = line.trim();

    // Skip empty lines and comments
    if line.is_empty() || line.starts_with(';') {
        return None;
    }

    // Extract comment if present
    let (code_part, comment) = if let Some(comment_pos) = line.find(';') {
        let (code, comment_str) = line.split_at(comment_pos);
        (code.trim(), Some(comment_str[1..].trim().to_string()))
    } else {
        (line, None)
    };

    if code_part.is_empty() {
        return None;
    }

    let parts: Vec<&str> = code_part.split_whitespace().collect();
    if parts.is_empty() {
        return None;
    }

    let command = parts[0].to_uppercase();
    let mut gcode_cmd = GcodeCommand::new(command, line_number);

    if let Some(comment) = comment {
        gcode_cmd = gcode_cmd.with_comment(comment);
    }

    // Parse parameters
    for part in &parts[1..] {
        if part.len() > 1 {
            let axis = part.chars().next().unwrap().to_uppercase().to_string();
            if let Ok(value) = part[1..].parse::<f32>() {
                gcode_cmd = gcode_cmd.with_parameter(axis, value);
            }
        }
    }

    Some(gcode_cmd)
}

/// Parse a complete G-code string into commands
pub fn parse_gcode(gcode: &str) -> Vec<GcodeCommand> {
    gcode
        .lines()
        .enumerate()
        .filter_map(|(i, line)| parse_gcode_line(line, i))
        .collect()
}

/// Convert processed commands back to G-code string
pub fn commands_to_gcode(commands: &[GcodeCommand]) -> String {
    commands
        .iter()
        .map(|cmd| cmd.to_gcode())
        .collect::<Vec<String>>()
        .join("\n")
}

/// Convert PathSegments to G-code string
pub fn path_segments_to_gcode(segments: &[PathSegment]) -> String {
    let mut gcode_lines = Vec::new();

    // Add header
    gcode_lines.push("G21 ; Set units to mm".to_string());
    gcode_lines.push("G90 ; Absolute positioning".to_string());
    gcode_lines.push("G0 Z5 ; Safe Z".to_string());

    for segment in segments {
        let _start = &segment.start;
        let end = &segment.end;

        match segment.move_type {
            MoveType::Rapid => {
                gcode_lines.push(format!("G0 X{:.3} Y{:.3} Z{:.3}", end.x, end.y, end.z));
            }
            MoveType::Feed => {
                gcode_lines.push(format!(
                    "G1 X{:.3} Y{:.3} Z{:.3} F1000",
                    end.x, end.y, end.z
                ));
            }
            MoveType::Arc => {
                // For arcs, we'd need center point information
                // For now, treat as feed move
                gcode_lines.push(format!(
                    "G1 X{:.3} Y{:.3} Z{:.3} F1000",
                    end.x, end.y, end.z
                ));
            }
        }
    }

    // Add footer
    gcode_lines.push("G0 Z5 ; Safe Z".to_string());
    gcode_lines.push("M30 ; Program end".to_string());

    gcode_lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_gcode() {
        let cmd = parse_gcode_line("G0 X10 Y20", 0).unwrap();
        assert_eq!(cmd.command, "G0");
        assert_eq!(cmd.get_parameter("X"), Some(10.0));
        assert_eq!(cmd.get_parameter("Y"), Some(20.0));
        assert_eq!(cmd.line_number, 0);
    }

    #[test]
    fn test_parse_gcode_with_comment() {
        let cmd = parse_gcode_line("G1 X30 Y40 ; Move to position", 1).unwrap();
        assert_eq!(cmd.command, "G1");
        assert_eq!(cmd.comment, Some("Move to position".to_string()));
    }

    #[test]
    fn test_parse_empty_line() {
        assert!(parse_gcode_line("", 0).is_none());
    }

    #[test]
    fn test_parse_comment_only() {
        assert!(parse_gcode_line("; This is a comment", 0).is_none());
    }

    #[test]
    fn test_command_to_gcode() {
        let cmd = GcodeCommand::new("G0".to_string(), 0)
            .with_parameter("X".to_string(), 10.0)
            .with_parameter("Y".to_string(), 20.0)
            .with_comment("Test comment".to_string());

        let gcode = cmd.to_gcode();
        assert_eq!(gcode, "G0 X10 Y20 ; Test comment");
    }

    #[test]
    fn test_post_processor_manager() {
        let mut manager = PostProcessorManager::new();

        // Test default processor
        assert_eq!(manager.get_current_processor(), ControllerType::Grbl);

        // Test setting processor
        manager.set_target_controller(ControllerType::Smoothieware);
        assert_eq!(
            manager.get_current_processor(),
            ControllerType::Smoothieware
        );

        // Test available processors
        let available = manager.get_available_processors();
        assert!(available.contains(&ControllerType::Grbl));
        assert!(available.contains(&ControllerType::Smoothieware));
    }

    #[test]
    fn test_grbl_post_processor() {
        let processor = GrblPostProcessor::new();
        let cmd = GcodeCommand::new("G0".to_string(), 0).with_parameter("X".to_string(), 10.0);

        let processed = processor.process_command(&cmd);
        assert_eq!(processed.len(), 1);
        assert_eq!(processed[0].command, "G0");
    }
}
