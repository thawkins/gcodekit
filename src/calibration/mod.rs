//! Machine calibration system.
//!
//! This module provides comprehensive machine calibration capabilities including
//! step calibration, backlash compensation, homing configuration, and calibration
//! history tracking and persistence.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::time::{Duration, SystemTime};

pub mod step_calibration;
pub mod backlash_compensation;
pub mod homing_config;
pub mod calibration_procedures;

pub use step_calibration::StepCalibration;
pub use backlash_compensation::BacklashCompensation;
pub use homing_config::HomingConfiguration;
pub use calibration_procedures::{CalibrationProcedure, CalibrationStep};

/// Axis identifier for calibration operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Axis {
    X,
    Y,
    Z,
}

impl std::fmt::Display for Axis {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Axis::X => write!(f, "X"),
            Axis::Y => write!(f, "Y"),
            Axis::Z => write!(f, "Z"),
        }
    }
}

/// Result of a calibration operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationResult {
    pub axis: Axis,
    pub parameter: String,
    pub old_value: f32,
    pub new_value: f32,
    pub timestamp: SystemTime,
    pub success: bool,
    pub notes: String,
}

/// Complete machine calibration configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineCalibration {
    /// Step calibration (steps/mm) for each axis
    pub step_calibration: StepCalibration,

    /// Backlash compensation (mm) for each axis
    pub backlash_compensation: BacklashCompensation,

    /// Homing configuration
    pub homing_config: HomingConfiguration,

    /// Calibration history
    pub calibration_history: Vec<CalibrationResult>,

    /// Last calibration date
    pub last_calibrated: Option<SystemTime>,

    /// Machine name/profile identifier
    pub machine_name: String,

    /// Additional notes about this calibration
    pub notes: String,
}

impl Default for MachineCalibration {
    fn default() -> Self {
        Self {
            step_calibration: StepCalibration::default(),
            backlash_compensation: BacklashCompensation::default(),
            homing_config: HomingConfiguration::default(),
            calibration_history: Vec::new(),
            last_calibrated: None,
            machine_name: "Default Machine".to_string(),
            notes: String::new(),
        }
    }
}

impl MachineCalibration {
    /// Create a new machine calibration with a given name.
    ///
    /// # Arguments
    /// * `machine_name` - Name identifier for the machine profile
    pub fn new(machine_name: String) -> Self {
        Self {
            machine_name,
            ..Default::default()
        }
    }

    /// Record a calibration result in history.
    pub fn record_calibration(&mut self, result: CalibrationResult) {
        self.calibration_history.push(result);
        self.last_calibrated = Some(SystemTime::now());
    }

    /// Get the most recent calibration for a specific axis and parameter.
    pub fn get_latest_calibration(&self, axis: Axis, param: &str) -> Option<&CalibrationResult> {
        self.calibration_history
            .iter()
            .rev()
            .find(|r| r.axis == axis && r.parameter == param)
    }

    /// Get all calibrations for a specific axis.
    pub fn get_axis_calibrations(&self, axis: Axis) -> Vec<&CalibrationResult> {
        self.calibration_history
            .iter()
            .filter(|r| r.axis == axis)
            .collect()
    }

    /// Export calibration to JSON file.
    pub fn save_to_file(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Import calibration from JSON file.
    pub fn load_from_file(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let json = std::fs::read_to_string(path)?;
        let calibration = serde_json::from_str(&json)?;
        Ok(calibration)
    }

    /// Get GRBL commands to apply current calibration.
    pub fn get_grbl_commands(&self) -> Vec<String> {
        let mut commands = Vec::new();

        // Step calibration commands ($100-$102 for XYZ)
        commands.extend(self.step_calibration.get_grbl_commands());

        // Backlash compensation commands ($130-$132 for XYZ)
        commands.extend(self.backlash_compensation.get_grbl_commands());

        // Homing configuration commands ($22-$25)
        commands.extend(self.homing_config.get_grbl_commands());

        commands
    }

    /// Clear all calibration history while keeping current values.
    pub fn clear_history(&mut self) {
        self.calibration_history.clear();
        self.last_calibrated = None;
    }

    /// Calculate time since last calibration.
    pub fn time_since_calibration(&self) -> Option<Duration> {
        self.last_calibrated
            .and_then(|t| t.elapsed().ok())
    }

    /// Check if calibration is stale (older than specified duration).
    pub fn is_stale(&self, threshold: Duration) -> bool {
        match self.time_since_calibration() {
            Some(elapsed) => elapsed > threshold,
            None => true,
        }
    }
}

/// Collection of machine calibration profiles.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationProfiles {
    pub profiles: HashMap<String, MachineCalibration>,
    pub active_profile: String,
}

impl Default for CalibrationProfiles {
    fn default() -> Self {
        let mut profiles = HashMap::new();
        profiles.insert("Default".to_string(), MachineCalibration::default());

        Self {
            profiles,
            active_profile: "Default".to_string(),
        }
    }
}

