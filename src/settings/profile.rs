//! Machine Profile Management
//!
//! Manages GRBL machine profiles with settings, presets, and profile switching.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// GRBL machine profile settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileSettings {
    /// Step/mm for X axis
    pub x_step_mm: f32,
    /// Step/mm for Y axis
    pub y_step_mm: f32,
    /// Step/mm for Z axis
    pub z_step_mm: f32,
    /// Max rate for X axis (mm/min)
    pub x_max_rate: f32,
    /// Max rate for Y axis (mm/min)
    pub y_max_rate: f32,
    /// Max rate for Z axis (mm/min)
    pub z_max_rate: f32,
    /// Acceleration for X axis (mm/sec²)
    pub x_acceleration: f32,
    /// Acceleration for Y axis (mm/sec²)
    pub y_acceleration: f32,
    /// Acceleration for Z axis (mm/sec²)
    pub z_acceleration: f32,
    /// Max spindle speed (RPM)
    pub max_spindle_speed: u32,
    /// Min spindle speed (RPM)
    pub min_spindle_speed: u32,
    /// Enable soft limits
    pub soft_limits_enabled: bool,
    /// Travel limit for X axis (mm)
    pub x_travel_limit: f32,
    /// Travel limit for Y axis (mm)
    pub y_travel_limit: f32,
    /// Travel limit for Z axis (mm)
    pub z_travel_limit: f32,
    /// Invert X axis
    pub x_axis_inverted: bool,
    /// Invert Y axis
    pub y_axis_inverted: bool,
    /// Invert Z axis
    pub z_axis_inverted: bool,
}

impl Default for ProfileSettings {
    fn default() -> Self {
        Self {
            x_step_mm: 250.0,
            y_step_mm: 250.0,
            z_step_mm: 250.0,
            x_max_rate: 500.0,
            y_max_rate: 500.0,
            z_max_rate: 300.0,
            x_acceleration: 10.0,
            y_acceleration: 10.0,
            z_acceleration: 5.0,
            max_spindle_speed: 10000,
            min_spindle_speed: 100,
            soft_limits_enabled: true,
            x_travel_limit: 200.0,
            y_travel_limit: 200.0,
            z_travel_limit: 100.0,
            x_axis_inverted: false,
            y_axis_inverted: false,
            z_axis_inverted: false,
        }
    }
}

/// A complete machine profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineProfile {
    /// Unique profile name
    pub name: String,
    /// Optional description
    pub description: String,
    /// Machine type (e.g., "CNC Mill", "Laser Engraver")
    pub machine_type: String,
    /// Port/connection info
    pub port: String,
    /// GRBL settings
    pub settings: ProfileSettings,
    /// Custom tags for organization
    pub tags: Vec<String>,
    /// Creation timestamp (ISO 8601)
    pub created_at: String,
    /// Last modified timestamp (ISO 8601)
    pub modified_at: String,
}

impl MachineProfile {
    /// Create a new machine profile
    pub fn new(name: String, machine_type: String) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            name,
            description: String::new(),
            machine_type,
            port: String::new(),
            settings: ProfileSettings::default(),
            tags: Vec::new(),
            created_at: now.clone(),
            modified_at: now,
        }
    }

    /// Update the modified timestamp
    pub fn update_timestamp(&mut self) {
        self.modified_at = chrono::Utc::now().to_rfc3339();
    }
}

/// Manages machine profiles
#[derive(Debug, Clone)]
pub struct ProfileManager {
    profiles: HashMap<String, MachineProfile>,
    active_profile: Option<String>,
}

impl ProfileManager {
    /// Create a new profile manager
    pub fn new() -> Self {
        Self {
            profiles: HashMap::new(),
            active_profile: None,
        }
    }

    /// Add a profile to the manager
    pub fn add_profile(&mut self, profile: MachineProfile) {
        self.profiles.insert(profile.name.clone(), profile);
    }

