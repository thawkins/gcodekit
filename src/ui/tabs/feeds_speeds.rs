use crate::GcodeKitApp;

/// Renders the feeds and speeds calculator tab UI.
/// Provides a calculator for determining optimal spindle speeds and feed rates
/// based on material, tool parameters, and cutting conditions.
pub fn show_feeds_speeds_tab(_app: &mut GcodeKitApp, ui: &mut egui::Ui) {
    ui.heading("Feeds and Speeds Calculator");

    ui.separator();

    ui.label("This calculator helps determine optimal spindle speeds (RPM) and feed rates for CNC operations.");

    ui.separator();

    // Units selection
    static mut UNITS_METRIC: bool = false;
    ui.horizontal(|ui| {
        ui.label("Units:");
        ui.radio_value(
            &mut unsafe { UNITS_METRIC },
            false,
            "Imperial (inches, SFM, IPM)",
        );
        ui.radio_value(&mut unsafe { UNITS_METRIC }, true, "Metric (mm, SMM, MMPM)");
    });

    ui.separator();

    // Material selection
    static mut MATERIAL: usize = 0;
    let materials = [
        "Aluminum",
        "Steel",
        "Stainless Steel",
        "Brass",
        "Titanium",
        "Plastic",
        "Wood",
        "MDF",
        "Carbon Fiber",
        "Fiberglass",
    ];
    ui.horizontal(|ui| {
        ui.label("Material:");
        egui::ComboBox::from_id_salt("feeds_speeds_material")
            .selected_text(materials[unsafe { MATERIAL }])
            .show_ui(ui, |ui| {
                for (i, mat) in materials.iter().enumerate() {
                    ui.selectable_value(&mut unsafe { MATERIAL }, i, *mat);
                }
            });
    });

    // Tool parameters
    static mut TOOL_DIAMETER: f32 = 0.25;
    static mut NUM_FLUTES: u32 = 2;
    let diameter_label = if unsafe { UNITS_METRIC } {
        "Tool Diameter (mm):"
    } else {
        "Tool Diameter (inches):"
    };
    let diameter_range = if unsafe { UNITS_METRIC } {
        1.0..=50.0
    } else {
        0.0625..=2.0
    };
    let diameter_speed = if unsafe { UNITS_METRIC } { 0.1 } else { 0.01 };
    ui.horizontal(|ui| {
        ui.label(diameter_label);
        ui.add(
            egui::DragValue::new(&mut unsafe { TOOL_DIAMETER })
                .speed(diameter_speed)
                .range(diameter_range),
        );
    });
    ui.horizontal(|ui| {
        ui.label("Number of Flutes:");
        ui.add(egui::DragValue::new(&mut unsafe { NUM_FLUTES }).range(1..=6));
    });

    // Operation type
    static mut OPERATION: usize = 0;
    let operations = ["Roughing", "Finishing"];
    ui.horizontal(|ui| {
        ui.label("Operation:");
        egui::ComboBox::from_id_salt("feeds_speeds_operation")
            .selected_text(operations[unsafe { OPERATION }])
            .show_ui(ui, |ui| {
                for (i, op) in operations.iter().enumerate() {
                    ui.selectable_value(&mut unsafe { OPERATION }, i, *op);
                }
            });
    });

    // Tool wear compensation
    static mut TOOL_WEAR_PERCENT: f32 = 0.0;
    ui.horizontal(|ui| {
        ui.label("Tool Wear (%):");
        ui.add(
            egui::DragValue::new(&mut unsafe { TOOL_WEAR_PERCENT })
                .speed(1.0)
                .range(0.0..=50.0),
        );
        ui.label("(Reduces feed rate for worn tools)");
    });

    ui.separator();

    // Calculate button
    static mut CALCULATED_RPM: f32 = 0.0;
    static mut CALCULATED_FEED: f32 = 0.0;
    static mut HAS_RESULTS: bool = false;

    if ui.button("Calculate").clicked() {
        // Perform calculation
        let surface_speed = get_surface_speed(unsafe { MATERIAL }, unsafe { UNITS_METRIC });
        unsafe { CALCULATED_RPM = calculate_rpm(surface_speed, TOOL_DIAMETER, UNITS_METRIC) };
        let chip_load = get_chip_load(unsafe { MATERIAL }, unsafe { OPERATION }, unsafe {
            UNITS_METRIC
        });
        let base_feed_rate = unsafe { CALCULATED_RPM } * unsafe { NUM_FLUTES } as f32 * chip_load;
        // Apply tool wear compensation
        let wear_factor = 1.0 - (unsafe { TOOL_WEAR_PERCENT } / 100.0);
        unsafe { CALCULATED_FEED = base_feed_rate * wear_factor };
        unsafe { HAS_RESULTS = true };
    }

    if unsafe { HAS_RESULTS } {
        ui.separator();
        ui.label(format!("Recommended Spindle Speed: {:.0} RPM", unsafe {
            CALCULATED_RPM
        }));
        let feed_unit = if unsafe { UNITS_METRIC } {
            "MMPM"
        } else {
            "IPM"
        };
        ui.label(format!(
            "Recommended Feed Rate: {:.2} {}",
            unsafe { CALCULATED_FEED },
            feed_unit
        ));
        ui.label("Note: These are starting recommendations. Adjust based on your machine capabilities and test cuts.");

        ui.separator();
        ui.horizontal(|ui| {
            ui.label("Export Results:");
            if ui.button("Copy to Clipboard").clicked() {
                let material_name = materials[unsafe { MATERIAL }];
                let operation_name = operations[unsafe { OPERATION }];
                let units_name = if unsafe { UNITS_METRIC } {
                    "Metric"
                } else {
                    "Imperial"
                };
                let diameter_unit = if unsafe { UNITS_METRIC } {
                    "mm"
                } else {
                    "inches"
                };

                let export_text = format!(
                    "Feeds & Speeds Calculator Results\n\
                     Units: {}\n\
                     Material: {}\n\
                     Tool Diameter: {:.3} {}\n\
                     Number of Flutes: {}\n\
                     Operation: {}\n\
                     Tool Wear: {:.1}%\n\
                     Recommended RPM: {:.0}\n\
                     Recommended Feed Rate: {:.2} {}\n",
                    units_name,
                    material_name,
                    unsafe { TOOL_DIAMETER },
                    diameter_unit,
                    unsafe { NUM_FLUTES },
                    operation_name,
                    unsafe { TOOL_WEAR_PERCENT },
                    unsafe { CALCULATED_RPM },
                    unsafe { CALCULATED_FEED },
                    feed_unit
                );

                ui.ctx().copy_text(export_text);
                ui.label("Results copied to clipboard!");
            }
        });
    }
}

