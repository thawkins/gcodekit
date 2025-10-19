//! Settings Management UI Widget
//!
//! Provides the user interface for managing machine profiles and settings.

use crate::settings::{MachineProfile, ProfileManager, SettingsStorage};
use egui::{Button, ComboBox, TextEdit, Ui};

/// State for settings UI
#[derive(Debug, Clone)]
pub struct SettingsUiState {
    /// Current profile manager
    pub profile_manager: ProfileManager,
    /// Whether the profile creation dialog is open
    pub show_profile_dialog: bool,
    /// New profile name input
    pub new_profile_name: String,
    /// New profile machine type input
    pub new_profile_machine_type: String,
    /// New profile port input
    pub new_profile_port: String,
    /// Whether the profile deletion confirmation dialog is open
    pub show_delete_confirmation: bool,
    /// Profile name to delete
    pub profile_to_delete: String,
    /// Error message from operations
    pub last_error: Option<String>,
    /// Success message from operations
    pub last_success: Option<String>,
}

impl Default for SettingsUiState {
    fn default() -> Self {
        Self::new()
    }
}

impl SettingsUiState {
    /// Create a new settings UI state
    pub fn new() -> Self {
        let mut manager = ProfileManager::new();

        // Try to load existing profiles
        if let Ok(profile_names) = SettingsStorage::list_profiles() {
            for name in profile_names {
                if let Ok(profile) = SettingsStorage::load_profile(&name) {
                    manager.add_profile(profile);
                }
            }
        }

        Self {
            profile_manager: manager,
            show_profile_dialog: false,
            new_profile_name: String::new(),
            new_profile_machine_type: String::new(),
            new_profile_port: String::new(),
            show_delete_confirmation: false,
            profile_to_delete: String::new(),
            last_error: None,
            last_success: None,
        }
    }

    /// Create a new profile
    pub fn create_profile(&mut self) {
        if self.new_profile_name.is_empty() {
            self.last_error = Some("Profile name cannot be empty".to_string());
            return;
        }

        let mut profile = MachineProfile::new(
            self.new_profile_name.clone(),
            self.new_profile_machine_type.clone(),
        );
        profile.port = self.new_profile_port.clone();

        // Save to disk
        if let Err(e) = SettingsStorage::save_profile(&profile) {
            self.last_error = Some(format!("Failed to save profile: {}", e));
            return;
        }

        // Add to manager
        self.profile_manager.add_profile(profile.clone());

        // Reset UI
        self.new_profile_name.clear();
        self.new_profile_machine_type.clear();
        self.new_profile_port.clear();
        self.show_profile_dialog = false;
        self.last_success = Some(format!("Profile '{}' created successfully", profile.name));
    }

    /// Delete a profile
    pub fn delete_profile(&mut self, name: &str) {
        if let Err(e) = SettingsStorage::delete_profile(name) {
            self.last_error = Some(format!("Failed to delete profile: {}", e));
            return;
        }

        self.profile_manager.remove_profile(name);
        self.show_delete_confirmation = false;
        self.profile_to_delete.clear();
        self.last_success = Some(format!("Profile '{}' deleted successfully", name));
    }

    /// Set the active profile
    pub fn set_active_profile(&mut self, name: String) {
        match self.profile_manager.set_active_profile(name.clone()) {
            Ok(_) => {
                self.last_success = Some(format!("Profile '{}' activated", name));
            }
            Err(e) => {
                self.last_error = Some(format!("Failed to activate profile: {}", e));
            }
        }
    }

    /// Clear error message
    pub fn clear_error(&mut self) {
        self.last_error = None;
    }

    /// Clear success message
    pub fn clear_success(&mut self) {
        self.last_success = None;
    }
}

