//! Enhanced 3D Visualizer Module
//!
//! Provides real-time 3D visualization of toolpaths, machine position,
//! and material stock with interactive camera controls.

use crate::types::MoveType;
use eframe::egui;

/// Material for stock visualization
#[derive(Clone, Debug)]
pub struct StockMaterial {
    pub name: String,
    pub color_rgb: (u8, u8, u8),
    pub opacity: f32,
    pub material_type: String,
}

impl Default for StockMaterial {
    fn default() -> Self {
        Self {
            name: "Aluminum".to_string(),
            color_rgb: (200, 200, 200),
            opacity: 0.7,
            material_type: "Metal".to_string(),
        }
    }
}

impl StockMaterial {
    /// Create StockMaterial from material properties
    pub fn from_material_properties(name: &str, material_type: &str) -> Self {
        // Map material types to colors
        let (r, g, b) = match material_type.to_lowercase().as_str() {
            // Metals
            "aluminum" => (200, 200, 200),    // Light gray
            "steel" => (60, 60, 60),          // Dark gray
            "stainless" => (180, 180, 190),   // Light silvery
            "brass" => (184, 130, 11),        // Brass color
            "copper" => (184, 115, 51),       // Copper color
            "titanium" => (210, 180, 140),    // Light bronze
            
            // Plastics
            "acrylic" => (230, 230, 250),     // Very light blue
            "pvc" => (200, 200, 220),         // Light blue
            "abs" => (210, 210, 210),         // Off-white
            "polycarbonate" => (220, 220, 230), // Light lavender
            
            // Wood
            "oak" => (139, 90, 43),           // Brown
            "pine" => (160, 110, 60),         // Light brown
            "maple" => (150, 100, 50),        // Medium brown
            "walnut" => (100, 60, 40),        // Dark brown
            
            // Composites
            "carbon" => (40, 40, 40),         // Very dark
            "fiberglass" => (150, 150, 160),  // Light gray
            "composite" => (130, 130, 140),   // Medium gray
            
            // Default
            _ => (180, 180, 180),             // Medium gray
        };

        Self {
            name: name.to_string(),
            color_rgb: (r, g, b),
            opacity: 0.7,
            material_type: material_type.to_string(),
        }
    }

    /// Adjust opacity (0.0 to 1.0)
    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity.clamp(0.0, 1.0);
        self
    }

    /// Set custom color
    pub fn with_color(mut self, r: u8, g: u8, b: u8) -> Self {
        self.color_rgb = (r, g, b);
        self
    }
}

/// 3D Visualizer state for camera and rendering
#[derive(Clone, Debug)]
pub struct Visualizer3DState {
    /// Camera rotation angles (pitch, yaw, roll)
    pub camera_pitch: f32,
    pub camera_yaw: f32,
    pub camera_roll: f32,
    
    /// Camera zoom level
    pub zoom: f32,
    
    /// Pan offset
    pub pan_x: f32,
    pub pan_y: f32,
    
    /// Show options
    pub show_rapid_moves: bool,
    pub show_feed_moves: bool,
    pub show_arc_moves: bool,
    pub show_machine_position: bool,
    pub show_stock: bool,
    pub show_grid: bool,
    
    /// Stock dimensions
    pub stock_x: f32,
    pub stock_y: f32,
    pub stock_z: f32,
    
    /// Stock material
    pub stock_material: StockMaterial,
}

impl Default for Visualizer3DState {
    fn default() -> Self {
        Self {
            camera_pitch: 45.0,
            camera_yaw: 45.0,
            camera_roll: 0.0,
            zoom: 0.7,  // Default zoom fits grid in visualizer
            pan_x: 0.0,
            pan_y: 0.0,
            show_rapid_moves: true,
            show_feed_moves: true,
            show_arc_moves: true,
            show_machine_position: true,
            show_stock: true,
            show_grid: true,
            stock_x: 100.0,
            stock_y: 100.0,
            stock_z: 50.0,
            stock_material: StockMaterial::default(),
        }
    }
}