fn get_surface_speed(material_index: usize, metric: bool) -> f32 {
    let imperial_sfm = match material_index {
        0 => 800.0, // Aluminum
        1 => 100.0, // Steel
        2 => 80.0,  // Stainless Steel
        3 => 300.0, // Brass
        4 => 150.0, // Titanium
        5 => 200.0, // Plastic
        6 => 400.0, // Wood
        7 => 300.0, // MDF
        8 => 600.0, // Carbon Fiber
        9 => 400.0, // Fiberglass
        _ => 100.0,
    };

    if metric {
        // Convert SFM to SMM (surface meters per minute)
        imperial_sfm * 0.3048
    } else {
        imperial_sfm
    }
}

fn calculate_rpm(surface_speed: f32, diameter: f32, metric: bool) -> f32 {
    if metric {
        // Metric: RPM = (SMM × 1000) / (π × diameter_mm)
        (surface_speed * 1000.0) / (std::f32::consts::PI * diameter)
    } else {
        // Imperial: RPM = (SFM × 12) / (π × diameter_inches)
        (surface_speed * 12.0) / (std::f32::consts::PI * diameter)
    }
}

fn get_chip_load(material_index: usize, operation_index: usize, metric: bool) -> f32 {
    let imperial_chip_load = match material_index {
        0 => 0.008, // Aluminum
        1 => 0.003, // Steel
        2 => 0.002, // Stainless Steel
        3 => 0.005, // Brass
        4 => 0.002, // Titanium
        5 => 0.010, // Plastic
        6 => 0.015, // Wood
        7 => 0.012, // MDF
        8 => 0.006, // Carbon Fiber
        9 => 0.008, // Fiberglass
        _ => 0.005,
    };

    let base_load = if metric {
        // Convert inches per tooth to mm per tooth
        imperial_chip_load * 25.4
    } else {
        imperial_chip_load
    };

    // Adjust for operation
    if operation_index == 1 {
        // Finishing
        base_load * 0.7
    } else {
        base_load
    }
}
