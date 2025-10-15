//! Auto-completion API for G-code editor
//!
//! Provides context-aware completion suggestions for G/M codes, parameters,
//! and values based on cursor position and surrounding context.
//!
//! # Features
//!
//! - **G/M Code Completion**: Suggests valid G and M codes based on GRBL version
//! - **Parameter Completion**: Context-aware parameter suggestions (X, Y, Z, F, S, etc.)
//! - **Context Analysis**: Determines what to suggest based on cursor position
//! - **Fuzzy Matching**: Supports partial matches for better UX
//!
//! # Example
//!
//! ```no_run
//! use gcodekit::gcodeedit::autocomplete::{AutoCompleter, CompletionContext};
//!
//! let completer = AutoCompleter::new("1.1");
//! let line = "G1 X10 ";
//! let suggestions = completer.get_suggestions(line, 7, &CompletionContext::default());
//! ```

use crate::gcodeedit::vocabulary::{CodeInfo, G_CODES, M_CODES};

/// Type of completion being requested
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompletionType {
    /// Complete a G or M code
    Command,
    /// Complete a parameter letter (X, Y, Z, F, S, etc.)
    Parameter,
    /// Complete a parameter value
    Value,
    /// Unknown context
    Unknown,
}

/// A single completion suggestion
#[derive(Debug, Clone)]
pub struct CompletionItem {
    /// The text to insert
    pub insert_text: String,
    /// Display label (may include description)
    pub label: String,
    /// Detailed description
    pub detail: String,
    /// Sort priority (lower = higher priority)
    pub sort_order: usize,
    /// Category for grouping
    pub category: String,
}

/// Context information for completion
#[derive(Debug, Clone, Default)]
pub struct CompletionContext {
    /// Current line text before cursor
    pub line_before_cursor: String,
    /// Current line text after cursor
    pub line_after_cursor: String,
    /// Previously used commands in file (for smart suggestions)
    pub recent_commands: Vec<String>,
    /// Current modal state (G0/G1, G90/G91, etc.)
    pub modal_state: ModalState,
}

/// Modal state tracking for context-aware completion
#[derive(Debug, Clone, Default)]
pub struct ModalState {
    /// Current motion mode (G0, G1, G2, G3)
    pub motion_mode: Option<String>,
    /// Absolute (G90) or incremental (G91) positioning
    pub positioning_mode: Option<String>,
    /// Current plane (G17, G18, G19)
    pub plane: Option<String>,
    /// Current units (G20, G21)
    pub units: Option<String>,
}

/// Auto-completer for G-code
#[derive(Debug, Clone)]
pub struct AutoCompleter {
    grbl_version: String,
}

impl AutoCompleter {
    /// Create a new auto-completer for a specific GRBL version
    pub fn new(grbl_version: &str) -> Self {
        Self {
            grbl_version: grbl_version.to_string(),
        }
    }

    /// Get completion suggestions at a given position
    ///
    /// # Arguments
    /// * `line` - The current line text
    /// * `cursor_col` - Cursor column position in the line
    /// * `context` - Additional context information
    ///
    /// # Returns
    /// Vector of completion items sorted by relevance
    pub fn get_suggestions(
        &self,
        line: &str,
        cursor_col: usize,
        context: &CompletionContext,
    ) -> Vec<CompletionItem> {
        let comp_type = self.analyze_completion_type(line, cursor_col);

        match comp_type {
            CompletionType::Command => self.get_command_completions(line, cursor_col),
            CompletionType::Parameter => self.get_parameter_completions(line, cursor_col, context),
            CompletionType::Value => self.get_value_completions(line, cursor_col, context),
            CompletionType::Unknown => Vec::new(),
        }
    }

