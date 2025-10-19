//! Input handling module.
//!
//! This module manages keyboard shortcuts, user actions, and input processing
//! for the gcodekit application, providing customizable key bindings and gamepad support.

pub mod gamepad;

use eframe::egui;
use std::collections::HashMap;

pub use gamepad::{AnalogStickState, GamepadButton, GamepadController, GamepadMapping, GamepadState};

/// Represents user actions that can be triggered by keyboard shortcuts or UI elements.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Action {
    OpenFile,
    SaveFile,
    ExportGcode,
    ImportVector,
    Undo,
    Redo,
    ZoomIn,
    ZoomOut,
    Home,
    JogXPlus,
    JogXMinus,
    JogYPlus,
    JogYMinus,
    JogZPlus,
    JogZMinus,
    ProbeZ,
    FeedHold,
    Resume,
    Reset,
}

/// Represents a keyboard shortcut binding consisting of a key and modifier combination.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct KeyBinding {
    pub key: egui::Key,
    pub modifiers: egui::Modifiers,
}

/// Creates the default key bindings for all actions.
pub fn create_default_keybindings() -> HashMap<Action, KeyBinding> {
    let mut map = HashMap::new();
    map.insert(
        Action::OpenFile,
        KeyBinding {
            key: egui::Key::O,
            modifiers: egui::Modifiers::CTRL,
        },
    );
    map.insert(
        Action::SaveFile,
        KeyBinding {
            key: egui::Key::S,
            modifiers: egui::Modifiers::CTRL,
        },
    );
    map.insert(
        Action::ExportGcode,
        KeyBinding {
            key: egui::Key::E,
            modifiers: egui::Modifiers::CTRL,
        },
    );
    map.insert(
        Action::ImportVector,
        KeyBinding {
            key: egui::Key::I,
            modifiers: egui::Modifiers::CTRL,
        },
    );
    map.insert(
        Action::Undo,
        KeyBinding {
            key: egui::Key::Z,
            modifiers: egui::Modifiers::CTRL,
        },
    );
    map.insert(
        Action::Redo,
        KeyBinding {
            key: egui::Key::Y,
            modifiers: egui::Modifiers::CTRL,
        },
    );
    map.insert(
        Action::ZoomIn,
        KeyBinding {
            key: egui::Key::Plus,
            modifiers: egui::Modifiers::CTRL,
        },
    );
    map.insert(
        Action::ZoomOut,
        KeyBinding {
            key: egui::Key::Minus,
            modifiers: egui::Modifiers::CTRL,
        },
    );
    map.insert(
        Action::Home,
        KeyBinding {
            key: egui::Key::H,
            modifiers: egui::Modifiers::CTRL,
        },
    );
    map.insert(
        Action::JogXPlus,
        KeyBinding {
            key: egui::Key::ArrowRight,
            modifiers: egui::Modifiers::ALT,
        },
    );
    map.insert(
        Action::JogXMinus,
        KeyBinding {
            key: egui::Key::ArrowLeft,
            modifiers: egui::Modifiers::ALT,
        },
    );
    map.insert(
        Action::JogYPlus,
        KeyBinding {
            key: egui::Key::ArrowUp,
            modifiers: egui::Modifiers::ALT,
        },
    );
    map.insert(
        Action::JogYMinus,
        KeyBinding {
            key: egui::Key::ArrowDown,
            modifiers: egui::Modifiers::ALT,
        },
    );
    map.insert(
        Action::JogZPlus,
        KeyBinding {
            key: egui::Key::PageUp,
            modifiers: egui::Modifiers::ALT,
        },
    );
    map.insert(
        Action::JogZMinus,
        KeyBinding {
            key: egui::Key::PageDown,
            modifiers: egui::Modifiers::ALT,
        },
    );
    map.insert(
        Action::ProbeZ,
        KeyBinding {
            key: egui::Key::P,
            modifiers: egui::Modifiers::CTRL,
        },
    );
    map.insert(
        Action::FeedHold,
        KeyBinding {
            key: egui::Key::Space,
            modifiers: egui::Modifiers::CTRL,
        },
    );
    map.insert(
        Action::Resume,
        KeyBinding {
            key: egui::Key::Space,
            modifiers: egui::Modifiers::SHIFT,
        },
    );
    map.insert(
        Action::Reset,
        KeyBinding {
            key: egui::Key::R,
            modifiers: egui::Modifiers::ALT,
        },
    );
    map
}

/// Handles keyboard shortcuts and executes corresponding actions.
/// This function should be called from the main update loop to process input.
///
/// # Arguments
/// * `ctx` - The egui context for input handling
/// * `keybindings` - The current key binding configuration
/// * `action_handler` - A closure that handles the executed actions
pub fn handle_keyboard_shortcuts<F>(
    ctx: &egui::Context,
    keybindings: &HashMap<Action, KeyBinding>,
    mut action_handler: F,
) where
    F: FnMut(&Action),
{
    for (action, binding) in keybindings {
        if ctx.input(|i| i.key_pressed(binding.key) && i.modifiers == binding.modifiers) {
            action_handler(action);
        }
    }
}