impl Visualizer3DState {
    /// Perform 3D rotation transformation
    pub fn rotate_point(&self, x: f32, y: f32, z: f32) -> (f32, f32, f32) {
        let pitch_rad = self.camera_pitch.to_radians();
        let yaw_rad = self.camera_yaw.to_radians();

        // Pitch rotation (around X axis)
        let y1 = y * pitch_rad.cos() - z * pitch_rad.sin();
        let z1 = y * pitch_rad.sin() + z * pitch_rad.cos();

        // Yaw rotation (around Z axis)
        let x2 = x * yaw_rad.cos() - y1 * yaw_rad.sin();
        let y2 = x * yaw_rad.sin() + y1 * yaw_rad.cos();

        (x2, y2, z1)
    }

    /// Project 3D point to 2D screen coordinates
    pub fn project_to_2d(
        &self,
        x: f32,
        y: f32,
        z: f32,
        center: egui::Pos2,
    ) -> egui::Pos2 {
        let (x3d, y3d, z3d) = self.rotate_point(x, y, z);

        // Perspective scaling
        let scale = self.zoom / (3.0 + z3d * 0.01);

        let screen_x = center.x + (x3d * scale) + self.pan_x;
        let screen_y = center.y - (y3d * scale) + self.pan_y;

        egui::pos2(screen_x, screen_y)
    }

    /// Reset camera to default view
    pub fn reset_camera(&mut self) {
        self.camera_pitch = 45.0;
        self.camera_yaw = 45.0;
        self.camera_roll = 0.0;
        self.zoom = 1.0;
        self.pan_x = 0.0;
        self.pan_y = 0.0;
    }

    /// Fit all toolpaths to view
    pub fn fit_to_view(&mut self) {
        // Calculate zoom to fit the grid/stock in the viewport
        // Using the larger of stock dimensions as the reference
        let max_dim = self.stock_x.max(self.stock_y).max(self.stock_z);
        // Default grid size is 100, so scale accordingly
        self.zoom = if max_dim > 0.0 {
            (100.0 / max_dim * 0.7).clamp(0.1, 25.0)
        } else {
            1.0
        };
        self.pan_x = 0.0;
        self.pan_y = 0.0;
    }
}

/// Draw the 3D grid
pub fn draw_3d_grid(
    painter: &egui::Painter,
    state: &Visualizer3DState,
    center: egui::Pos2,
    grid_size: f32,
    step: f32,
) {
    if !state.show_grid {
        return;
    }

    let color = egui::Color32::from_rgba_unmultiplied(100, 100, 100, 128);

    // Draw X-Y plane grid
    let mut x = -grid_size;
    while x <= grid_size {
        let start = state.project_to_2d(x, -grid_size, 0.0, center);
        let end = state.project_to_2d(x, grid_size, 0.0, center);
        painter.line_segment([start, end], egui::Stroke::new(0.5, color));
        x += step;
    }

    let mut y = -grid_size;
    while y <= grid_size {
        let start = state.project_to_2d(-grid_size, y, 0.0, center);
        let end = state.project_to_2d(grid_size, y, 0.0, center);
        painter.line_segment([start, end], egui::Stroke::new(0.5, color));
        y += step;
    }

    // Draw axis indicators
    let origin = state.project_to_2d(0.0, 0.0, 0.0, center);
    let x_end = state.project_to_2d(20.0, 0.0, 0.0, center);
    let y_end = state.project_to_2d(0.0, 20.0, 0.0, center);
    let z_end = state.project_to_2d(0.0, 0.0, 20.0, center);

    painter.line_segment([origin, x_end], egui::Stroke::new(2.0, egui::Color32::RED));
    painter.line_segment([origin, y_end], egui::Stroke::new(2.0, egui::Color32::GREEN));
    painter.line_segment([origin, z_end], egui::Stroke::new(2.0, egui::Color32::BLUE));
}

