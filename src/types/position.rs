//! Machine position for X, Y, Z axes
//!
//! Represents the current position of the CNC machine on the three primary axes.
#[derive(Clone, Debug, Default)]
pub struct MachinePosition {
    /// X-axis position in mm
    pub x: f32,
    /// Y-axis position in mm
    pub y: f32,
    /// Z-axis position in mm
    pub z: f32,
}

impl MachinePosition {
    /// Create a new machine position
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// Get axis value by character ('X', 'Y', or 'Z')
    pub fn get_axis(&self, axis: char) -> Option<f32> {
        match axis {
            'X' | 'x' => Some(self.x),
            'Y' | 'y' => Some(self.y),
            'Z' | 'z' => Some(self.z),
            _ => None,
        }
    }

    /// Set axis value by character ('X', 'Y', or 'Z')
    pub fn set_axis(&mut self, axis: char, value: f32) {
        match axis {
            'X' | 'x' => self.x = value,
            'Y' | 'y' => self.y = value,
            'Z' | 'z' => self.z = value,
            _ => {}
        }
    }

    /// Calculate distance to another position (3D Euclidean distance)
    pub fn distance_to(&self, other: &MachinePosition) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    /// Format position as string: "X... Y... Z..."
    pub fn format(&self) -> String {
        format!("X{:.3} Y{:.3} Z{:.3}", self.x, self.y, self.z)
    }
}
