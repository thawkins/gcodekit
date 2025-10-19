//! Step calibration for machine axes.
//!
//! Manages step/mm calibration for X, Y, and Z axes. This calibration determines
//! how many stepper motor steps are required to move each axis by 1mm.

use serde::{Deserialize, Serialize};

/// Step calibration values for each axis (steps/mm).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct StepCalibration {
    pub x_steps: f32,
    pub y_steps: f32,
    pub z_steps: f32,
}

impl Default for StepCalibration {
    fn default() -> Self {
        // Default GRBL step values (100 steps/mm is common)
        Self {
            x_steps: 100.0,
            y_steps: 100.0,
            z_steps: 100.0,
        }
    }
}

impl StepCalibration {
    /// Create new step calibration with specified values.
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            x_steps: x,
            y_steps: y,
            z_steps: z,
        }
    }

    /// Get GRBL commands for step calibration.
    /// Returns commands for settings $100, $101, $102 (steps/mm for X, Y, Z).
    pub fn get_grbl_commands(&self) -> Vec<String> {
        vec![
            format!("$100={:.3}", self.x_steps),
            format!("$101={:.3}", self.y_steps),
            format!("$102={:.3}", self.z_steps),
        ]
    }

    /// Calculate correct step value based on actual vs expected distance.
    ///
    /// # Arguments
    /// * `current_steps` - Current steps/mm setting
    /// * `commanded_distance` - Distance that was commanded (mm)
    /// * `actual_distance` - Actual distance the machine moved (mm)
    ///
    /// # Returns
    /// Corrected steps/mm value
    pub fn calculate_correction(
        current_steps: f32,
        commanded_distance: f32,
        actual_distance: f32,
    ) -> f32 {
        if actual_distance == 0.0 || commanded_distance == 0.0 {
            current_steps
        } else {
            current_steps * commanded_distance / actual_distance
        }
    }

    /// Validate step value is within reasonable range.
    pub fn validate_step_value(steps: f32) -> bool {
        steps > 0.0 && steps < 1000.0 // Reasonable range: 0.1 to 1000 steps/mm
    }

    /// Update X axis step calibration.
    pub fn set_x_steps(&mut self, steps: f32) -> Result<(), String> {
        if Self::validate_step_value(steps) {
            self.x_steps = steps;
            Ok(())
        } else {
            Err(format!(
                "Invalid step value {}: must be between 0.1 and 1000",
                steps
            ))
        }
    }

    /// Update Y axis step calibration.
    pub fn set_y_steps(&mut self, steps: f32) -> Result<(), String> {
        if Self::validate_step_value(steps) {
            self.y_steps = steps;
            Ok(())
        } else {
            Err(format!(
                "Invalid step value {}: must be between 0.1 and 1000",
                steps
            ))
        }
    }

    /// Update Z axis step calibration.
    pub fn set_z_steps(&mut self, steps: f32) -> Result<(), String> {
        if Self::validate_step_value(steps) {
            self.z_steps = steps;
            Ok(())
        } else {
            Err(format!(
                "Invalid step value {}: must be between 0.1 and 1000",
                steps
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_values() {
        let cal = StepCalibration::default();
        assert_eq!(cal.x_steps, 100.0);
        assert_eq!(cal.y_steps, 100.0);
        assert_eq!(cal.z_steps, 100.0);
    }

    #[test]
    fn test_new() {
        let cal = StepCalibration::new(80.0, 90.0, 110.0);
        assert_eq!(cal.x_steps, 80.0);
        assert_eq!(cal.y_steps, 90.0);
        assert_eq!(cal.z_steps, 110.0);
    }

    #[test]
    fn test_get_grbl_commands() {
        let cal = StepCalibration::new(80.5, 90.5, 110.5);
        let commands = cal.get_grbl_commands();
        assert_eq!(commands.len(), 3);
        assert_eq!(commands[0], "$100=80.500");
        assert_eq!(commands[1], "$101=90.500");
        assert_eq!(commands[2], "$102=110.500");
    }

    #[test]
    fn test_calculate_correction() {
        // If machine moved 9.5mm when told to move 10mm,
        // correction = 100 * 10 / 9.5 = 105.26
        let corrected = StepCalibration::calculate_correction(100.0, 10.0, 9.5);
        assert!((corrected - 105.26).abs() < 0.01);
    }

    #[test]
    fn test_calculate_correction_perfect() {
        let corrected = StepCalibration::calculate_correction(100.0, 10.0, 10.0);
        assert_eq!(corrected, 100.0);
    }

    #[test]
    fn test_validate_step_value_valid() {
        assert!(StepCalibration::validate_step_value(100.0));
        assert!(StepCalibration::validate_step_value(0.5));
        assert!(StepCalibration::validate_step_value(999.9));
    }

    #[test]
    fn test_validate_step_value_invalid() {
        assert!(!StepCalibration::validate_step_value(0.0));
        assert!(!StepCalibration::validate_step_value(-10.0));
        assert!(!StepCalibration::validate_step_value(1000.5));
    }

    #[test]
    fn test_set_x_steps_valid() {
        let mut cal = StepCalibration::default();
        assert!(cal.set_x_steps(95.5).is_ok());
        assert_eq!(cal.x_steps, 95.5);
    }

    #[test]
    fn test_set_x_steps_invalid() {
        let mut cal = StepCalibration::default();
        assert!(cal.set_x_steps(0.0).is_err());
        assert!(cal.set_x_steps(1500.0).is_err());
    }

    #[test]
    fn test_set_y_steps_valid() {
        let mut cal = StepCalibration::default();
        assert!(cal.set_y_steps(105.0).is_ok());
        assert_eq!(cal.y_steps, 105.0);
    }

    #[test]
    fn test_set_z_steps_valid() {
        let mut cal = StepCalibration::default();
        assert!(cal.set_z_steps(110.0).is_ok());
        assert_eq!(cal.z_steps, 110.0);
    }
}
