use crate::GcodeKitApp;

/// Renders the feeds and speeds calculator tab UI.
/// Provides a calculator for determining optimal spindle speeds and feed rates
/// based on material, tool parameters, and cutting conditions.
pub fn show_feeds_speeds_tab(app: &mut GcodeKitApp, ui: &mut egui::Ui) {
    ui.heading("Feeds and Speeds Calculator");

    ui.separator();

    ui.label("This calculator helps determine optimal spindle speeds (RPM) and feed rates for CNC operations.");

    ui.separator();

    // Units selection
    ui.horizontal(|ui| {
        ui.label("Units:");
        ui.radio_value(
            &mut app.ui.feeds_speeds.units_metric,
            false,
            "Imperial (inches, SFM, IPM)",
        );
        ui.radio_value(&mut app.ui.feeds_speeds.units_metric, true, "Metric (mm, SMM, MMPM)");
    });

    ui.separator();

    // Material selection
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
            .selected_text(materials[app.ui.feeds_speeds.material])
            .show_ui(ui, |ui| {
                for (i, mat) in materials.iter().enumerate() {
                    ui.selectable_value(&mut app.ui.feeds_speeds.material, i, *mat);
                }
            });
    });

    // Tool parameters
    let diameter_label = if app.ui.feeds_speeds.units_metric {
        "Tool Diameter (mm):"
    } else {
        "Tool Diameter (inches):"
    };
    let diameter_range = if app.ui.feeds_speeds.units_metric {
        1.0..=50.0
    } else {
        0.0625..=2.0
    };
    let diameter_speed = if app.ui.feeds_speeds.units_metric { 0.1 } else { 0.01 };
    ui.horizontal(|ui| {
        ui.label(diameter_label);
        ui.add(
            egui::DragValue::new(&mut app.ui.feeds_speeds.tool_diameter)
                .speed(diameter_speed)
                .range(diameter_range),
        );
    });
    ui.horizontal(|ui| {
        ui.label("Number of Flutes:");
        ui.add(egui::DragValue::new(&mut app.ui.feeds_speeds.num_flutes).range(1..=6));
    });

    // Operation type
    let operations = ["Roughing", "Finishing"];
    ui.horizontal(|ui| {
        ui.label("Operation:");
        egui::ComboBox::from_id_salt("feeds_speeds_operation")
            .selected_text(operations[app.ui.feeds_speeds.operation])
            .show_ui(ui, |ui| {
                for (i, op) in operations.iter().enumerate() {
                    ui.selectable_value(&mut app.ui.feeds_speeds.operation, i, *op);
                }
            });
    });

    // Tool wear compensation
    ui.horizontal(|ui| {
        ui.label("Tool Wear (%):");
        ui.add(
            egui::DragValue::new(&mut app.ui.feeds_speeds.tool_wear_percent)
                .speed(1.0)
                .range(0.0..=50.0),
        );
        ui.label("(Reduces feed rate for worn tools)");
    });

    ui.separator();

    // Calculate button
    if ui.button("Calculate").clicked() {
        // Perform calculation
        let surface_speed = get_surface_speed(app.ui.feeds_speeds.material, app.ui.feeds_speeds.units_metric);
        app.ui.feeds_speeds.calculated_rpm = calculate_rpm(surface_speed, app.ui.feeds_speeds.tool_diameter, app.ui.feeds_speeds.units_metric);
        let chip_load = get_chip_load(app.ui.feeds_speeds.material, app.ui.feeds_speeds.operation, app.ui.feeds_speeds.units_metric);
        let base_feed_rate = app.ui.feeds_speeds.calculated_rpm * app.ui.feeds_speeds.num_flutes as f32 * chip_load;
        // Apply tool wear compensation
        let wear_factor = 1.0 - (app.ui.feeds_speeds.tool_wear_percent / 100.0);
        app.ui.feeds_speeds.calculated_feed = base_feed_rate * wear_factor;
        app.ui.feeds_speeds.has_results = true;
    }

    if app.ui.feeds_speeds.has_results {
        ui.separator();
        ui.label(format!("Recommended Spindle Speed: {:.0} RPM", app.ui.feeds_speeds.calculated_rpm));
        let feed_unit = if app.ui.feeds_speeds.units_metric {
            "MMPM"
        } else {
            "IPM"
        };
        ui.label(format!(
            "Recommended Feed Rate: {:.2} {}",
            app.ui.feeds_speeds.calculated_feed,
            feed_unit
        ));
        ui.label("Note: These are starting recommendations. Adjust based on your machine capabilities and test cuts.");

        ui.separator();
        ui.horizontal(|ui| {
            ui.label("Export Results:");
            if ui.button("Copy to Clipboard").clicked() {
                let material_name = materials[app.ui.feeds_speeds.material];
                let operation_name = operations[app.ui.feeds_speeds.operation];
                let units_name = if app.ui.feeds_speeds.units_metric {
                    "Metric"
                } else {
                    "Imperial"
                };
                let diameter_unit = if app.ui.feeds_speeds.units_metric {
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
                    app.ui.feeds_speeds.tool_diameter,
                    diameter_unit,
                    app.ui.feeds_speeds.num_flutes,
                    operation_name,
                    app.ui.feeds_speeds.tool_wear_percent,
                    app.ui.feeds_speeds.calculated_rpm,
                    app.ui.feeds_speeds.calculated_feed,
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