impl CalibrationProfiles {
    /// Get the currently active calibration profile.
    pub fn get_active(&self) -> Option<&MachineCalibration> {
        self.profiles.get(&self.active_profile)
    }

    /// Get mutable reference to the active profile.
    pub fn get_active_mut(&mut self) -> Option<&mut MachineCalibration> {
        self.profiles.get_mut(&self.active_profile)
    }

    /// Switch to a different profile.
    pub fn set_active_profile(&mut self, name: String) -> Result<(), String> {
        if self.profiles.contains_key(&name) {
            self.active_profile = name;
            Ok(())
        } else {
            Err(format!("Profile '{}' not found", name))
        }
    }

    /// Create a new calibration profile.
    pub fn create_profile(&mut self, name: String) -> Result<(), String> {
        if self.profiles.contains_key(&name) {
            Err(format!("Profile '{}' already exists", name))
        } else {
            self.profiles.insert(name, MachineCalibration::default());
            Ok(())
        }
    }

    /// Delete a calibration profile.
    pub fn delete_profile(&mut self, name: &str) -> Result<(), String> {
        if name == "Default" {
            Err("Cannot delete the Default profile".to_string())
        } else if self.profiles.remove(name).is_some() {
            if self.active_profile == name {
                self.active_profile = "Default".to_string();
            }
            Ok(())
        } else {
            Err(format!("Profile '{}' not found", name))
        }
    }

    /// Save all profiles to file.
    pub fn save_to_file(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Load profiles from file.
    pub fn load_from_file(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let json = std::fs::read_to_string(path)?;
        let profiles = serde_json::from_str(&json)?;
        Ok(profiles)
    }

    /// List all available profile names.
    pub fn list_profiles(&self) -> Vec<String> {
        self.profiles.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_machine_calibration_default() {
        let cal = MachineCalibration::default();
        assert_eq!(cal.machine_name, "Default Machine");
        assert!(cal.calibration_history.is_empty());
        assert!(cal.last_calibrated.is_none());
    }

    #[test]
    fn test_machine_calibration_new() {
        let cal = MachineCalibration::new("Test Machine".to_string());
        assert_eq!(cal.machine_name, "Test Machine");
    }

    #[test]
    fn test_record_calibration() {
        let mut cal = MachineCalibration::default();
        let result = CalibrationResult {
            axis: Axis::X,
            parameter: "$100".to_string(),
            old_value: 100.0,
            new_value: 105.0,
            timestamp: SystemTime::now(),
            success: true,
            notes: "Step calibration".to_string(),
        };

        cal.record_calibration(result);
        assert_eq!(cal.calibration_history.len(), 1);
        assert!(cal.last_calibrated.is_some());
    }

    #[test]
    fn test_calibration_profiles_default() {
        let profiles = CalibrationProfiles::default();
        assert_eq!(profiles.active_profile, "Default");
        assert!(profiles.get_active().is_some());
    }

    #[test]
    fn test_create_profile() {
        let mut profiles = CalibrationProfiles::default();
        assert!(profiles.create_profile("Profile1".to_string()).is_ok());
        assert!(profiles.profiles.contains_key("Profile1"));
    }

    #[test]
    fn test_create_duplicate_profile() {
        let mut profiles = CalibrationProfiles::default();
        assert!(profiles.create_profile("Profile1".to_string()).is_ok());
        assert!(profiles.create_profile("Profile1".to_string()).is_err());
    }

    #[test]
    fn test_delete_profile() {
        let mut profiles = CalibrationProfiles::default();
        profiles.create_profile("Profile1".to_string()).unwrap();
        assert!(profiles.delete_profile("Profile1").is_ok());
        assert!(!profiles.profiles.contains_key("Profile1"));
    }

    #[test]
    fn test_cannot_delete_default() {
        let mut profiles = CalibrationProfiles::default();
        assert!(profiles.delete_profile("Default").is_err());
    }

    #[test]
    fn test_switch_profile() {
        let mut profiles = CalibrationProfiles::default();
        profiles.create_profile("Profile1".to_string()).unwrap();
        assert!(profiles.set_active_profile("Profile1".to_string()).is_ok());
        assert_eq!(profiles.active_profile, "Profile1");
    }

    #[test]
    fn test_list_profiles() {
        let mut profiles = CalibrationProfiles::default();
        profiles.create_profile("Profile1".to_string()).unwrap();
        profiles.create_profile("Profile2".to_string()).unwrap();
        let list = profiles.list_profiles();
        assert_eq!(list.len(), 3); // Default + 2 new
    }

    #[test]
    fn test_axis_display() {
        assert_eq!(format!("{}", Axis::X), "X");
        assert_eq!(format!("{}", Axis::Y), "Y");
        assert_eq!(format!("{}", Axis::Z), "Z");
    }
}
