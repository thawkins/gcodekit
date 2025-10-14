//! UI panel rendering for gcodekit.
//!
//! This module contains functions for rendering the main UI panels
//! in a modular way, separating UI logic from the main update loop.

use crate::GcodeKitApp;
use eframe::egui;

/// Renders all main UI panels for the application.
///
/// This function handles the layout and rendering of top menu, bottom status,
/// left panel, right panel, and center panel in a structured way.
pub fn render_panels(app: &mut GcodeKitApp, ctx: &egui::Context) {
    // Top menu bar
    crate::layout::show_top_menu(app, ctx);

    // Bottom status bar
    crate::layout::show_bottom_status(app, ctx);

    // Left panel - Machine control
    crate::layout::show_left_panel(app, ctx);

    // Right panel - CAM functions
    render_right_panel(app, ctx);

    // Center panel
    crate::layout::show_center_panel(app, ctx);
}

/// Renders the right panel containing CAM functions.
fn render_right_panel(app: &mut GcodeKitApp, ctx: &egui::Context) {
    if app.ui.show_right_panel {
        egui::SidePanel::right("right_panel")
            .resizable(true)
            .default_width(250.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.heading("CAM Functions");
                    crate::ui::widgets::render_cam_widgets(ui, app);
                });
            });
    }
}
