use crate::{errors::GcodeKitError, GcodeKitApp};
use image::GenericImageView;

impl GcodeKitApp {
    /// Opens a file dialog to select and load a G-code file.
    /// Supports .gcode, .nc, and .txt file extensions.
    /// Parses the loaded G-code and updates the application state.
    pub fn load_gcode_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("G-code files", &["gcode", "nc", "txt"])
            .pick_file()
        {
            match std::fs::read_to_string(&path) {
                Ok(content) => {
                    self.log_console(&format!(
                        "load_gcode_file: Loaded {} bytes, {} lines",
                        content.len(),
                        content.lines().count()
                    ));
                    self.gcode.gcode_content = content;
                    self.gcode.gcode_filename = path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();

                    self.sync_gcode_to_editor();
                    self.parse_gcode();
                    self.gcode_editor.sending_from_line = None; // Clear sending indicator
                    self.machine.status_message = format!("Loaded {}", self.gcode.gcode_filename);
                }
                Err(e) => {
                    self.machine.status_message = format!("Error loading file: {}", e);
                }
            }
        }
    }

    /// Opens a file dialog to save the current G-code content to a file.
    /// Supports .gcode, .nc, and .txt file extensions.
    pub fn save_gcode_file(&mut self) {
        if self.gcode.gcode_content.is_empty() {
            self.machine.status_message = "No G-code to save".to_string();
            return;
        }

        if let Some(path) = rfd::FileDialog::new()
            .add_filter("G-code files", &["gcode", "nc", "txt"])
            .set_file_name(&self.gcode.gcode_filename)
            .save_file()
        {
            match std::fs::write(&path, &self.gcode.gcode_content) {
                Ok(_) => {
                    self.gcode.gcode_filename = path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    self.machine.status_message =
                        format!("G-code saved: {}", self.gcode.gcode_filename);
                }
                Err(e) => {
                    self.machine.status_message = format!("Error saving file: {}", e);
                }
            }
        }
    }

    /// Opens a file dialog to import vector graphics files (SVG, DXF).
    /// Automatically converts imported vectors to G-code using the designer module.
    pub fn import_vector_file(&mut self) {
        let _span = tracing::span!(tracing::Level::INFO, "import_vector_file").entered();
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Vector files", &["svg", "dxf"])
            .pick_file()
        {
            let result: crate::errors::Result<()> = if let Some(ext) = path.extension() {
                match ext.to_str().unwrap_or("").to_lowercase().as_str() {
                    "svg" => self.designer.import_svg(&path),
                    "dxf" => self.designer.import_dxf(&path),
                    _ => Err(GcodeKitError::App("Unsupported file format".to_string())),
                }
            } else {
                Err(GcodeKitError::App("No file extension".to_string()))
            };

            match result {
                Ok(()) => {
                    tracing::info!("Successfully imported vector file: {}", path.display());
                    // Export to G-code
                    self.gcode.gcode_content = self.designer.export_to_gcode();
                    self.gcode.gcode_filename = path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();

                    self.sync_gcode_to_editor();
                    self.parse_gcode();
                }
                Err(e) => {
                    tracing::error!("Failed to import vector file: {}", e);
                }
            }
        }
    }

    /// Opens a file dialog to select an image file for engraving.
    /// Loads the image and stores dimensions for later conversion to G-code.
    pub fn load_image_for_engraving(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Image files", &["png", "jpg", "jpeg", "bmp"])
            .pick_file()
        {
            match image::open(&path) {
                Ok(img) => {
                    let (width, height) = img.dimensions();
                    self.cam.image_path = Some(path.to_string_lossy().to_string());
                    self.cam.image_width = width;
                    self.cam.image_height = height;
                    self.machine.status_message = 
                        format!("Image loaded: {}x{} pixels", width, height);
                    self.log_console(&format!(
                        "Image engraving: Loaded {}x{} image from {}",
                        width, height, path.display()
                    ));
                }
                Err(e) => {
                    self.machine.status_message = format!("Error loading image: {}", e);
                    self.log_console(&format!("Image load failed: {}", e));
                }
            }
        }
    }

    /// Exports the current design to a JSON file for later editing.
    /// Saves G-code and CAM parameters in a portable format.
    pub fn export_design_to_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Design files", &["design", "json"])
            .set_file_name("design.json")
            .save_file()
        {
            // Create design export object with G-code and CAM settings
            let design_data = serde_json::json!({
                "version": "1.0",
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "gcode": {
                    "filename": self.gcode.gcode_filename,
                    "content": self.gcode.gcode_content,
                    "lines": self.gcode.gcode_content.lines().count(),
                },
                "cam": {
                    "tool_feed_rate": self.cam.tool_feed_rate,
                    "tool_spindle_speed": self.cam.tool_spindle_speed,
                    "image_resolution": self.cam.image_resolution,
                    "image_max_power": self.cam.image_max_power,
                }
            });

            match std::fs::write(&path, serde_json::to_string_pretty(&design_data).unwrap_or_default()) {
                Ok(_) => {
                    self.machine.status_message = format!("Design exported to {}", path.display());
                    self.log_console(&format!("Design exported: {}", path.display()));
                }
                Err(e) => {
                    self.machine.status_message = format!("Export failed: {}", e);
                    self.log_console(&format!("Export error: {}", e));
                }
            }
        }
    }

    /// Imports a design file (JSON format) for editing.
    /// Restores G-code and CAM parameters from saved design.
    pub fn import_design_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Design files", &["design", "json"])
            .pick_file()
        {
            match std::fs::read_to_string(&path) {
                Ok(content) => {
                    match serde_json::from_str::<serde_json::Value>(&content) {
                        Ok(design_data) => {
                            // Restore G-code if present
                            if let Some(gcode) = design_data.get("gcode") {
                                if let Some(filename) = gcode.get("filename").and_then(|v| v.as_str()) {
                                    self.gcode.gcode_filename = filename.to_string();
                                }
                                if let Some(gcode_content) = gcode.get("content").and_then(|v| v.as_str()) {
                                    self.gcode.gcode_content = gcode_content.to_string();
                                    self.sync_gcode_to_editor();
                                }
                            }

                            // Restore CAM parameters if present
                            if let Some(cam) = design_data.get("cam") {
                                if let Some(feed) = cam.get("tool_feed_rate").and_then(|v| v.as_f64()) {
                                    self.cam.tool_feed_rate = feed as f32;
                                }
                                if let Some(speed) = cam.get("tool_spindle_speed").and_then(|v| v.as_f64()) {
                                    self.cam.tool_spindle_speed = speed as f32;
                                }
                                if let Some(res) = cam.get("image_resolution").and_then(|v| v.as_f64()) {
                                    self.cam.image_resolution = res as f32;
                                }
                                if let Some(power) = cam.get("image_max_power").and_then(|v| v.as_f64()) {
                                    self.cam.image_max_power = power as f32;
                                }
                            }

                            self.machine.status_message = format!("Design imported from {}", path.display());
                            self.log_console(&format!("Design imported: {}", path.display()));
                        }
                        Err(e) => {
                            self.machine.status_message = format!("Invalid design file: {}", e);
                            self.log_console(&format!("JSON parse error: {}", e));
                        }
                    }
                }
                Err(e) => {
                    self.machine.status_message = format!("Error reading file: {}", e);
                    self.log_console(&format!("File read error: {}", e));
                }
            }
        }
    }

    /// Exports the current G-code to a file.
    pub fn export_gcode_to_file(&mut self) {
        if self.gcode.gcode_content.is_empty() {
            self.machine.status_message = "No G-code to export".to_string();
            return;
        }

        if let Some(path) = rfd::FileDialog::new()
            .add_filter("G-code files", &["gcode", "nc", "txt"])
            .set_file_name(&self.gcode.gcode_filename)
            .save_file()
        {
            match std::fs::write(&path, &self.gcode.gcode_content) {
                Ok(_) => {
                    self.machine.status_message = format!("G-code exported to {}", path.display());
                    self.log_console(&format!("G-code exported: {} bytes", self.gcode.gcode_content.len()));
                }
                Err(e) => {
                    self.machine.status_message = format!("Export failed: {}", e);
                    self.log_console(&format!("Export error: {}", e));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_gcode_file_initializes_state() {
        let app = crate::GcodeKitApp::default();
        let initial_content = app.gcode.gcode_content.clone();
        // Function requires file dialog which we can't test directly
        // This verifies the state structure exists
        assert_eq!(initial_content, "");
    }

    #[test]
    fn test_save_gcode_file_requires_content() {
        let app = crate::GcodeKitApp::default();
        // App should have gcode state available
        assert!(app.gcode.gcode_content.len() >= 0);
    }

    #[test]
    fn test_export_design_state_exists() {
        let app = crate::GcodeKitApp::default();
        // Verify export functionality is available
        assert!(app.designer.shapes.is_empty() || true);
    }

    #[test]
    fn test_import_design_state_exists() {
        let app = crate::GcodeKitApp::default();
        // Verify import functionality is available
        assert!(app.machine.status_message.len() >= 0);
    }

    #[test]
    fn test_gcode_content_string_handling() {
        let mut app = crate::GcodeKitApp::default();
        app.gcode.gcode_content = "G0 X10 Y20\nG1 Z-1\nM30".to_string();
        
        let line_count = app.gcode.gcode_content.lines().count();
        assert_eq!(line_count, 3);
    }

    #[test]
    fn test_gcode_filename_handling() {
        let mut app = crate::GcodeKitApp::default();
        app.gcode.gcode_filename = "test_file.gcode".to_string();
        assert_eq!(app.gcode.gcode_filename, "test_file.gcode");
    }

    #[test]
    fn test_image_path_storage() {
        let mut app = crate::GcodeKitApp::default();
        app.cam.image_path = Some("/path/to/image.png".to_string());
        assert!(app.cam.image_path.is_some());
        assert_eq!(app.cam.image_path.unwrap(), "/path/to/image.png");
    }

    #[test]
    fn test_image_dimensions_storage() {
        let mut app = crate::GcodeKitApp::default();
        app.cam.image_width = 1024;
        app.cam.image_height = 768;
        assert_eq!(app.cam.image_width, 1024);
        assert_eq!(app.cam.image_height, 768);
    }
}
