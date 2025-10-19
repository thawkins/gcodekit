use crate::designer::bitmap_processing::ThresholdMethod;
use crate::GcodeKitApp;
use eframe::egui;

pub fn show_bitmap_import_widget(ui: &mut egui::Ui, app: &mut GcodeKitApp) {
    ui.group(|ui| {
        ui.label("Bitmap Import & Vectorization");

        ui.horizontal(|ui| {
            ui.label("Threshold Method:");
            ui.selectable_value(
                &mut app.cam.vectorization_config.threshold_method,
                ThresholdMethod::Otsu,
                "Otsu",
            );
            ui.selectable_value(
                &mut app.cam.vectorization_config.threshold_method,
                ThresholdMethod::Fixed,
                "Fixed",
            );
            ui.selectable_value(
                &mut app.cam.vectorization_config.threshold_method,
                ThresholdMethod::Adaptive,
                "Adaptive",
            );
        });

        if matches!(
            app.cam.vectorization_config.threshold_method,
            ThresholdMethod::Fixed
        ) {
            ui.horizontal(|ui| {
                ui.label("Threshold Value:");
                ui.add(
                    egui::DragValue::new(&mut app.cam.vectorization_config.threshold_value)
                        .range(0..=255),
                );
            });
        }

        ui.checkbox(
            &mut app.cam.vectorization_config.noise_reduction,
            "Noise Reduction",
        );
        ui.checkbox(&mut app.cam.vectorization_config.smoothing, "Smoothing");

        ui.horizontal(|ui| {
            ui.label("Min Contour Length:");
            ui.add(
                egui::DragValue::new(&mut app.cam.vectorization_config.min_contour_length)
                    .range(3..=1000),
            );
        });

        ui.horizontal(|ui| {
            ui.label("Simplification Tolerance:");
            ui.add(
                egui::DragValue::new(&mut app.cam.vectorization_config.simplification_tolerance)
                    .range(0.1..=10.0),
            );
        });

        if ui.button("Load Image for Engraving").clicked() {
            app.load_image_for_engraving();
        }

        // Show loaded image status
        if let Some(path) = &app.cam.image_path {
            ui.label(format!("Loaded: {}", path));
            ui.label(format!(
                "Dimensions: {}x{} pixels",
                app.cam.image_width, app.cam.image_height
            ));
            
            if ui.button("Generate Engraving G-code").clicked() {
                app.generate_image_engraving();
            }
        } else {
            ui.label("No image loaded");
        }
    });
}
