//! UI layout components for gcodekit.
//!
//! This module contains the layout structure and rendering functions
//! for different panels and UI sections of the application.

pub mod bottom_status;
pub mod center_panel;
pub mod left_panel;
pub mod right_panel;
pub mod top_central_panel;
pub mod top_menu;

pub use bottom_status::show_bottom_status;
pub use center_panel::show_center_panel;
pub use left_panel::show_left_panel;
pub use top_central_panel::show_top_central_panel;
pub use top_menu::show_top_menu;