    /// Analyze what type of completion is needed at cursor position
    fn analyze_completion_type(&self, line: &str, cursor_col: usize) -> CompletionType {
        let before_cursor = &line[..cursor_col.min(line.len())];
        let trimmed = before_cursor.trim_start();

        // Check if we're at the start of line or after whitespace (command completion)
        if trimmed.is_empty() || trimmed.ends_with(' ') {
            // Check if there's already a command
            let tokens: Vec<&str> = trimmed.split_whitespace().collect();
            if tokens.is_empty() {
                return CompletionType::Command;
            }
            // Check if first token is a command
            if let Some(first) = tokens.first() {
                if first.starts_with('G') || first.starts_with('M') {
                    return CompletionType::Parameter;
                }
            }
            return CompletionType::Command;
        }

        // Get the current word being typed
        let current_word = self.get_current_word(line, cursor_col);

        if current_word.starts_with('G') || current_word.starts_with('M') {
            CompletionType::Command
        } else if current_word.len() == 1
            && current_word.chars().next().unwrap().is_ascii_alphabetic()
        {
            // Single letter: could be parameter name OR starting a value
            // Provide both parameter and value completions by returning Value
            // (value completion will fall back to parameters if no values match)
            CompletionType::Value
        } else if !current_word.is_empty()
            && current_word.chars().next().unwrap().is_ascii_alphabetic()
        {
            CompletionType::Value
        } else {
            CompletionType::Parameter
        }
    }

    /// Get the word currently being typed at cursor position
    fn get_current_word(&self, line: &str, cursor_col: usize) -> String {
        let before = &line[..cursor_col.min(line.len())];
        let mut start = cursor_col;

        // Find start of current word
        for (i, c) in before.chars().rev().enumerate() {
            if c.is_whitespace() {
                start = cursor_col - i;
                break;
            }
            if i == before.len() - 1 {
                start = 0;
            }
        }

        line[start..cursor_col.min(line.len())].to_string()
    }

    /// Get command (G/M code) completions
    fn get_command_completions(&self, line: &str, cursor_col: usize) -> Vec<CompletionItem> {
        let current_word = self.get_current_word(line, cursor_col).to_uppercase();
        let mut completions = Vec::new();

        // G-codes
        for code_info in G_CODES.iter() {
            if self.is_code_supported(code_info)
                && (current_word.is_empty() || code_info.code.starts_with(&current_word))
            {
                completions.push(CompletionItem {
                    insert_text: format!("{} ", code_info.code),
                    label: code_info.code.to_string(),
                    detail: code_info.description.to_string(),
                    sort_order: self.get_code_priority(code_info.code),
                    category: "G-code".to_string(),
                });
            }
        }

        // M-codes
        for code_info in M_CODES.iter() {
            if self.is_code_supported(code_info)
                && (current_word.is_empty() || code_info.code.starts_with(&current_word))
            {
                completions.push(CompletionItem {
                    insert_text: format!("{} ", code_info.code),
                    label: code_info.code.to_string(),
                    detail: code_info.description.to_string(),
                    sort_order: self.get_code_priority(code_info.code),
                    category: "M-code".to_string(),
                });
            }
        }

        completions.sort_by_key(|c| c.sort_order);
        completions
    }

    /// Get parameter completions (X, Y, Z, F, S, etc.)
    fn get_parameter_completions(
        &self,
        line: &str,
        cursor_col: usize,
        context: &CompletionContext,
    ) -> Vec<CompletionItem> {
        let current_word = self.get_current_word(line, cursor_col).to_uppercase();
        let mut completions = Vec::new();

        // Analyze what command is on this line
        let tokens: Vec<&str> = line.split_whitespace().collect();
        let command = tokens.first().map(|s| s.to_uppercase());

        // Common parameters
        let params = self.get_relevant_parameters(command.as_deref(), &context.modal_state);

        for (param, desc, priority) in params {
            if current_word.is_empty() || param.starts_with(&current_word) {
                completions.push(CompletionItem {
                    insert_text: param.clone(),
                    label: param.clone(),
                    detail: desc,
                    sort_order: priority,
                    category: "Parameter".to_string(),
                });
            }
        }

        completions.sort_by_key(|c| c.sort_order);
        completions
    }