    /// Remove a profile by name
    pub fn remove_profile(&mut self, name: &str) -> Option<MachineProfile> {
        let removed = self.profiles.remove(name);
        if self.active_profile.as_deref() == Some(name) {
            self.active_profile = None;
        }
        removed
    }

    /// Get a profile by name
    pub fn get_profile(&self, name: &str) -> Option<&MachineProfile> {
        self.profiles.get(name)
    }

    /// Get mutable reference to a profile
    pub fn get_profile_mut(&mut self, name: &str) -> Option<&mut MachineProfile> {
        self.profiles.get_mut(name)
    }

    /// Get all profiles
    pub fn list_profiles(&self) -> Vec<&MachineProfile> {
        self.profiles.values().collect()
    }

    /// Set the active profile
    pub fn set_active_profile(&mut self, name: String) -> anyhow::Result<()> {
        if self.profiles.contains_key(&name) {
            self.active_profile = Some(name);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Profile not found: {}", name))
        }
    }

    /// Get the active profile
    pub fn get_active_profile(&self) -> Option<&MachineProfile> {
        self.active_profile
            .as_ref()
            .and_then(|name| self.profiles.get(name))
    }

    /// Get active profile name
    pub fn active_profile_name(&self) -> Option<&str> {
        self.active_profile.as_deref()
    }

    /// Get profile count
    pub fn profile_count(&self) -> usize {
        self.profiles.len()
    }

    /// Rename a profile
    pub fn rename_profile(&mut self, old_name: &str, new_name: String) -> anyhow::Result<()> {
        if let Some(mut profile) = self.profiles.remove(old_name) {
            if self.active_profile.as_deref() == Some(old_name) {
                self.active_profile = Some(new_name.clone());
            }
            profile.name = new_name.clone();
            profile.update_timestamp();
            self.profiles.insert(new_name, profile);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Profile not found: {}", old_name))
        }
    }
}

impl Default for ProfileManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_creation() {
        let profile = MachineProfile::new("Test CNC".to_string(), "CNC Mill".to_string());
        assert_eq!(profile.name, "Test CNC");
        assert_eq!(profile.machine_type, "CNC Mill");
    }

    #[test]
    fn test_profile_timestamp_update() {
        let mut profile = MachineProfile::new("Test".to_string(), "CNC".to_string());
        let original_timestamp = profile.modified_at.clone();
        std::thread::sleep(std::time::Duration::from_millis(10));
        profile.update_timestamp();
        assert_ne!(original_timestamp, profile.modified_at);
    }

    #[test]
    fn test_profile_manager_add_profile() {
        let mut manager = ProfileManager::new();
        let profile = MachineProfile::new("Test".to_string(), "CNC".to_string());
        manager.add_profile(profile);
        assert_eq!(manager.profile_count(), 1);
    }

    #[test]
    fn test_profile_manager_set_active() {
        let mut manager = ProfileManager::new();
        let profile = MachineProfile::new("Test".to_string(), "CNC".to_string());
        manager.add_profile(profile);
        assert!(manager.set_active_profile("Test".to_string()).is_ok());
        assert_eq!(manager.active_profile_name(), Some("Test"));
    }

    #[test]
    fn test_profile_manager_remove() {
        let mut manager = ProfileManager::new();
        let profile = MachineProfile::new("Test".to_string(), "CNC".to_string());
        manager.add_profile(profile);
        assert!(manager.remove_profile("Test").is_some());
        assert_eq!(manager.profile_count(), 0);
    }

    #[test]
    fn test_profile_manager_rename() {
        let mut manager = ProfileManager::new();
        let profile = MachineProfile::new("Test".to_string(), "CNC".to_string());
        manager.add_profile(profile);
        manager.set_active_profile("Test".to_string()).unwrap();
        assert!(manager.rename_profile("Test", "NewTest".to_string()).is_ok());
        assert_eq!(manager.active_profile_name(), Some("NewTest"));
    }

    #[test]
    fn test_profile_settings_default() {
        let settings = ProfileSettings::default();
        assert_eq!(settings.max_spindle_speed, 10000);
        assert!(settings.soft_limits_enabled);
    }
}