/// Draw stock/material visualization with material color
pub fn draw_stock(
    painter: &egui::Painter,
    state: &Visualizer3DState,
    center: egui::Pos2,
) {
    if !state.show_stock {
        return;
    }

    let hx = state.stock_x / 2.0;
    let hy = state.stock_y / 2.0;
    let hz = state.stock_z;

    // Draw stock box edges with material color
    let (r, g, b) = state.stock_material.color_rgb;
    let color = egui::Color32::from_rgba_unmultiplied(
        r,
        g,
        b,
        (state.stock_material.opacity * 255.0) as u8,
    );
    let stroke = egui::Stroke::new(1.5, color);

    // Bottom face (Z=0)
    let p1 = state.project_to_2d(-hx, -hy, 0.0, center);
    let p2 = state.project_to_2d(hx, -hy, 0.0, center);
    let p3 = state.project_to_2d(hx, hy, 0.0, center);
    let p4 = state.project_to_2d(-hx, hy, 0.0, center);

    painter.line_segment([p1, p2], stroke);
    painter.line_segment([p2, p3], stroke);
    painter.line_segment([p3, p4], stroke);
    painter.line_segment([p4, p1], stroke);

    // Top face (Z=stock_z)
    let p5 = state.project_to_2d(-hx, -hy, hz, center);
    let p6 = state.project_to_2d(hx, -hy, hz, center);
    let p7 = state.project_to_2d(hx, hy, hz, center);
    let p8 = state.project_to_2d(-hx, hy, hz, center);

    painter.line_segment([p5, p6], stroke);
    painter.line_segment([p6, p7], stroke);
    painter.line_segment([p7, p8], stroke);
    painter.line_segment([p8, p5], stroke);

    // Vertical edges
    painter.line_segment([p1, p5], stroke);
    painter.line_segment([p2, p6], stroke);
    painter.line_segment([p3, p7], stroke);
    painter.line_segment([p4, p8], stroke);
}

/// Draw machine position indicator
pub fn draw_machine_position(
    painter: &egui::Painter,
    state: &Visualizer3DState,
    center: egui::Pos2,
    x: f32,
    y: f32,
    z: f32,
) {
    if !state.show_machine_position {
        return;
    }

    let pos = state.project_to_2d(x, y, z, center);

    // Draw position sphere/cross
    painter.circle_filled(pos, 5.0, egui::Color32::RED);
    painter.circle(
        pos,
        5.0,
        egui::Color32::DARK_RED,
        egui::Stroke::new(2.0, egui::Color32::DARK_RED),
    );

    // Draw coordinate lines
    let cross_size = 8.0;
    painter.line_segment(
        [
            egui::pos2(pos.x - cross_size, pos.y),
            egui::pos2(pos.x + cross_size, pos.y),
        ],
        egui::Stroke::new(1.0, egui::Color32::RED),
    );
    painter.line_segment(
        [
            egui::pos2(pos.x, pos.y - cross_size),
            egui::pos2(pos.x, pos.y + cross_size),
        ],
        egui::Stroke::new(1.0, egui::Color32::RED),
    );
}

/// Draw a 3D line segment
pub fn draw_3d_line(
    painter: &egui::Painter,
    state: &Visualizer3DState,
    center: egui::Pos2,
    x1: f32,
    y1: f32,
    z1: f32,
    x2: f32,
    y2: f32,
    z2: f32,
    stroke: egui::Stroke,
) {
    let start = state.project_to_2d(x1, y1, z1, center);
    let end = state.project_to_2d(x2, y2, z2, center);
    painter.line_segment([start, end], stroke);
}