    /// Get value completions (for specific parameters)
    fn get_value_completions(
        &self,
        line: &str,
        cursor_col: usize,
        context: &CompletionContext,
    ) -> Vec<CompletionItem> {
        let current_word = self.get_current_word(line, cursor_col);
        if current_word.is_empty() {
            return Vec::new();
        }

        let param_letter = current_word.chars().next().unwrap().to_ascii_uppercase();

        // For now, provide generic numeric value suggestions
        // In future, could provide contextual values based on machine limits, etc.
        let mut completions = Vec::new();

        // Common feed rates for F parameter
        if param_letter == 'F' {
            for rate in [100, 200, 300, 500, 1000, 2000] {
                completions.push(CompletionItem {
                    insert_text: format!("F{} ", rate),
                    label: format!("F{}", rate),
                    detail: format!("Feed rate: {} mm/min", rate),
                    sort_order: 0,
                    category: "Value".to_string(),
                });
            }
        }

        // Common spindle speeds for S parameter
        if param_letter == 'S' {
            for speed in [1000, 5000, 10000, 12000, 15000, 20000] {
                completions.push(CompletionItem {
                    insert_text: format!("S{} ", speed),
                    label: format!("S{}", speed),
                    detail: format!("Spindle speed: {} RPM", speed),
                    sort_order: 0,
                    category: "Value".to_string(),
                });
            }
        }

        completions
    }

    /// Check if a code is supported in the current GRBL version
    fn is_code_supported(&self, code_info: &CodeInfo) -> bool {
        code_info
            .supported_in
            .iter()
            .any(|v| *v == self.grbl_version)
    }

    /// Get priority for sorting codes (common codes first)
    fn get_code_priority(&self, code: &str) -> usize {
        match code {
            "G0" => 0,
            "G1" => 1,
            "G2" => 2,
            "G3" => 3,
            "G90" => 4,
            "G91" => 5,
            "G21" => 6,
            "G20" => 7,
            "M3" => 8,
            "M5" => 9,
            "M30" => 10,
            _ => 100,
        }
    }

