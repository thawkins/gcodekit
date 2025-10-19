//! Gamepad and joystick input handling for gcodekit.
//!
//! This module provides cross-platform gamepad/joystick support via gilrs for machine control,
//! with customizable button mapping and analog stick control for jogging.

use crate::input::Action;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Represents a gamepad button.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GamepadButton {
    South,      // PlayStation Circle / Xbox B
    East,       // PlayStation Triangle / Xbox Y
    West,       // PlayStation Square / Xbox X
    North,      // PlayStation Cross / Xbox A
    LeftShoulder,
    RightShoulder,
    LeftStickPress,
    RightStickPress,
    Start,
    Back,
    Guide,
}

/// Represents gamepad analog stick state.
#[derive(Clone, Debug)]
pub struct AnalogStickState {
    pub x: f32,  // -1.0 to 1.0
    pub y: f32,  // -1.0 to 1.0
}

impl AnalogStickState {
    /// Create a new analog stick state.
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x: x.clamp(-1.0, 1.0),
            y: y.clamp(-1.0, 1.0),
        }
    }

    /// Check if stick is beyond deadzone threshold.
    pub fn is_active(&self, deadzone: f32) -> bool {
        (self.x.abs() > deadzone) || (self.y.abs() > deadzone)
    }

    /// Get magnitude of stick input.
    pub fn magnitude(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
}

/// Represents the current state of a gamepad.
#[derive(Clone, Debug)]
pub struct GamepadState {
    pub buttons: HashMap<GamepadButton, bool>,
    pub left_stick: AnalogStickState,
    pub right_stick: AnalogStickState,
    pub left_trigger: f32,   // 0.0 to 1.0
    pub right_trigger: f32,  // 0.0 to 1.0
    pub connected: bool,
}

impl Default for GamepadState {
    fn default() -> Self {
        Self {
            buttons: HashMap::new(),
            left_stick: AnalogStickState::new(0.0, 0.0),
            right_stick: AnalogStickState::new(0.0, 0.0),
            left_trigger: 0.0,
            right_trigger: 0.0,
            connected: false,
        }
    }
}

/// Gamepad button to action mapping configuration.
#[derive(Clone, Debug)]
pub struct GamepadMapping {
    /// Maps gamepad buttons to actions
    pub button_map: HashMap<GamepadButton, Action>,
    /// Analog stick deadzone (default 0.15)
    pub deadzone: f32,
    /// Enable left analog stick for X/Y jogging
    pub enable_left_stick_jog: bool,
    /// Enable right analog stick for Z jogging
    pub enable_right_stick_jog: bool,
    /// Jog step size multiplier for analog stick (0.1 to 10.0)
    pub jog_sensitivity: f32,
}

impl Default for GamepadMapping {
    fn default() -> Self {
        let mut button_map = HashMap::new();
        // Default button mappings
        button_map.insert(GamepadButton::North, Action::ProbeZ);
        button_map.insert(GamepadButton::East, Action::Reset);
        button_map.insert(GamepadButton::West, Action::Home);
        button_map.insert(GamepadButton::South, Action::FeedHold);
        button_map.insert(GamepadButton::LeftShoulder, Action::ZoomOut);
        button_map.insert(GamepadButton::RightShoulder, Action::ZoomIn);

        Self {
            button_map,
            deadzone: 0.15,
            enable_left_stick_jog: true,
            enable_right_stick_jog: true,
            jog_sensitivity: 1.0,
        }
    }
}

/// Gamepad input processor and state manager.
#[derive(Clone, Debug)]
pub struct GamepadController {
    state: Arc<Mutex<GamepadState>>,
    mapping: GamepadMapping,
}