/// Draw the settings panel UI
pub fn draw_settings_panel(ui: &mut Ui, state: &mut SettingsUiState) {
    ui.heading("⚙️ Settings Management");

    // Show error/success messages
    if let Some(error) = state.last_error.clone() {
        ui.colored_label(egui::Color32::RED, format!("❌ {}", error));
        if ui.button("Dismiss").clicked() {
            state.clear_error();
        }
        ui.separator();
    }

    if let Some(success) = state.last_success.clone() {
        ui.colored_label(egui::Color32::GREEN, format!("✓ {}", success));
        if ui.button("Dismiss").clicked() {
            state.clear_success();
        }
        ui.separator();
    }

    // Profile Management Section
    ui.group(|ui| {
        ui.label("📋 Machine Profiles");

        // Clone the data we need to avoid borrowing issues
        let profiles_data: Vec<(String, String, String)> = state
            .profile_manager
            .list_profiles()
            .iter()
            .map(|p| (p.name.clone(), p.machine_type.clone(), p.port.clone()))
            .collect();
        let active_name = state.profile_manager.active_profile_name().map(|s| s.to_string());

        if profiles_data.is_empty() {
            ui.label("No profiles created yet");
        } else {
            let mut action: Option<(String, &str)> = None;

            for (name, machine_type, port) in &profiles_data {
                let is_active = active_name.as_ref() == Some(name);

                ui.horizontal(|ui| {
                    if is_active {
                        ui.label("✓");
                    } else {
                        ui.label(" ");
                    }

                    ui.vertical(|ui| {
                        ui.label(name);
                        ui.label(
                            egui::RichText::new(format!("{} • Port: {}", machine_type, port))
                                .small()
                                .weak(),
                        );
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("🗑").on_hover_text("Delete").clicked() {
                            action = Some((name.clone(), "delete"));
                        }

                        if !is_active && ui.button("✓").on_hover_text("Activate").clicked() {
                            action = Some((name.clone(), "activate"));
                        }
                    });
                });
                ui.separator();
            }

            // Apply action after loop to avoid borrowing issues
            if let Some((profile_name, action_type)) = action {
                match action_type {
                    "delete" => {
                        state.profile_to_delete = profile_name;
                        state.show_delete_confirmation = true;
                    }
                    "activate" => {
                        state.set_active_profile(profile_name);
                    }
                    _ => {}
                }
            }
        }

        if ui.button("➕ New Profile").clicked() {
            state.show_profile_dialog = true;
        }
    });
}

/// Draw the settings dialogs
pub fn draw_settings_dialogs(ctx: &egui::Context, state: &mut SettingsUiState) {
    // New Profile Dialog
    if state.show_profile_dialog {
        let mut open = true;
        egui::Window::new("Create New Profile")
            .open(&mut open)
            .resizable(true)
            .show(ctx, |ui| {
                ui.label("Profile Name:");
                TextEdit::singleline(&mut state.new_profile_name).show(ui);

                ui.label("Machine Type:");
                ComboBox::from_label("")
                    .selected_text(&state.new_profile_machine_type)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut state.new_profile_machine_type,
                            "CNC Mill".to_string(),
                            "CNC Mill",
                        );
                        ui.selectable_value(
                            &mut state.new_profile_machine_type,
                            "Laser Engraver".to_string(),
                            "Laser Engraver",
                        );
                        ui.selectable_value(
                            &mut state.new_profile_machine_type,
                            "3D Printer".to_string(),
                            "3D Printer",
                        );
                        ui.selectable_value(
                            &mut state.new_profile_machine_type,
                            "Plasma Cutter".to_string(),
                            "Plasma Cutter",
                        );
                    });

                ui.label("Serial Port:");
                TextEdit::singleline(&mut state.new_profile_port).show(ui);

                ui.horizontal(|ui| {
                    if ui.button("Create").on_hover_text("Create the new profile").clicked() {
                        state.create_profile();
                    }
                    if ui.button("Cancel").on_hover_text("Cancel profile creation").clicked() {
                        state.show_profile_dialog = false;
                        state.new_profile_name.clear();
                        state.new_profile_machine_type.clear();
                        state.new_profile_port.clear();
                    }
                });
            });
        state.show_profile_dialog = open;
    }

    // Delete Confirmation Dialog
    if state.show_delete_confirmation {
        let mut open = true;
        egui::Window::new("Delete Profile Confirmation")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label(format!(
                    "Are you sure you want to delete profile '{}'?",
                    state.profile_to_delete
                ));
                ui.label("This action cannot be undone.");

                ui.horizontal(|ui| {
                    let profile_to_delete = state.profile_to_delete.clone();
                    if ui.button("Delete").on_hover_text("Permanently delete the profile").clicked() {
                        state.delete_profile(&profile_to_delete);
                    }
                    if ui.button("Cancel").on_hover_text("Cancel deletion").clicked() {
                        state.show_delete_confirmation = false;
                        state.profile_to_delete.clear();
                    }
                });
            });
        state.show_delete_confirmation = open;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_ui_state_creation() {
        let state = SettingsUiState::new();
        assert!(!state.show_profile_dialog);
        assert!(!state.show_delete_confirmation);
    }

    #[test]
    fn test_settings_ui_create_profile() {
        let mut state = SettingsUiState::new();
        let initial_count = state.profile_manager.profile_count();
        state.new_profile_name = "UniqueTestProfile12345".to_string();
        state.new_profile_machine_type = "CNC Mill".to_string();
        state.new_profile_port = "/dev/ttyUSB0".to_string();

        state.create_profile();

        assert!(state.new_profile_name.is_empty());
        assert!(state.last_success.is_some());
        assert_eq!(state.profile_manager.profile_count(), initial_count + 1);

        // Cleanup
        let _ = SettingsStorage::delete_profile("UniqueTestProfile12345");
    }

    #[test]
    fn test_settings_ui_error_on_empty_name() {
        let mut state = SettingsUiState::new();
        state.new_profile_name.clear();
        state.create_profile();
        assert!(state.last_error.is_some());
    }
}
