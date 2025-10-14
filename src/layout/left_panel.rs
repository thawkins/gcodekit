use crate::app::GcodeKitApp;
use egui;

/// Renders the left panel containing machine control widgets.
/// Includes connection settings, jogging controls, and overrides.
pub fn show_left_panel(app: &mut GcodeKitApp, ctx: &egui::Context) {
    if app.ui.show_left_panel {
        let response = egui::SidePanel::left("left_panel")
            .resizable(true)
            .default_width(200.0)
            .show(ctx, |ui| {
                ui.heading("Machine Control");
                ui.separator();

                crate::widgets::show_connection_widget(ui, app.machine.communication.as_mut());
                ui.separator();
                crate::widgets::show_jog_widget(ui, app);
                ui.separator();
                crate::widgets::show_overrides_widget(ui, app);
            });
        app.ui.left_panel_width = response.response.rect.width();
    }
}
