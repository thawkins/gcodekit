//! Homing configuration for machine axes.
//!
//! Manages homing sequence configuration including homing cycle enable, direction,
//! feed rates, and seek rates. GRBL uses limit switches to establish a known
//! machine zero position.

use serde::{Deserialize, Serialize};

/// Homing configuration settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HomingConfiguration {
    /// Enable homing cycle (GRBL $22)
    pub homing_enable: bool,

    /// Homing direction invert mask (GRBL $23) - bitmask for axis directions
    pub homing_dir_invert: u8,

    /// Homing feed rate (mm/min) - slow approach after switch contact (GRBL $24)
    pub homing_feed_rate: f32,

    /// Homing seek rate (mm/min) - fast approach to switch (GRBL $25)
    pub homing_seek_rate: f32,

    /// Homing axis order: which axes home first, second, third
    pub homing_sequence: [bool; 3], // [X, Y, Z]
}

impl Default for HomingConfiguration {
    fn default() -> Self {
        Self {
            homing_enable: true,
            homing_dir_invert: 0, // No inversion
            homing_feed_rate: 25.0, // 25 mm/min
            homing_seek_rate: 500.0, // 500 mm/min
            homing_sequence: [true, true, true], // Home all axes
        }
    }
}

impl HomingConfiguration {
    /// Get GRBL commands for homing configuration.
    /// Returns commands for settings $22-$25.
    pub fn get_grbl_commands(&self) -> Vec<String> {
        vec![
            format!("$22={}", if self.homing_enable { 1 } else { 0 }),
            format!("$23={}", self.homing_dir_invert),
            format!("$24={:.1}", self.homing_feed_rate),
            format!("$25={:.1}", self.homing_seek_rate),
        ]
    }

    /// Validate feed rate value.
    pub fn validate_feed_rate(rate: f32) -> bool {
        rate > 0.0 && rate <= 5000.0 // Reasonable range: 0.1 to 5000 mm/min
    }

    /// Set homing enable/disable.
    pub fn set_homing_enable(&mut self, enable: bool) {
        self.homing_enable = enable;
    }

    /// Set homing direction invert mask.
    /// Bits 0-2 control X, Y, Z axis directions respectively.
    pub fn set_homing_dir_invert(&mut self, mask: u8) {
        self.homing_dir_invert = mask & 0x07; // Only use bits 0-2
    }

    /// Set X axis homing direction invert.
    pub fn set_x_dir_invert(&mut self, invert: bool) {
        if invert {
            self.homing_dir_invert |= 0x01; // Set bit 0
        } else {
            self.homing_dir_invert &= !0x01; // Clear bit 0
        }
    }

    /// Set Y axis homing direction invert.
    pub fn set_y_dir_invert(&mut self, invert: bool) {
        if invert {
            self.homing_dir_invert |= 0x02; // Set bit 1
        } else {
            self.homing_dir_invert &= !0x02; // Clear bit 1
        }
    }

    /// Set Z axis homing direction invert.
    pub fn set_z_dir_invert(&mut self, invert: bool) {
        if invert {
            self.homing_dir_invert |= 0x04; // Set bit 2
        } else {
            self.homing_dir_invert &= !0x04; // Clear bit 2
        }
    }

    /// Get X axis homing direction invert status.
    pub fn get_x_dir_invert(&self) -> bool {
        (self.homing_dir_invert & 0x01) != 0
    }

    /// Get Y axis homing direction invert status.
    pub fn get_y_dir_invert(&self) -> bool {
        (self.homing_dir_invert & 0x02) != 0
    }

    /// Get Z axis homing direction invert status.
    pub fn get_z_dir_invert(&self) -> bool {
        (self.homing_dir_invert & 0x04) != 0
    }

    /// Set homing feed rate.
    pub fn set_homing_feed_rate(&mut self, rate: f32) -> Result<(), String> {
        if Self::validate_feed_rate(rate) {
            self.homing_feed_rate = rate;
            Ok(())
        } else {
            Err(format!(
                "Invalid feed rate {}: must be between 0.1 and 5000 mm/min",
                rate
            ))
        }
    }

    /// Set homing seek rate.
    pub fn set_homing_seek_rate(&mut self, rate: f32) -> Result<(), String> {
        if Self::validate_feed_rate(rate) {
            self.homing_seek_rate = rate;
            Ok(())
        } else {
            Err(format!(
                "Invalid seek rate {}: must be between 0.1 and 5000 mm/min",
                rate
            ))
        }
    }

    /// Enable homing for specific axis.
    pub fn enable_axis_homing(&mut self, axis_index: usize, enable: bool) {
        if axis_index < 3 {
            self.homing_sequence[axis_index] = enable;
        }
    }

