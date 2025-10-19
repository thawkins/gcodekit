//! Gamepad Configuration UI Widget
//!
//! Provides the user interface for configuring gamepad button mapping,
//! deadzone, and jogging sensitivity settings.

use crate::input::gamepad::{GamepadButton, GamepadController, GamepadMapping};
use crate::input::Action;
use egui::{Button, ComboBox, Slider, TextEdit, Ui};
use std::collections::HashMap;

/// State for gamepad configuration UI
#[derive(Debug, Clone)]
pub struct GamepadSettingsUiState {
    /// Gamepad controller reference
    pub controller: GamepadController,
    /// Whether gamepad settings panel is visible
    pub show_gamepad_panel: bool,
    /// Current mapping being edited
    pub current_mapping: GamepadMapping,
    /// Button currently being configured
    pub selected_button: Option<GamepadButton>,
    /// Last status message
    pub status_message: Option<String>,
    /// Gamepad connected status
    pub gamepad_connected: bool,
}

impl Default for GamepadSettingsUiState {
    fn default() -> Self {
        Self::new()
    }
}

impl GamepadSettingsUiState {
    /// Create new gamepad settings UI state
    pub fn new() -> Self {
        Self {
            controller: GamepadController::new(),
            show_gamepad_panel: false,
            current_mapping: GamepadMapping::default(),
            selected_button: None,
            status_message: None,
            gamepad_connected: false,
        }
    }

    /// Reset to default mapping
    pub fn reset_to_default(&mut self) {
        self.current_mapping = GamepadMapping::default();
        self.status_message = Some("Reset to default gamepad mapping".to_string());
    }

    /// Save mapping to controller
    pub fn apply_mapping(&mut self) {
        self.controller.set_mapping(self.current_mapping.clone());
        self.status_message = Some("Gamepad mapping applied".to_string());
    }

    /// Clear a button mapping
    pub fn clear_button_mapping(&mut self, button: GamepadButton) {
        self.current_mapping.button_map.remove(&button);
        self.status_message = Some(format!("Removed mapping for {:?}", button));
    }

    /// Set a button mapping
    pub fn set_button_mapping(&mut self, button: GamepadButton, action: Action) {
        self.current_mapping.button_map.insert(button, action);
        self.status_message = Some(format!("Mapped {:?} to {:?}", button, action));
    }
}

/// Render the gamepad settings panel
pub fn show_gamepad_settings(ui: &mut Ui, state: &mut GamepadSettingsUiState) {
    ui.group(|ui| {
        ui.vertical(|ui| {
            ui.heading("🎮 Gamepad Settings");

            // Connection status
            let connection_text = if state.gamepad_connected {
                "✅ Gamepad Connected"
            } else {
                "❌ No Gamepad Detected"
            };
            ui.label(connection_text);

            ui.separator();

            // Deadzone slider
            ui.horizontal(|ui| {
                ui.label("Deadzone:");
                ui.add(
                    Slider::new(&mut state.current_mapping.deadzone, 0.0..=0.5)
                        .show_value(true)
                );
                ui.label(format!("{:.2}", state.current_mapping.deadzone));
            });

            // Jog sensitivity slider
            ui.horizontal(|ui| {
                ui.label("Jog Sensitivity:");
                ui.add(
                    Slider::new(&mut state.current_mapping.jog_sensitivity, 0.1..=10.0)
                        .show_value(true)
                );
                ui.label(format!("{}x", state.current_mapping.jog_sensitivity));
            });

            ui.separator();

            // Enable/disable stick jogging
            ui.horizontal(|ui| {
                ui.checkbox(&mut state.current_mapping.enable_left_stick_jog, "Enable Left Stick X/Y Jog");
            });
            ui.horizontal(|ui| {
                ui.checkbox(&mut state.current_mapping.enable_right_stick_jog, "Enable Right Stick Z Jog");
            });

            ui.separator();

            // Button mapping controls
            ui.label("Button Mapping:");
            button_mapping_ui(ui, state);

            ui.separator();

            // Status message
            if let Some(msg) = &state.status_message {
                ui.colored_label(egui::Color32::LIGHT_GREEN, msg);
            }

            // Control buttons
            ui.horizontal(|ui| {
                if ui.button("Apply").clicked() {
                    state.apply_mapping();
                }
                if ui.button("Reset to Default").clicked() {
                    state.reset_to_default();
                }
            });
        });
    });
}

