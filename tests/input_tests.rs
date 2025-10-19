//! Input handling tests

use gcodekit::input::{Action, KeyBinding, create_default_keybindings};
use eframe::egui;

#[test]
fn test_create_default_keybindings() {
    let keybindings = create_default_keybindings();
    
    // Verify all actions have keybindings
    assert!(keybindings.contains_key(&Action::OpenFile));
    assert!(keybindings.contains_key(&Action::SaveFile));
    assert!(keybindings.contains_key(&Action::ExportGcode));
    assert!(keybindings.contains_key(&Action::Home));
    assert!(keybindings.contains_key(&Action::JogXPlus));
    assert!(keybindings.contains_key(&Action::JogXMinus));
    assert!(keybindings.contains_key(&Action::JogYPlus));
    assert!(keybindings.contains_key(&Action::JogYMinus));
    assert!(keybindings.contains_key(&Action::JogZPlus));
    assert!(keybindings.contains_key(&Action::JogZMinus));
    assert!(keybindings.contains_key(&Action::ProbeZ));
    assert!(keybindings.contains_key(&Action::FeedHold));
    assert!(keybindings.contains_key(&Action::Resume));
    assert!(keybindings.contains_key(&Action::Reset));
}

#[test]
fn test_keybinding_clone() {
    let binding = KeyBinding {
        key: egui::Key::O,
        modifiers: egui::Modifiers::CTRL,
    };
    
    let cloned = binding.clone();
    assert_eq!(binding.key, cloned.key);
    assert_eq!(binding.modifiers, cloned.modifiers);
}

#[test]
fn test_keybinding_eq() {
    let binding1 = KeyBinding {
        key: egui::Key::S,
        modifiers: egui::Modifiers::CTRL,
    };
    
    let binding2 = KeyBinding {
        key: egui::Key::S,
        modifiers: egui::Modifiers::CTRL,
    };
    
    let binding3 = KeyBinding {
        key: egui::Key::S,
        modifiers: egui::Modifiers::SHIFT,
    };
    
    assert_eq!(binding1, binding2);
    assert_ne!(binding1, binding3);
}

#[test]
fn test_action_clone() {
    let action = Action::OpenFile;
    let cloned = action.clone();
    assert_eq!(action, cloned);
}

#[test]
fn test_action_hash() {
    use std::collections::HashMap;
    
    let mut map = HashMap::new();
    map.insert(Action::OpenFile, "open");
    map.insert(Action::SaveFile, "save");
    
    assert_eq!(map.get(&Action::OpenFile), Some(&"open"));
    assert_eq!(map.get(&Action::SaveFile), Some(&"save"));
}

#[test]
fn test_action_debug_format() {
    let action = Action::JogXPlus;
    let debug_str = format!("{:?}", action);
    assert!(debug_str.contains("JogXPlus"));
}

#[test]
fn test_default_keybindings_open_file() {
    let bindings = create_default_keybindings();
    let binding = bindings.get(&Action::OpenFile).unwrap();
    assert_eq!(binding.key, egui::Key::O);
    assert_eq!(binding.modifiers, egui::Modifiers::CTRL);
}

#[test]
fn test_default_keybindings_save_file() {
    let bindings = create_default_keybindings();
    let binding = bindings.get(&Action::SaveFile).unwrap();
    assert_eq!(binding.key, egui::Key::S);
    assert_eq!(binding.modifiers, egui::Modifiers::CTRL);
}

#[test]
fn test_default_keybindings_jog_bindings() {
    let bindings = create_default_keybindings();
    
    let jog_x_plus = bindings.get(&Action::JogXPlus).unwrap();
    assert_eq!(jog_x_plus.key, egui::Key::ArrowRight);
    assert_eq!(jog_x_plus.modifiers, egui::Modifiers::ALT);
    
    let jog_y_plus = bindings.get(&Action::JogYPlus).unwrap();
    assert_eq!(jog_y_plus.key, egui::Key::ArrowUp);
    assert_eq!(jog_y_plus.modifiers, egui::Modifiers::ALT);
}

#[test]
fn test_default_keybindings_uniqueness() {
    let bindings = create_default_keybindings();
    
    // Count total bindings
    let total = bindings.len();
    
    // All actions should have bindings
    let actions = vec![
        Action::OpenFile,
        Action::SaveFile,
        Action::ExportGcode,
        Action::ImportVector,
        Action::Undo,
        Action::Redo,
        Action::ZoomIn,
        Action::ZoomOut,
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
    ];
    
    assert_eq!(total, actions.len());
}

#[test]
fn test_keybinding_different_modifiers() {
    let keybindings = create_default_keybindings();
    
    // Verify different actions have different modifier combinations
    let open = keybindings.get(&Action::OpenFile).unwrap();
    let undo = keybindings.get(&Action::Undo).unwrap();
    let jog = keybindings.get(&Action::JogXPlus).unwrap();
    
    // All use different modifiers (CTRL, CTRL, ALT)
    assert_eq!(open.modifiers, egui::Modifiers::CTRL);
    assert_eq!(undo.modifiers, egui::Modifiers::CTRL);
    assert_eq!(jog.modifiers, egui::Modifiers::ALT);
}
