//! User interface components and layout.
//!
//! This module contains UI components, panels, and tab implementations
//! for the gcodekit application interface.

pub mod centralpanel;
pub mod panels;
pub mod state_indicator;
pub mod status_display;
pub mod status_history;
pub mod status_panel;
pub mod tabs;
pub mod widgets;

pub use state_indicator::{AnimatedStateIndicator, StatusIndicatorWidget};
pub use status_display::{
    display_buffer_status, display_feed_and_speed, display_job_progress, display_overrides,
    display_pin_states, display_position, display_state_indicator, display_status_bar,
    display_status_panel, StateColors,
};
pub use status_history::{display_history_overview, display_position_trace, StatusTrendChart};
pub use status_panel::{
    StatusPanelConfig, StatusPanelState, display_status_panel_ui, display_history_panel_ui,
    display_trace_panel_ui, display_status_window,
};
