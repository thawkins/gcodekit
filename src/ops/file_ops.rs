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

    /// Exports the current design from the designer module to G-code.
    /// Currently a placeholder - full implementation is TODO.
    pub fn export_design_to_gcode(&mut self) {
        // TODO: Implement design export
        self.machine.status_message = "Exporting design to G-code...".to_string();
    }

    /// Imports a design file for editing in the designer module.
    /// Currently a placeholder - full implementation is TODO.
    pub fn import_design_file(&mut self) {
        // TODO: Implement design import
        self.machine.status_message = "Importing design file...".to_string();
    }
}