impl GamepadController {
    /// Create a new gamepad controller with default mapping.
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(GamepadState::default())),
            mapping: GamepadMapping::default(),
        }
    }

    /// Create a new gamepad controller with custom mapping.
    pub fn with_mapping(mapping: GamepadMapping) -> Self {
        Self {
            state: Arc::new(Mutex::new(GamepadState::default())),
            mapping,
        }
    }

    /// Update gamepad state with button press.
    pub fn set_button(&self, button: GamepadButton, pressed: bool) {
        if let Ok(mut state) = self.state.lock() {
            state.buttons.insert(button, pressed);
        }
    }

    /// Update left analog stick state.
    pub fn set_left_stick(&self, x: f32, y: f32) {
        if let Ok(mut state) = self.state.lock() {
            state.left_stick = AnalogStickState::new(x, y);
        }
    }

    /// Update right analog stick state.
    pub fn set_right_stick(&self, x: f32, y: f32) {
        if let Ok(mut state) = self.state.lock() {
            state.right_stick = AnalogStickState::new(x, y);
        }
    }

    /// Update left trigger state.
    pub fn set_left_trigger(&self, value: f32) {
        if let Ok(mut state) = self.state.lock() {
            state.left_trigger = value.clamp(0.0, 1.0);
        }
    }

    /// Update right trigger state.
    pub fn set_right_trigger(&self, value: f32) {
        if let Ok(mut state) = self.state.lock() {
            state.right_trigger = value.clamp(0.0, 1.0);
        }
    }

    /// Set gamepad connected status.
    pub fn set_connected(&self, connected: bool) {
        if let Ok(mut state) = self.state.lock() {
            state.connected = connected;
        }
    }

    /// Get current gamepad state (non-destructive).
    pub fn get_state(&self) -> Option<GamepadState> {
        self.state.lock().ok().map(|s| s.clone())
    }

    /// Get all pressed button actions.
    pub fn get_pressed_actions(&self) -> Vec<Action> {
        let mut actions = Vec::new();
        if let Ok(state) = self.state.lock() {
            for (button, pressed) in &state.buttons {
                if *pressed {
                    if let Some(action) = self.mapping.button_map.get(button) {
                        actions.push(action.clone());
                    }
                }
            }
        }
        actions
    }

    /// Check if a button is currently pressed.
    pub fn is_button_pressed(&self, button: GamepadButton) -> bool {
        self.state
            .lock()
            .ok()
            .map(|s| s.buttons.get(&button).copied().unwrap_or(false))
            .unwrap_or(false)
    }

    /// Get left stick state for jogging.
    pub fn get_left_stick_jog(&self) -> Option<(f32, f32)> {
        self.state.lock().ok().and_then(|s| {
            if s.left_stick.is_active(self.mapping.deadzone) && self.mapping.enable_left_stick_jog {
                Some((s.left_stick.x, -s.left_stick.y))
            } else {
                None
            }
        })
    }

    /// Get right stick state for Z-axis jogging.
    pub fn get_right_stick_jog(&self) -> Option<f32> {
        self.state.lock().ok().and_then(|s| {
            if s.right_stick.is_active(self.mapping.deadzone) && self.mapping.enable_right_stick_jog {
                Some(-s.right_stick.y)
            } else {
                None
            }
        })
    }

    /// Update gamepad mapping.
    pub fn set_mapping(&mut self, mapping: GamepadMapping) {
        self.mapping = mapping;
    }

    /// Get current button mapping.
    pub fn get_mapping(&self) -> GamepadMapping {
        self.mapping.clone()
    }
}

impl Default for GamepadController {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gamepad_state_default() {
        let state = GamepadState::default();
        assert!(!state.connected);
        assert_eq!(state.left_trigger, 0.0);
        assert_eq!(state.right_trigger, 0.0);
    }

    #[test]
    fn test_analog_stick_clamping() {
        let stick = AnalogStickState::new(1.5, -1.5);
        assert_eq!(stick.x, 1.0);
        assert_eq!(stick.y, -1.0);
    }

    #[test]
    fn test_analog_stick_magnitude() {
        let stick = AnalogStickState::new(0.6, 0.8);
        let mag = stick.magnitude();
        assert!((mag - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_analog_stick_active() {
        let stick_inactive = AnalogStickState::new(0.1, 0.1);
        let stick_active = AnalogStickState::new(0.2, 0.2);

        assert!(!stick_inactive.is_active(0.15));
        assert!(stick_active.is_active(0.15));
    }

    #[test]
    fn test_gamepad_controller_button() {
        let controller = GamepadController::new();
        controller.set_button(GamepadButton::North, true);
        
        assert!(controller.is_button_pressed(GamepadButton::North));
        assert!(!controller.is_button_pressed(GamepadButton::South));
    }

    #[test]
    fn test_gamepad_controller_sticks() {
        let controller = GamepadController::new();
        controller.set_left_stick(0.5, 0.7);
        controller.set_right_stick(-0.3, -0.9);

        let state = controller.get_state().unwrap();
        assert_eq!(state.left_stick.x, 0.5);
        assert_eq!(state.left_stick.y, 0.7);
        assert_eq!(state.right_stick.x, -0.3);
        assert_eq!(state.right_stick.y, -0.9);
    }

    #[test]
    fn test_gamepad_mapping_default() {
        let mapping = GamepadMapping::default();
        assert_eq!(mapping.deadzone, 0.15);
        assert!(mapping.enable_left_stick_jog);
        assert!(mapping.enable_right_stick_jog);
        assert!(!mapping.button_map.is_empty());
    }

    #[test]
    fn test_gamepad_controller_pressed_actions() {
        let controller = GamepadController::new();
        controller.set_button(GamepadButton::North, true);
        controller.set_button(GamepadButton::West, true);

        let actions = controller.get_pressed_actions();
        assert!(!actions.is_empty());
        assert!(actions.contains(&Action::ProbeZ));
        assert!(actions.contains(&Action::Home));
    }

    #[test]
    fn test_gamepad_controller_jog() {
        let controller = GamepadController::new();
        controller.set_left_stick(0.5, 0.7);
        
        let jog = controller.get_left_stick_jog();
        assert!(jog.is_some());
        let (x, y) = jog.unwrap();
        assert_eq!(x, 0.5);
        assert_eq!(y, -0.7);
    }

    #[test]
    fn test_gamepad_controller_triggers() {
        let controller = GamepadController::new();
        controller.set_left_trigger(0.5);
        controller.set_right_trigger(0.8);

        let state = controller.get_state().unwrap();
        assert_eq!(state.left_trigger, 0.5);
        assert_eq!(state.right_trigger, 0.8);
    }

    #[test]
    fn test_gamepad_controller_connected() {
        let controller = GamepadController::new();
        assert!(!controller.get_state().unwrap().connected);
        
        controller.set_connected(true);
        assert!(controller.get_state().unwrap().connected);
    }
}
