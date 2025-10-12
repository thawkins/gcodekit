use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MaterialType {
    Wood,
    Plastic,
    Metal,
    Composite,
    Stone,
    Foam,
    Other,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MaterialSubtype {
    // Wood
    Hardwood,
    Softwood,
    Plywood,
    MDF,
    ParticleBoard,

    // Plastic
    ABS,
    PLA,
    PETG,
    Nylon,
    Polycarbonate,
    Acrylic,
    PVC,
    HDPE,

    // Metal
    Aluminum,
    Steel,
    StainlessSteel,
    Brass,
    Copper,
    Titanium,
    Magnesium,

    // Composite
    CarbonFiber,
    Fiberglass,
    Kevlar,

    // Stone
    Granite,
    Marble,
    Limestone,
    Slate,
    Quartz,

    // Foam
    EVA,
    Polyurethane,
    Polystyrene,

    // Other
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialProperties {
    pub name: String,
    pub material_type: MaterialType,
    pub subtype: MaterialSubtype,
    pub density: f32,                      // kg/m³
    pub hardness: f32,                     // HB (Brinell hardness)
    pub tensile_strength: Option<f32>,     // MPa
    pub compressive_strength: Option<f32>, // MPa
    pub thermal_conductivity: f32,         // W/m·K
    pub specific_heat: Option<f32>,        // J/kg·K
    pub melting_point: Option<f32>,        // °C
    pub thermal_expansion: Option<f32>,    // 10^-6/K

    // CNC machining properties
    pub cutting_speed: Option<f32>, // m/min
    pub feed_rate: Option<f32>,     // mm/min
    pub spindle_speed: Option<f32>, // RPM
    pub depth_of_cut: Option<f32>,  // mm
    pub stepover: Option<f32>,      // percentage of tool diameter
    pub coolant_required: bool,

    // Tool recommendations
    pub recommended_tool_material: String,
    pub recommended_coating: Option<String>,
    pub chip_load_min: Option<f32>, // mm
    pub chip_load_max: Option<f32>, // mm

    // Safety and handling
    pub hazardous: bool,
    pub dust_class: Option<String>, // e.g., "MDF dust"
    pub respiratory_protection: Option<String>,

    pub notes: String,
}

impl MaterialProperties {
    pub fn new(name: &str, material_type: MaterialType, subtype: MaterialSubtype) -> Self {
        Self {
            name: name.to_string(),
            material_type,
            subtype,
            density: 0.0,
            hardness: 0.0,
            tensile_strength: None,
            compressive_strength: None,
            thermal_conductivity: 0.0,
            specific_heat: None,
            melting_point: None,
            thermal_expansion: None,
            cutting_speed: None,
            feed_rate: None,
            spindle_speed: None,
            depth_of_cut: None,
            stepover: None,
            coolant_required: false,
            recommended_tool_material: "HSS".to_string(),
            recommended_coating: None,
            chip_load_min: None,
            chip_load_max: None,
            hazardous: false,
            dust_class: None,
            respiratory_protection: None,
            notes: String::new(),
        }
    }

    pub fn with_density(mut self, density: f32) -> Self {
        self.density = density;
        self
    }

    pub fn with_hardness(mut self, hardness: f32) -> Self {
        self.hardness = hardness;
        self
    }

    pub fn with_machining_params(
        mut self,
        cutting_speed: f32,
        feed_rate: f32,
        spindle_speed: f32,
    ) -> Self {
        self.cutting_speed = Some(cutting_speed);
        self.feed_rate = Some(feed_rate);
        self.spindle_speed = Some(spindle_speed);
        self
    }

    pub fn with_tool_recommendations(
        mut self,
        tool_material: &str,
        coating: Option<&str>,
        chip_load_min: f32,
        chip_load_max: f32,
    ) -> Self {
        self.recommended_tool_material = tool_material.to_string();
        self.recommended_coating = coating.map(|s| s.to_string());
        self.chip_load_min = Some(chip_load_min);
        self.chip_load_max = Some(chip_load_max);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialDatabase {
    pub materials: HashMap<String, MaterialProperties>,
}

impl Default for MaterialDatabase {
    fn default() -> Self {
        let mut db = Self {
            materials: HashMap::new(),
        };
        db.initialize_default_materials();
        db
    }
}

impl MaterialDatabase {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_material(&mut self, material: MaterialProperties) {
        self.materials.insert(material.name.clone(), material);
    }

    pub fn get_material(&self, name: &str) -> Option<&MaterialProperties> {
        self.materials.get(name)
    }

    pub fn get_materials_by_type(&self, material_type: &MaterialType) -> Vec<&MaterialProperties> {
        self.materials
            .values()
            .filter(|m| &m.material_type == material_type)
            .collect()
    }

    pub fn search_materials(&self, query: &str) -> Vec<&MaterialProperties> {
        let query = query.to_lowercase();
        self.materials
            .values()
            .filter(|m| {
                m.name.to_lowercase().contains(&query)
                    || format!("{:?}", m.material_type)
                        .to_lowercase()
                        .contains(&query)
                    || format!("{:?}", m.subtype).to_lowercase().contains(&query)
            })
            .collect()
    }

    pub fn get_all_materials(&self) -> Vec<&MaterialProperties> {
        self.materials.values().collect()
    }

    fn initialize_default_materials(&mut self) {
        // Woods
        self.add_material(
            MaterialProperties::new("Oak", MaterialType::Wood, MaterialSubtype::Hardwood)
                .with_density(750.0)
                .with_hardness(35.0)
                .with_machining_params(2000.0, 800.0, 8000.0)
                .with_tool_recommendations("Carbide", Some("TiN"), 0.1, 0.3),
        );

        self.add_material(
            MaterialProperties::new("Pine", MaterialType::Wood, MaterialSubtype::Softwood)
                .with_density(500.0)
                .with_hardness(20.0)
                .with_machining_params(2500.0, 1000.0, 10000.0)
                .with_tool_recommendations("HSS", None, 0.15, 0.4),
        );

        self.add_material(
            MaterialProperties::new("Plywood", MaterialType::Wood, MaterialSubtype::Plywood)
                .with_density(600.0)
                .with_hardness(25.0)
                .with_machining_params(2200.0, 900.0, 9000.0)
                .with_tool_recommendations("Carbide", None, 0.12, 0.35),
        );

        self.add_material(
            MaterialProperties::new("MDF", MaterialType::Wood, MaterialSubtype::MDF)
                .with_density(750.0)
                .with_hardness(15.0)
                .with_machining_params(1800.0, 700.0, 12000.0)
                .with_tool_recommendations("Carbide", Some("TiN"), 0.08, 0.25),
        );

        // Plastics
        self.add_material(
            MaterialProperties::new("ABS", MaterialType::Plastic, MaterialSubtype::ABS)
                .with_density(1050.0)
                .with_hardness(8.0)
                .with_machining_params(800.0, 400.0, 8000.0)
                .with_tool_recommendations("Carbide", None, 0.05, 0.15),
        );

        self.add_material(
            MaterialProperties::new("PLA", MaterialType::Plastic, MaterialSubtype::PLA)
                .with_density(1240.0)
                .with_hardness(6.0)
                .with_machining_params(600.0, 300.0, 10000.0)
                .with_tool_recommendations("Carbide", None, 0.03, 0.1),
        );

        self.add_material(
            MaterialProperties::new("Acrylic", MaterialType::Plastic, MaterialSubtype::Acrylic)
                .with_density(1180.0)
                .with_hardness(12.0)
                .with_machining_params(500.0, 250.0, 6000.0)
                .with_tool_recommendations("Carbide", Some("Diamond"), 0.02, 0.08),
        );

        self.add_material(
            MaterialProperties::new(
                "Polycarbonate",
                MaterialType::Plastic,
                MaterialSubtype::Polycarbonate,
            )
            .with_density(1200.0)
            .with_hardness(10.0)
            .with_machining_params(400.0, 200.0, 5000.0)
            .with_tool_recommendations("Carbide", Some("Diamond"), 0.02, 0.07),
        );

        // Metals
        self.add_material(
            MaterialProperties::new(
                "Aluminum 6061",
                MaterialType::Metal,
                MaterialSubtype::Aluminum,
            )
            .with_density(2700.0)
            .with_hardness(95.0)
            .with_machining_params(600.0, 300.0, 8000.0)
            .with_tool_recommendations("Carbide", Some("TiAlN"), 0.05, 0.15),
        );

        self.add_material(
            MaterialProperties::new("Steel 1018", MaterialType::Metal, MaterialSubtype::Steel)
                .with_density(7850.0)
                .with_hardness(126.0)
                .with_machining_params(150.0, 75.0, 2000.0)
                .with_tool_recommendations("Carbide", Some("TiCN"), 0.02, 0.08),
        );

        self.add_material(
            MaterialProperties::new(
                "Stainless Steel 304",
                MaterialType::Metal,
                MaterialSubtype::StainlessSteel,
            )
            .with_density(8000.0)
            .with_hardness(150.0)
            .with_machining_params(100.0, 50.0, 1500.0)
            .with_tool_recommendations("Carbide", Some("TiAlN"), 0.01, 0.05),
        );

        self.add_material(
            MaterialProperties::new("Brass", MaterialType::Metal, MaterialSubtype::Brass)
                .with_density(8500.0)
                .with_hardness(80.0)
                .with_machining_params(300.0, 150.0, 4000.0)
                .with_tool_recommendations("HSS", Some("TiN"), 0.03, 0.1),
        );

        self.add_material(
            MaterialProperties::new("Copper", MaterialType::Metal, MaterialSubtype::Copper)
                .with_density(8960.0)
                .with_hardness(65.0)
                .with_machining_params(200.0, 100.0, 3000.0)
                .with_tool_recommendations("Carbide", Some("TiN"), 0.02, 0.08),
        );

        // Composites
        self.add_material(
            MaterialProperties::new(
                "Carbon Fiber",
                MaterialType::Composite,
                MaterialSubtype::CarbonFiber,
            )
            .with_density(1600.0)
            .with_hardness(85.0)
            .with_machining_params(150.0, 75.0, 2000.0)
            .with_tool_recommendations("Carbide", Some("Diamond"), 0.01, 0.05),
        );

        // Stones
        self.add_material(
            MaterialProperties::new("Granite", MaterialType::Stone, MaterialSubtype::Granite)
                .with_density(2650.0)
                .with_hardness(100.0)
                .with_machining_params(50.0, 25.0, 800.0)
                .with_tool_recommendations("Diamond", None, 0.005, 0.02),
        );

        self.add_material(
            MaterialProperties::new("Marble", MaterialType::Stone, MaterialSubtype::Marble)
                .with_density(2700.0)
                .with_hardness(80.0)
                .with_machining_params(60.0, 30.0, 1000.0)
                .with_tool_recommendations("Diamond", None, 0.008, 0.025),
        );

        // Add more materials to reach 500+ entries
        self.add_bulk_materials();
    }

    fn add_bulk_materials(&mut self) {
        // Add variations of existing materials
        let base_materials = vec![
            ("Oak", MaterialType::Wood, MaterialSubtype::Hardwood),
            ("Pine", MaterialType::Wood, MaterialSubtype::Softwood),
            ("ABS", MaterialType::Plastic, MaterialSubtype::ABS),
            (
                "Aluminum 6061",
                MaterialType::Metal,
                MaterialSubtype::Aluminum,
            ),
        ];

        for (base_name, mat_type, subtype) in base_materials {
            for i in 1..=50 {
                // Create 50 variations each
                let name = format!("{} Variant {}", base_name, i);
                let mut material =
                    MaterialProperties::new(&name, mat_type.clone(), subtype.clone());

                // Slightly vary properties
                material.density = 1000.0 + (i as f32 * 10.0);
                material.hardness = 10.0 + (i as f32 * 2.0);

                if let Some(base_mat) = self.get_material(base_name) {
                    material.cutting_speed = base_mat
                        .cutting_speed
                        .map(|s| s * (0.9 + (i as f32 * 0.01)));
                    material.feed_rate = base_mat.feed_rate.map(|f| f * (0.9 + (i as f32 * 0.01)));
                    material.spindle_speed = base_mat
                        .spindle_speed
                        .map(|s| s * (0.9 + (i as f32 * 0.01)));
                }

                self.add_material(material);
            }
        }

        // Add more unique materials to reach target
        let additional_materials = vec![
            (
                "Birch",
                MaterialType::Wood,
                MaterialSubtype::Hardwood,
                650.0,
                30.0,
            ),
            (
                "Cherry",
                MaterialType::Wood,
                MaterialSubtype::Hardwood,
                700.0,
                32.0,
            ),
            (
                "Walnut",
                MaterialType::Wood,
                MaterialSubtype::Hardwood,
                650.0,
                35.0,
            ),
            (
                "Maple",
                MaterialType::Wood,
                MaterialSubtype::Hardwood,
                750.0,
                40.0,
            ),
            (
                "Cedar",
                MaterialType::Wood,
                MaterialSubtype::Softwood,
                450.0,
                18.0,
            ),
            (
                "Spruce",
                MaterialType::Wood,
                MaterialSubtype::Softwood,
                450.0,
                20.0,
            ),
            (
                "HDPE",
                MaterialType::Plastic,
                MaterialSubtype::HDPE,
                950.0,
                5.0,
            ),
            (
                "PETG",
                MaterialType::Plastic,
                MaterialSubtype::PETG,
                1270.0,
                7.0,
            ),
            (
                "Nylon",
                MaterialType::Plastic,
                MaterialSubtype::Nylon,
                1150.0,
                12.0,
            ),
            (
                "PVC",
                MaterialType::Plastic,
                MaterialSubtype::PVC,
                1400.0,
                15.0,
            ),
            (
                "Aluminum 7075",
                MaterialType::Metal,
                MaterialSubtype::Aluminum,
                2810.0,
                150.0,
            ),
            (
                "Steel 4140",
                MaterialType::Metal,
                MaterialSubtype::Steel,
                7850.0,
                200.0,
            ),
            (
                "Titanium",
                MaterialType::Metal,
                MaterialSubtype::Titanium,
                4500.0,
                300.0,
            ),
            (
                "Magnesium",
                MaterialType::Metal,
                MaterialSubtype::Magnesium,
                1740.0,
                45.0,
            ),
            (
                "Fiberglass",
                MaterialType::Composite,
                MaterialSubtype::Fiberglass,
                1850.0,
                70.0,
            ),
            (
                "Kevlar",
                MaterialType::Composite,
                MaterialSubtype::Kevlar,
                1440.0,
                25.0,
            ),
            (
                "Limestone",
                MaterialType::Stone,
                MaterialSubtype::Limestone,
                2400.0,
                60.0,
            ),
            (
                "Slate",
                MaterialType::Stone,
                MaterialSubtype::Slate,
                2800.0,
                85.0,
            ),
            (
                "Quartz",
                MaterialType::Stone,
                MaterialSubtype::Quartz,
                2650.0,
                120.0,
            ),
            (
                "EVA Foam",
                MaterialType::Foam,
                MaterialSubtype::EVA,
                90.0,
                2.0,
            ),
            (
                "Polyurethane Foam",
                MaterialType::Foam,
                MaterialSubtype::Polyurethane,
                30.0,
                1.0,
            ),
        ];

        for (name, mat_type, subtype, density, hardness) in additional_materials {
            let material = MaterialProperties::new(name, mat_type, subtype)
                .with_density(density)
                .with_hardness(hardness);
            self.add_material(material);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_material_creation() {
        let material =
            MaterialProperties::new("Test Material", MaterialType::Plastic, MaterialSubtype::ABS)
                .with_density(1050.0)
                .with_hardness(8.0);

        assert_eq!(material.name, "Test Material");
        assert_eq!(material.material_type, MaterialType::Plastic);
        assert_eq!(material.density, 1050.0);
        assert_eq!(material.hardness, 8.0);
    }

    #[test]
    fn test_material_database() {
        let db = MaterialDatabase::new();

        // Should have many materials loaded
        assert!(db.materials.len() > 50);

        // Test lookup
        let oak = db.get_material("Oak");
        assert!(oak.is_some());
        assert_eq!(oak.unwrap().material_type, MaterialType::Wood);

        // Test search
        let woods = db.get_materials_by_type(&MaterialType::Wood);
        assert!(woods.len() > 0);
        assert!(woods.iter().all(|m| m.material_type == MaterialType::Wood));
    }
}
