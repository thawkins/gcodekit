//! Editor configuration and persistence
//!
//! This module handles editor settings, rule configuration, and persistence
//! to disk for maintaining user preferences across sessions.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Editor configuration that can be persisted
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorConfig {
    /// GRBL version for validation (1.0, 1.1, 1.2)
    pub grbl_version: String,
    /// Enable/disable validation rules
    pub validation_enabled: bool,
    /// Individual rule states
    pub rule_states: Vec<RuleState>,
    /// Enable virtualization for large files
    pub enable_virtualization: bool,
    /// Virtualization threshold (lines)
    pub virtualization_threshold: usize,
    /// Enable auto-completion
    pub enable_autocomplete: bool,
    /// Line height for rendering
    pub line_height: f32,
    /// Enable syntax highlighting
    pub enable_syntax_highlighting: bool,
    /// Show line numbers
    pub show_line_numbers: bool,
    /// Tab size
    pub tab_size: usize,
    /// Use spaces instead of tabs
    pub use_spaces: bool,
    /// Auto-save interval (seconds, 0 = disabled)
    pub auto_save_interval: u64,
}

/// State of an individual validation rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleState {
    pub id: String,
    pub enabled: bool,
    pub severity: RuleSeverity,
}

/// Severity level for rules
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RuleSeverity {
    Error,
    Warning,
    Info,
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            grbl_version: "1.1".to_string(),
            validation_enabled: true,
            rule_states: vec![
                RuleState {
                    id: "unknown_code".to_string(),
                    enabled: true,
                    severity: RuleSeverity::Error,
                },
                RuleState {
                    id: "empty_line".to_string(),
                    enabled: true,
                    severity: RuleSeverity::Info,
                },
            ],
            enable_virtualization: true,
            virtualization_threshold: 500,
            enable_autocomplete: true,
            line_height: 14.0,
            enable_syntax_highlighting: true,
            show_line_numbers: true,
            tab_size: 4,
            use_spaces: true,
            auto_save_interval: 300, // 5 minutes
        }
    }
}

impl EditorConfig {
    /// Create a new config with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the config file path
    fn config_path() -> PathBuf {
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("gcodekit");
        fs::create_dir_all(&path).ok();
        path.push("editor_config.json");
        path
    }

    /// Load configuration from disk
    pub fn load() -> Result<Self, String> {
        let path = Self::config_path();
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read config: {}", e))?;
        
        serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse config: {}", e))
    }

    /// Save configuration to disk
    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path();
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;
        
        fs::write(&path, content)
            .map_err(|e| format!("Failed to write config: {}", e))
    }

    /// Get rule state by ID
    pub fn get_rule_state(&self, id: &str) -> Option<&RuleState> {
        self.rule_states.iter().find(|r| r.id == id)
    }

    /// Update rule state
    pub fn set_rule_state(&mut self, id: String, enabled: bool, severity: RuleSeverity) {
        if let Some(state) = self.rule_states.iter_mut().find(|r| r.id == id) {
            state.enabled = enabled;
            state.severity = severity;
        } else {
            self.rule_states.push(RuleState {
                id,
                enabled,
                severity,
            });
        }
    }

    /// Apply configuration to editor state
    pub fn apply_to_editor(&self, editor: &mut crate::gcodeedit::GcodeEditorState) {
        // Apply GRBL version
        editor.rules.grbl_version = self.grbl_version.clone();

        // Apply rule states
        for rule_state in &self.rule_states {
            if rule_state.enabled {
                editor.rules.enable_rule(&rule_state.id);
            } else {
                editor.rules.disable_rule(&rule_state.id);
            }
        }

        // Apply virtualization settings
        editor.enable_virtualization = self.enable_virtualization;
        editor.virtualized_config.line_height = self.line_height;
    }
}

/// UI for editing configuration
pub struct ConfigUI {
    config: EditorConfig,
    show_window: bool,
    config_changed: bool,
}

impl ConfigUI {
    /// Create a new config UI
    pub fn new(config: EditorConfig) -> Self {
        Self {
            config,
            show_window: false,
            config_changed: false,
        }
    }

    /// Show the configuration window
    pub fn show(&mut self, ctx: &egui::Context) -> Option<EditorConfig> {
        if !self.show_window {
            return None;
        }

        let mut result = None;
        
        egui::Window::new("Editor Configuration")
            .open(&mut self.show_window)
            .resizable(true)
            .default_width(500.0)
            .show(ctx, |ui| {
                ui.label("Configuration settings");
                // Simplified UI for now - full implementation would go here
                if ui.button("Close").clicked() {
                    result = Some(self.config.clone());
                }
            });

        result
    }

    /// Toggle window visibility
    pub fn toggle(&mut self) {
        self.show_window = !self.show_window;
    }

    /// Show the window
    pub fn open(&mut self) {
        self.show_window = true;
    }

    /// Check if window is open
    pub fn is_open(&self) -> bool {
        self.show_window
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = EditorConfig::default();
        assert_eq!(config.grbl_version, "1.1");
        assert!(config.validation_enabled);
        assert!(config.enable_virtualization);
        assert_eq!(config.virtualization_threshold, 500);
    }

    #[test]
    fn test_get_rule_state() {
        let config = EditorConfig::default();
        let state = config.get_rule_state("unknown_code");
        assert!(state.is_some());
        assert!(
            state.expect("rule state should be present").enabled,
            "rule should be enabled by default"
        );
    }

    #[test]
    fn test_set_rule_state() {
        let mut config = EditorConfig::default();
        config.set_rule_state(
            "test_rule".to_string(),
            false,
            RuleSeverity::Warning,
        );

        let state = config.get_rule_state("test_rule");
        assert!(state.is_some());
        let state = state.expect("rule state should be present");
        assert!(!state.enabled);
        assert_eq!(state.severity, RuleSeverity::Warning);
    }

    #[test]
    fn test_serialization() {
        let config = EditorConfig::default();
        let json = serde_json::to_string(&config)
            .expect("config serialization should succeed");
        let deserialized: EditorConfig = serde_json::from_str(&json)
            .expect("config deserialization should succeed");

        assert_eq!(config.grbl_version, deserialized.grbl_version);
        assert_eq!(config.validation_enabled, deserialized.validation_enabled);
    }
}
