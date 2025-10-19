//! 3D Visualization Module
//!
//! Provides advanced 3D visualization capabilities for toolpath preview,
//! machine position tracking, and stock visualization with interactive
//! camera controls.

pub mod visualizer_3d;

pub use visualizer_3d::{
    calculate_bounds, draw_3d_grid, draw_3d_line, draw_machine_position, draw_stock,
    Visualizer3DState,
};
