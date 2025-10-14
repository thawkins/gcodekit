//! UI widget rendering for gcodekit.
//!
//! This module contains functions for rendering individual UI widgets
//! in a modular way, separating widget logic from panel layout.

use crate::GcodeKitApp;
use eframe::egui;

/// Renders the CAM operations widgets in the right panel.
///
/// This includes shape generation, toolpath generation, vector import,
/// image engraving, tabbed box, and jigsaw widgets.
pub fn render_cam_widgets(ui: &mut egui::Ui, app: &mut GcodeKitApp) {
    // Import the widget functions
    use crate::designer::{
        show_image_engraving_widget, show_jigsaw_widget, show_shape_generation_widget,
        show_tabbed_box_widget, show_toolpath_generation_widget, show_vector_import_widget,
    };

    show_shape_generation_widget(ui, app);
    ui.separator();
    show_toolpath_generation_widget(ui, app);
    ui.separator();
    show_vector_import_widget(ui, app);
    ui.separator();
    show_image_engraving_widget(ui, app);
    ui.separator();
    show_tabbed_box_widget(ui, app);
    ui.separator();
    show_jigsaw_widget(ui, app);
}