/// Calculate toolpath bounds for fit-to-view
pub fn calculate_bounds(
    segments: &[crate::types::PathSegment],
) -> Option<(f32, f32, f32, f32, f32, f32)> {
    if segments.is_empty() {
        return None;
    }

    let mut min_x = f32::MAX;
    let mut max_x = f32::MIN;
    let mut min_y = f32::MAX;
    let mut max_y = f32::MIN;
    let mut min_z = f32::MAX;
    let mut max_z = f32::MIN;

    for segment in segments {
        min_x = min_x.min(segment.start.x).min(segment.end.x);
        max_x = max_x.max(segment.start.x).max(segment.end.x);
        min_y = min_y.min(segment.start.y).min(segment.end.y);
        max_y = max_y.max(segment.start.y).max(segment.end.y);
        min_z = min_z.min(segment.start.z).min(segment.end.z);
        max_z = max_z.max(segment.start.z).max(segment.end.z);
    }

    Some((min_x, max_x, min_y, max_y, min_z, max_z))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stock_material_default() {
        let material = StockMaterial::default();
        assert_eq!(material.name, "Aluminum");
        assert_eq!(material.opacity, 0.7);
        assert_eq!(material.color_rgb, (200, 200, 200));
    }

    #[test]
    fn test_stock_material_from_properties() {
        let al_material = StockMaterial::from_material_properties("Al 6061", "Aluminum");
        assert_eq!(al_material.color_rgb, (200, 200, 200)); // Light gray

        let steel_material = StockMaterial::from_material_properties("AISI 1018", "Steel");
        assert_eq!(steel_material.color_rgb, (60, 60, 60)); // Dark gray

        let brass_material = StockMaterial::from_material_properties("Brass", "Brass");
        assert_eq!(brass_material.color_rgb, (184, 130, 11)); // Brass color
    }

    #[test]
    fn test_stock_material_with_opacity() {
        let material = StockMaterial::default().with_opacity(0.5);
        assert_eq!(material.opacity, 0.5);

        // Test clamping
        let material = StockMaterial::default().with_opacity(1.5);
        assert_eq!(material.opacity, 1.0);

        let material = StockMaterial::default().with_opacity(-0.5);
        assert_eq!(material.opacity, 0.0);
    }

    #[test]
    fn test_stock_material_with_color() {
        let material = StockMaterial::default().with_color(100, 150, 200);
        assert_eq!(material.color_rgb, (100, 150, 200));
    }

    #[test]
    fn test_visualizer_3d_state_with_material() {
        let state = Visualizer3DState::default();
        assert_eq!(state.stock_material.name, "Aluminum");
        assert!(state.show_stock);
    }

    #[test]
    fn test_visualizer_3d_state_default() {
        let state = Visualizer3DState::default();
        assert_eq!(state.camera_pitch, 45.0);
        assert_eq!(state.camera_yaw, 45.0);
        assert_eq!(state.zoom, 0.7);  // Default zoom fits grid in visualizer
        assert!(state.show_rapid_moves);
        assert!(state.show_feed_moves);
        assert!(state.show_arc_moves);
    }

    #[test]
    fn test_visualizer_3d_reset_camera() {
        let mut state = Visualizer3DState::default();
        state.camera_pitch = 30.0;
        state.camera_yaw = 60.0;
        state.zoom = 2.0;
        state.reset_camera();
        
        assert_eq!(state.camera_pitch, 45.0);
        assert_eq!(state.camera_yaw, 45.0);
        assert_eq!(state.zoom, 1.0);
    }

    #[test]
    fn test_rotate_point() {
        let state = Visualizer3DState::default();
        let (x, y, z) = state.rotate_point(1.0, 0.0, 0.0);
        // Should rotate around axes
        assert!(x.is_finite());
        assert!(y.is_finite());
        assert!(z.is_finite());
    }

    #[test]
    fn test_project_to_2d() {
        let state = Visualizer3DState::default();
        let center = egui::pos2(100.0, 100.0);
        let pos = state.project_to_2d(0.0, 0.0, 0.0, center);
        
        // Origin should project near center
        assert!((pos.x - center.x).abs() < 20.0);
        assert!((pos.y - center.y).abs() < 20.0);
    }

    #[test]
    fn test_fit_to_view() {
        let mut state = Visualizer3DState::default();
        state.zoom = 5.0;
        state.pan_x = 100.0;
        state.pan_y = 50.0;
        
        state.fit_to_view();
        // With stock 100x100, zoom should be ~0.7
        assert!(state.zoom >= 0.5 && state.zoom <= 1.0);
        assert_eq!(state.pan_x, 0.0);
        assert_eq!(state.pan_y, 0.0);
    }

    #[test]
    fn test_calculate_bounds_empty() {
        let segments: Vec<crate::types::PathSegment> = vec![];
        assert!(calculate_bounds(&segments).is_none());
    }
}