    /// Check if axis homing is enabled.
    pub fn is_axis_homing_enabled(&self, axis_index: usize) -> bool {
        if axis_index < 3 {
            self.homing_sequence[axis_index]
        } else {
            false
        }
    }

    /// Get homing sequence as string (e.g., "XYZ", "XY", "Z").
    pub fn get_sequence_string(&self) -> String {
        let mut seq = String::new();
        if self.homing_sequence[0] {
            seq.push('X');
        }
        if self.homing_sequence[1] {
            seq.push('Y');
        }
        if self.homing_sequence[2] {
            seq.push('Z');
        }
        if seq.is_empty() {
            "None".to_string()
        } else {
            seq
        }
    }

    /// Generate homing command.
    pub fn get_home_command(&self) -> String {
        if self.homing_enable {
            "$H".to_string() // GRBL home all axes command
        } else {
            String::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_values() {
        let hom = HomingConfiguration::default();
        assert!(hom.homing_enable);
        assert_eq!(hom.homing_dir_invert, 0);
        assert_eq!(hom.homing_feed_rate, 25.0);
        assert_eq!(hom.homing_seek_rate, 500.0);
    }

    #[test]
    fn test_get_grbl_commands() {
        let hom = HomingConfiguration::default();
        let commands = hom.get_grbl_commands();
        assert_eq!(commands.len(), 4);
        assert_eq!(commands[0], "$22=1");
        assert_eq!(commands[1], "$23=0");
        assert_eq!(commands[2], "$24=25.0");
        assert_eq!(commands[3], "$25=500.0");
    }

    #[test]
    fn test_validate_feed_rate_valid() {
        assert!(HomingConfiguration::validate_feed_rate(100.0));
        assert!(HomingConfiguration::validate_feed_rate(0.1));
        assert!(HomingConfiguration::validate_feed_rate(5000.0));
    }

    #[test]
    fn test_validate_feed_rate_invalid() {
        assert!(!HomingConfiguration::validate_feed_rate(0.0));
        assert!(!HomingConfiguration::validate_feed_rate(-100.0));
        assert!(!HomingConfiguration::validate_feed_rate(5001.0));
    }

    #[test]
    fn test_set_x_dir_invert() {
        let mut hom = HomingConfiguration::default();
        hom.set_x_dir_invert(true);
        assert_eq!(hom.homing_dir_invert, 0x01);
        assert!(hom.get_x_dir_invert());
    }

    #[test]
    fn test_set_y_dir_invert() {
        let mut hom = HomingConfiguration::default();
        hom.set_y_dir_invert(true);
        assert_eq!(hom.homing_dir_invert, 0x02);
        assert!(hom.get_y_dir_invert());
    }

    #[test]
    fn test_set_z_dir_invert() {
        let mut hom = HomingConfiguration::default();
        hom.set_z_dir_invert(true);
        assert_eq!(hom.homing_dir_invert, 0x04);
        assert!(hom.get_z_dir_invert());
    }

    #[test]
    fn test_multiple_dir_inverts() {
        let mut hom = HomingConfiguration::default();
        hom.set_x_dir_invert(true);
        hom.set_y_dir_invert(true);
        assert_eq!(hom.homing_dir_invert, 0x03);
        assert!(hom.get_x_dir_invert());
        assert!(hom.get_y_dir_invert());
        assert!(!hom.get_z_dir_invert());
    }

    #[test]
    fn test_set_homing_feed_rate_valid() {
        let mut hom = HomingConfiguration::default();
        assert!(hom.set_homing_feed_rate(50.0).is_ok());
        assert_eq!(hom.homing_feed_rate, 50.0);
    }

    #[test]
    fn test_set_homing_feed_rate_invalid() {
        let mut hom = HomingConfiguration::default();
        assert!(hom.set_homing_feed_rate(0.0).is_err());
        assert!(hom.set_homing_feed_rate(6000.0).is_err());
    }

    #[test]
    fn test_enable_axis_homing() {
        let mut hom = HomingConfiguration::default();
        hom.enable_axis_homing(0, false);
        assert!(!hom.is_axis_homing_enabled(0));
    }

    #[test]
    fn test_sequence_string() {
        let mut hom = HomingConfiguration::default();
        assert_eq!(hom.get_sequence_string(), "XYZ");

        hom.enable_axis_homing(1, false);
        assert_eq!(hom.get_sequence_string(), "XZ");

        hom.enable_axis_homing(0, false);
        hom.enable_axis_homing(2, false);
        assert_eq!(hom.get_sequence_string(), "None");
    }

    #[test]
    fn test_get_home_command() {
        let mut hom = HomingConfiguration::default();
        assert_eq!(hom.get_home_command(), "$H");

        hom.set_homing_enable(false);
        assert_eq!(hom.get_home_command(), "");
    }
}
