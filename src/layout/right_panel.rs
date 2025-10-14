use crate::app::GcodeKitApp;
use egui;

/// Renders the right panel containing CAM functions and design tools.
/// Includes shape generation, toolpath generation, vector import, image engraving,
/// tabbed box, and jigsaw puzzle widgets.
pub fn show_right_panel(app: &mut GcodeKitApp, ctx: &egui::Context) {
    if app.ui.show_right_panel {
            let response = egui::SidePanel::right("right_panel")
            .resizable(true)
            .default_width(250.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.heading("CAM Functions");

                    crate::designer::show_shape_generation_widget(ui, app);
                    ui.separator();
                    crate::designer::show_toolpath_generation_widget(ui, app);
                    ui.separator();
                    crate::designer::show_vector_import_widget(ui, app);
                    ui.separator();
                    crate::designer::show_image_engraving_widget(ui, app);
                    ui.separator();
                    crate::designer::show_tabbed_box_widget(ui, app);
                    ui.separator();
                    crate::designer::show_jigsaw_widget(ui, app);
                });
            });
        app.ui.right_panel_width = response.response.rect.width();
    }
}
