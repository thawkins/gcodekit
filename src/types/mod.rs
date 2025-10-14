//! Core type definitions and data structures.
//!
//! This module contains fundamental types used throughout the application,
//! including enums, position types, and common data structures.

pub mod enums;
pub mod position;

// Re-export for convenience
pub use enums::{MachineMode, MoveType, PathSegment, Tab};
pub use position::*;
