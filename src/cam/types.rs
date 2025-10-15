/// CAM operation types
#[derive(Clone, Debug, PartialEq)]
#[derive(Default)]
pub enum CAMOperation {
    /// Default operation
    #[default]
    None,
    /// 2D contouring operation
    Contour2D {
        depth: f32,
        stepover: f32,
        direction: ContourDirection,
    },
    /// Side profile machining for vertical walls
    SideProfile {
        depth: f32,
        stepover: f32,
        direction: ContourDirection,
        wall_angle: f32, // Draft angle in degrees (0 = vertical)
    },
    /// 3D waterline machining
    Waterline {
        min_z: f32,
        max_z: f32,
        stepdown: f32,
        stepover: f32,
    },
    /// 3D scanline machining
    Scanline {
        min_z: f32,
        max_z: f32,
        stepdown: f32,
        stepover: f32,
        angle: f32, // Scan angle in degrees
    },
    /// Lathe turning operation
    Turning {
        diameter: f32,
        length: f32,
        finish_pass: f32,
        roughing_feed: f32,
        finishing_feed: f32,
    },
    /// Lathe facing operation
    Facing {
        diameter: f32,
        width: f32,
        depth: f32,
        stepover: f32,
    },
    /// Lathe threading operation
    Threading {
        major_diameter: f32,
        minor_diameter: f32,
        pitch: f32,
        length: f32,
    },
}


/// Contour machining direction
#[derive(Clone, Debug, PartialEq)]
pub enum ContourDirection {
    Clockwise,
    CounterClockwise,
    Climb,
    Conventional,
}

/// CAM operation parameters
#[derive(Clone, Debug)]
pub struct CAMParameters {
    pub tool_diameter: f32,
    pub stepdown: f32,
    pub stepover: f32,
    pub feed_rate: f32,
    pub plunge_rate: f32,
    pub spindle_speed: u32,
    pub safe_z: f32,
    pub stock_surface: f32,
    pub final_depth: f32,
    pub tabs_enabled: bool,
    pub tab_height: f32,
    pub tab_width: f32,
    pub lead_in_enabled: bool,
    pub lead_in_length: f32,
    pub lead_out_enabled: bool,
    pub lead_out_length: f32,
    pub selected_material: Option<String>,
}

impl Default for CAMParameters {
    fn default() -> Self {
        Self {
            tool_diameter: 3.0,
            stepdown: 1.0,
            stepover: 0.6, // 60% of tool diameter
            feed_rate: 100.0,
            plunge_rate: 50.0,
            spindle_speed: 10000,
            safe_z: 5.0,
            stock_surface: 0.0,
            final_depth: -5.0,
            tabs_enabled: false,
            tab_height: 1.0,
            tab_width: 3.0,
            lead_in_enabled: true,
            lead_in_length: 2.0,
            lead_out_enabled: true,
            lead_out_length: 2.0,
            selected_material: None,
        }
    }
}

/// Part nesting configuration
#[derive(Clone, Debug)]
pub struct PartNestingConfig {
    pub sheet_width: f32,
    pub sheet_height: f32,
    pub spacing: f32,
    pub rotation_enabled: bool,
    pub rotation_angles: Vec<f32>, // Angles in degrees
}

/// Nested part position and orientation
#[derive(Clone, Debug)]
pub struct NestedPart {
    pub x: f32,
    pub y: f32,
    pub rotation: f32, // Rotation angle in degrees
    pub part_index: usize,
}

/// 3D point in space
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point3D {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// 3D triangle face with normal
#[derive(Clone, Debug)]
pub struct Triangle {
    pub vertices: [Point3D; 3],
    pub normal: Point3D,
}

/// 3D mesh representation
#[derive(Clone, Debug)]
pub struct Mesh {
    pub triangles: Vec<Triangle>,
    pub bounds: BoundingBox,
}

/// Axis-aligned bounding box
#[derive(Clone, Debug)]
pub struct BoundingBox {
    pub min: Point3D,
    pub max: Point3D,
}

impl Default for BoundingBox {
    fn default() -> Self {
        Self::new()
    }
}

impl BoundingBox {
    pub fn new() -> Self {
        Self {
            min: Point3D {
                x: f32::INFINITY,
                y: f32::INFINITY,
                z: f32::INFINITY,
            },
            max: Point3D {
                x: f32::NEG_INFINITY,
                y: f32::NEG_INFINITY,
                z: f32::NEG_INFINITY,
            },
        }
    }

    pub fn expand(&mut self, point: &Point3D) {
        self.min.x = self.min.x.min(point.x);
        self.min.y = self.min.y.min(point.y);
        self.min.z = self.min.z.min(point.z);
        self.max.x = self.max.x.max(point.x);
        self.max.y = self.max.y.max(point.y);
        self.max.z = self.max.z.max(point.z);
    }
}

/// 3D machining surface types
#[derive(Clone, Debug)]
pub enum SurfaceType {
    Mesh(Mesh),
    HeightMap(Vec<Vec<f32>>), // 2D grid of Z heights
}