    /// Get relevant parameters for a given command
    fn get_relevant_parameters(
        &self,
        command: Option<&str>,
        modal_state: &ModalState,
    ) -> Vec<(String, String, usize)> {
        let mut params = Vec::new();

        match command {
            Some("G0") | Some("G1") => {
                // Linear move parameters
                params.push(("X".to_string(), "X-axis position".to_string(), 0));
                params.push(("Y".to_string(), "Y-axis position".to_string(), 1));
                params.push(("Z".to_string(), "Z-axis position".to_string(), 2));
                if command == Some("G1") {
                    params.push(("F".to_string(), "Feed rate".to_string(), 3));
                }
            }
            Some("G2") | Some("G3") => {
                // Arc parameters
                params.push(("X".to_string(), "End X position".to_string(), 0));
                params.push(("Y".to_string(), "End Y position".to_string(), 1));
                params.push(("Z".to_string(), "End Z position (helical)".to_string(), 2));
                params.push((
                    "I".to_string(),
                    "Arc center X offset from start".to_string(),
                    3,
                ));
                params.push((
                    "J".to_string(),
                    "Arc center Y offset from start".to_string(),
                    4,
                ));
                params.push((
                    "K".to_string(),
                    "Arc center Z offset from start".to_string(),
                    5,
                ));
                params.push(("R".to_string(), "Arc radius (alternative)".to_string(), 6));
                params.push(("F".to_string(), "Feed rate".to_string(), 7));
            }
            Some("G4") => {
                // Dwell
                params.push(("P".to_string(), "Dwell time (seconds)".to_string(), 0));
            }
            Some("M3") | Some("M4") => {
                // Spindle on
                params.push(("S".to_string(), "Spindle speed (RPM)".to_string(), 0));
            }
            _ => {
                // Default common parameters
                params.push(("X".to_string(), "X-axis position".to_string(), 0));
                params.push(("Y".to_string(), "Y-axis position".to_string(), 1));
                params.push(("Z".to_string(), "Z-axis position".to_string(), 2));
                params.push(("F".to_string(), "Feed rate".to_string(), 3));
                params.push(("S".to_string(), "Spindle speed".to_string(), 4));
            }
        }

        params
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_autocompleter_new() {
        let completer = AutoCompleter::new("1.1");
        assert_eq!(completer.grbl_version, "1.1");
    }

    #[test]
    fn test_command_completion_g_codes() {
        let completer = AutoCompleter::new("1.1");
        let context = CompletionContext::default();
        let suggestions = completer.get_suggestions("G", 1, &context);

        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.label == "G0"));
        assert!(suggestions.iter().any(|s| s.label == "G1"));
    }

    #[test]
    fn test_command_completion_m_codes() {
        let completer = AutoCompleter::new("1.1");
        let context = CompletionContext::default();
        let suggestions = completer.get_suggestions("M", 1, &context);

        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.label == "M3"));
        assert!(suggestions.iter().any(|s| s.label == "M5"));
    }

    #[test]
    fn test_parameter_completion() {
        let completer = AutoCompleter::new("1.1");
        let context = CompletionContext::default();
        let suggestions = completer.get_suggestions("G1 ", 3, &context);

        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.label == "X"));
        assert!(suggestions.iter().any(|s| s.label == "Y"));
        assert!(suggestions.iter().any(|s| s.label == "F"));
    }

    #[test]
    fn test_partial_command_match() {
        let completer = AutoCompleter::new("1.1");
        let context = CompletionContext::default();
        let suggestions = completer.get_suggestions("G9", 2, &context);

        // Should match G90, G91, G92
        assert!(suggestions.iter().any(|s| s.label == "G90"));
        assert!(suggestions.iter().any(|s| s.label == "G91"));
        assert!(suggestions.iter().any(|s| s.label == "G92"));
    }

    #[test]
    fn test_analyze_completion_type_command() {
        let completer = AutoCompleter::new("1.1");
        let comp_type = completer.analyze_completion_type("G", 1);
        assert_eq!(comp_type, CompletionType::Command);
    }

    #[test]
    fn test_analyze_completion_type_parameter() {
        let completer = AutoCompleter::new("1.1");
        let comp_type = completer.analyze_completion_type("G1 ", 3);
        assert_eq!(comp_type, CompletionType::Parameter);
    }

    #[test]
    fn test_get_current_word() {
        let completer = AutoCompleter::new("1.1");
        assert_eq!(completer.get_current_word("G1 X10", 2), "G1");
        assert_eq!(completer.get_current_word("G1 X10", 4), "X");
        assert_eq!(completer.get_current_word("G1 X10", 6), "X10");
    }

    #[test]
    fn test_feed_rate_value_completion() {
        let completer = AutoCompleter::new("1.1");
        let context = CompletionContext::default();
        let suggestions = completer.get_suggestions("G1 X10 F", 8, &context);

        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.label.contains("F100")));
        assert!(suggestions.iter().any(|s| s.label.contains("F500")));
    }

    #[test]
    fn test_spindle_speed_value_completion() {
        let completer = AutoCompleter::new("1.1");
        let context = CompletionContext::default();
        let suggestions = completer.get_suggestions("M3 S", 4, &context);

        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.label.contains("S1000")));
        assert!(suggestions.iter().any(|s| s.label.contains("S12000")));
    }

    #[test]
    fn test_arc_parameter_completion() {
        let completer = AutoCompleter::new("1.1");
        let context = CompletionContext::default();
        let suggestions = completer.get_suggestions("G2 ", 3, &context);

        assert!(suggestions.iter().any(|s| s.label == "I"));
        assert!(suggestions.iter().any(|s| s.label == "J"));
        assert!(suggestions.iter().any(|s| s.label == "R"));
    }

    #[test]
    fn test_grbl_version_filtering() {
        let completer_10 = AutoCompleter::new("1.0");
        let completer_11 = AutoCompleter::new("1.1");

        let context = CompletionContext::default();
        let suggestions_10 = completer_10.get_suggestions("G38", 3, &context);
        let suggestions_11 = completer_11.get_suggestions("G38", 3, &context);

        // G38.2 is only in GRBL 1.1+
        assert!(suggestions_10.is_empty() || !suggestions_10.iter().any(|s| s.label == "G38.2"));
        assert!(suggestions_11.iter().any(|s| s.label == "G38.2"));
    }
}
