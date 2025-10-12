pub mod connection;
pub mod gcode_loading;
pub mod image_engraving;
pub mod jigsaw;
pub mod jog;
pub mod overrides;
pub mod shape_generation;
pub mod tabbed_box;
pub mod toolpath_generation;
pub mod vector_import;

// Re-export the main widget functions for easy access
pub use connection::show_connection_widget;
pub use gcode_loading::show_gcode_loading_widget;
pub use image_engraving::show_image_engraving_widget;
pub use jigsaw::show_jigsaw_widget;
pub use jog::show_jog_widget;
pub use overrides::show_overrides_widget;
pub use shape_generation::show_shape_generation_widget;
pub use tabbed_box::show_tabbed_box_widget;
pub use toolpath_generation::show_toolpath_generation_widget;
pub use vector_import::show_vector_import_widget;
