//! Computer-Aided Manufacturing (CAM) operations.
//!
//! This module provides functionality for generating toolpaths, nesting parts,
//! and managing CAM-related data structures and algorithms.

pub mod nesting;
pub mod stl;
pub mod toolpaths;
pub mod types;

// Re-export for convenience
pub use types::*;