/// Render button mapping configuration UI
fn button_mapping_ui(ui: &mut Ui, state: &mut GamepadSettingsUiState) {
    ui.group(|ui| {
        // List all gamepad buttons with current mappings
        let buttons = vec![
            GamepadButton::North,
            GamepadButton::East,
            GamepadButton::West,
            GamepadButton::South,
            GamepadButton::LeftShoulder,
            GamepadButton::RightShoulder,
            GamepadButton::LeftStickPress,
            GamepadButton::RightStickPress,
            GamepadButton::Start,
            GamepadButton::Back,
            GamepadButton::Guide,
        ];

        for button in buttons {
            ui.horizontal(|ui| {
                let button_name = format!("{:?}", button);
                ui.label(button_name);

                let current_action = state
                    .current_mapping
                    .button_map
                    .get(&button)
                    .map(|a| format!("{:?}", a))
                    .unwrap_or_else(|| "None".to_string());

                ComboBox::from_id_salt(format!("button_map_{:?}", button))
                    .selected_text(current_action.clone())
                    .show_ui(ui, |ui| {
                        if ui.selectable_value(
                            &mut state.selected_button,
                            Some(button),
                            "None",
                        ).clicked() {
                            state.clear_button_mapping(button);
                        }
                        
                        // List possible actions
                        let actions = vec![
                            Action::Home,
                            Action::JogXPlus,
                            Action::JogXMinus,
                            Action::JogYPlus,
                            Action::JogYMinus,
                            Action::JogZPlus,
                            Action::JogZMinus,
                            Action::ProbeZ,
                            Action::FeedHold,
                            Action::Resume,
                            Action::Reset,
                            Action::ZoomIn,
                            Action::ZoomOut,
                        ];

                        for action in actions {
                            let action_str = format!("{:?}", action);
                            if ui.selectable_value(
                                &mut state.selected_button,
                                Some(button),
                                action_str,
                            ).clicked() {
                                state.set_button_mapping(button, action);
                            }
                        }
                    });

                if ui.button("X").clicked() {
                    state.clear_button_mapping(button);
                }
            });
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gamepad_settings_ui_state_default() {
        let state = GamepadSettingsUiState::default();
        assert!(!state.show_gamepad_panel);
        assert!(!state.gamepad_connected);
        assert!(state.status_message.is_none());
    }

    #[test]
    fn test_reset_to_default() {
        let mut state = GamepadSettingsUiState::new();
        state.current_mapping.deadzone = 0.5;
        state.reset_to_default();
        
        assert_eq!(state.current_mapping.deadzone, GamepadMapping::default().deadzone);
        assert!(state.status_message.is_some());
    }

    #[test]
    fn test_apply_mapping() {
        let mut state = GamepadSettingsUiState::new();
        state.apply_mapping();
        
        assert!(state.status_message.is_some());
        assert!(state.controller.get_state().unwrap().connected == false);
    }

    #[test]
    fn test_set_button_mapping() {
        let mut state = GamepadSettingsUiState::new();
        state.set_button_mapping(GamepadButton::North, Action::ProbeZ);
        
        assert_eq!(
            state.current_mapping.button_map.get(&GamepadButton::North),
            Some(&Action::ProbeZ)
        );
    }

    #[test]
    fn test_clear_button_mapping() {
        let mut state = GamepadSettingsUiState::new();
        state.set_button_mapping(GamepadButton::East, Action::Reset);
        state.clear_button_mapping(GamepadButton::East);
        
        assert!(state.current_mapping.button_map.get(&GamepadButton::East).is_none());
    }
}
