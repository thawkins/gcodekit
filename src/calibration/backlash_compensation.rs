//! Backlash compensation for machine axes.
//!
//! Manages backlash compensation values for X, Y, and Z axes. Backlash is the
//! amount of mechanical play in the lead screws or belt drive systems and causes
//! positioning errors when direction changes.

use serde::{Deserialize, Serialize};

/// Backlash compensation values for each axis (mm).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BacklashCompensation {
    pub x_backlash: f32,
    pub y_backlash: f32,
    pub z_backlash: f32,
}

impl Default for BacklashCompensation {
    fn default() -> Self {
        // Default zero backlash (no compensation)
        Self {
            x_backlash: 0.0,
            y_backlash: 0.0,
            z_backlash: 0.0,
        }
    }
}

impl BacklashCompensation {
    /// Create new backlash compensation with specified values.
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            x_backlash: x,
            y_backlash: y,
            z_backlash: z,
        }
    }

    /// Get GRBL commands for backlash compensation.
    /// Returns commands for settings $130, $131, $132 (backlash for X, Y, Z).
    pub fn get_grbl_commands(&self) -> Vec<String> {
        vec![
            format!("$130={:.3}", self.x_backlash),
            format!("$131={:.3}", self.y_backlash),
            format!("$132={:.3}", self.z_backlash),
        ]
    }

    /// Detect backlash by measuring a small movement in both directions.
    ///
    /// # Arguments
    /// * `forward_position` - Position after moving forward (mm)
    /// * `backward_position` - Position after moving backward from forward_pos (mm)
    /// * `test_distance` - Distance commanded in each direction (mm)
    ///
    /// # Returns
    /// Detected backlash value (mm)
    pub fn detect_backlash(
        forward_position: f32,
        backward_position: f32,
        _test_distance: f32,
    ) -> f32 {
        // Backlash is the difference between forward and backward position
        // when moving the same distance in opposite directions
        (forward_position - backward_position).abs()
    }

    /// Validate backlash value is within reasonable range.
    pub fn validate_backlash(backlash: f32) -> bool {
        (0.0..10.0).contains(&backlash) // Reasonable range: 0 to 10mm
    }

    /// Update X axis backlash compensation.
    pub fn set_x_backlash(&mut self, backlash: f32) -> Result<(), String> {
        if Self::validate_backlash(backlash) {
            self.x_backlash = backlash;
            Ok(())
        } else {
            Err(format!(
                "Invalid backlash value {}: must be between 0 and 10mm",
                backlash
            ))
        }
    }

    /// Update Y axis backlash compensation.
    pub fn set_y_backlash(&mut self, backlash: f32) -> Result<(), String> {
        if Self::validate_backlash(backlash) {
            self.y_backlash = backlash;
            Ok(())
        } else {
            Err(format!(
                "Invalid backlash value {}: must be between 0 and 10mm",
                backlash
            ))
        }
    }

    /// Update Z axis backlash compensation.
    pub fn set_z_backlash(&mut self, backlash: f32) -> Result<(), String> {
        if Self::validate_backlash(backlash) {
            self.z_backlash = backlash;
            Ok(())
        } else {
            Err(format!(
                "Invalid backlash value {}: must be between 0 and 10mm",
                backlash
            ))
        }
    }

    /// Get total backlash across all axes.
    pub fn total_backlash(&self) -> f32 {
        self.x_backlash + self.y_backlash + self.z_backlash
    }

    /// Check if any axis has significant backlash.
    pub fn has_significant_backlash(&self, threshold: f32) -> bool {
        self.x_backlash > threshold || self.y_backlash > threshold || self.z_backlash > threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_values() {
        let comp = BacklashCompensation::default();
        assert_eq!(comp.x_backlash, 0.0);
        assert_eq!(comp.y_backlash, 0.0);
        assert_eq!(comp.z_backlash, 0.0);
    }

    #[test]
    fn test_new() {
        let comp = BacklashCompensation::new(0.1, 0.15, 0.05);
        assert_eq!(comp.x_backlash, 0.1);
        assert_eq!(comp.y_backlash, 0.15);
        assert_eq!(comp.z_backlash, 0.05);
    }

    #[test]
    fn test_get_grbl_commands() {
        let comp = BacklashCompensation::new(0.1, 0.15, 0.05);
        let commands = comp.get_grbl_commands();
        assert_eq!(commands.len(), 3);
        assert_eq!(commands[0], "$130=0.100");
        assert_eq!(commands[1], "$131=0.150");
        assert_eq!(commands[2], "$132=0.050");
    }

    #[test]
    fn test_detect_backlash() {
        // Forward to 10mm, backward from 10mm should return to ~9.9mm with 0.1mm backlash
        let backlash = BacklashCompensation::detect_backlash(10.0, 9.9, 10.0);
        assert!((backlash - 0.1).abs() < 0.01);
    }

    #[test]
    fn test_detect_backlash_no_backlash() {
        let backlash = BacklashCompensation::detect_backlash(10.0, 10.0, 10.0);
        assert_eq!(backlash, 0.0);
    }

    #[test]
    fn test_validate_backlash_valid() {
        assert!(BacklashCompensation::validate_backlash(0.0));
        assert!(BacklashCompensation::validate_backlash(0.5));
        assert!(BacklashCompensation::validate_backlash(9.9));
    }

    #[test]
    fn test_validate_backlash_invalid() {
        assert!(!BacklashCompensation::validate_backlash(-0.1));
        assert!(!BacklashCompensation::validate_backlash(10.1));
    }

    #[test]
    fn test_set_x_backlash_valid() {
        let mut comp = BacklashCompensation::default();
        assert!(comp.set_x_backlash(0.2).is_ok());
        assert_eq!(comp.x_backlash, 0.2);
    }

    #[test]
    fn test_set_x_backlash_invalid() {
        let mut comp = BacklashCompensation::default();
        assert!(comp.set_x_backlash(-0.1).is_err());
        assert!(comp.set_x_backlash(10.5).is_err());
    }

    #[test]
    fn test_set_y_backlash_valid() {
        let mut comp = BacklashCompensation::default();
        assert!(comp.set_y_backlash(0.25).is_ok());
        assert_eq!(comp.y_backlash, 0.25);
    }

    #[test]
    fn test_set_z_backlash_valid() {
        let mut comp = BacklashCompensation::default();
        assert!(comp.set_z_backlash(0.15).is_ok());
        assert_eq!(comp.z_backlash, 0.15);
    }

    #[test]
    fn test_total_backlash() {
        let comp = BacklashCompensation::new(0.1, 0.2, 0.3);
        assert!((comp.total_backlash() - 0.6).abs() < 0.01);
    }

    #[test]
    fn test_has_significant_backlash() {
        let comp = BacklashCompensation::new(0.5, 0.1, 0.1);
        assert!(comp.has_significant_backlash(0.3));
        assert!(!comp.has_significant_backlash(0.6));
    }
}
