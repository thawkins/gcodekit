use super::types::{MaterialSubtype, MaterialType};
use serde::{Deserialize, Serialize};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_material_properties_new() {
        let material = MaterialProperties::new("Test", MaterialType::Plastic, MaterialSubtype::ABS);

        assert_eq!(material.name, "Test");
        assert_eq!(material.material_type, MaterialType::Plastic);
        assert_eq!(material.subtype, MaterialSubtype::ABS);
        assert_eq!(material.density, 0.0);
        assert_eq!(material.hardness, 0.0);
        assert_eq!(material.thermal_conductivity, 0.0);
        assert_eq!(material.recommended_tool_material, "HSS");
        assert!(!material.coolant_required);
        assert!(!material.hazardous);
        assert!(material.notes.is_empty());
    }

    #[test]
    fn test_with_density() {
        let material =
            MaterialProperties::new("Test", MaterialType::Metal, MaterialSubtype::Aluminum)
                .with_density(2700.0);

        assert_eq!(material.density, 2700.0);
        assert_eq!(material.name, "Test");
        assert_eq!(material.material_type, MaterialType::Metal);
    }

    #[test]
    fn test_with_hardness() {
        let material = MaterialProperties::new("Test", MaterialType::Metal, MaterialSubtype::Steel)
            .with_hardness(150.0);

        assert_eq!(material.hardness, 150.0);
    }

    #[test]
    fn test_with_machining_params() {
        let material =
            MaterialProperties::new("Test", MaterialType::Metal, MaterialSubtype::Aluminum)
                .with_machining_params(100.0, 500.0, 3000.0);

        assert_eq!(material.cutting_speed, Some(100.0));
        assert_eq!(material.feed_rate, Some(500.0));
        assert_eq!(material.spindle_speed, Some(3000.0));
    }

    #[test]
    fn test_with_tool_recommendations() {
        let material =
            MaterialProperties::new("Test", MaterialType::Wood, MaterialSubtype::Hardwood)
                .with_tool_recommendations("Carbide", Some("TiN"), 0.1, 0.3);

        assert_eq!(material.recommended_tool_material, "Carbide");
        assert_eq!(material.recommended_coating, Some("TiN".to_string()));
        assert_eq!(material.chip_load_min, Some(0.1));
        assert_eq!(material.chip_load_max, Some(0.3));
    }

    #[test]
    fn test_with_tool_recommendations_no_coating() {
        let material = MaterialProperties::new("Test", MaterialType::Plastic, MaterialSubtype::ABS)
            .with_tool_recommendations("HSS", None, 0.05, 0.15);

        assert_eq!(material.recommended_tool_material, "HSS");
        assert_eq!(material.recommended_coating, None);
        assert_eq!(material.chip_load_min, Some(0.05));
        assert_eq!(material.chip_load_max, Some(0.15));
    }

    #[test]
    fn test_builder_pattern_chaining() {
        let material = MaterialProperties::new(
            "Aluminum 6061",
            MaterialType::Metal,
            MaterialSubtype::Aluminum,
        )
        .with_density(2700.0)
        .with_hardness(95.0)
        .with_machining_params(200.0, 800.0, 4000.0)
        .with_tool_recommendations("Carbide", Some("TiAlN"), 0.15, 0.4);

        assert_eq!(material.name, "Aluminum 6061");
        assert_eq!(material.material_type, MaterialType::Metal);
        assert_eq!(material.subtype, MaterialSubtype::Aluminum);
        assert_eq!(material.density, 2700.0);
        assert_eq!(material.hardness, 95.0);
        assert_eq!(material.cutting_speed, Some(200.0));
        assert_eq!(material.feed_rate, Some(800.0));
        assert_eq!(material.spindle_speed, Some(4000.0));
        assert_eq!(material.recommended_tool_material, "Carbide");
        assert_eq!(material.recommended_coating, Some("TiAlN".to_string()));
        assert_eq!(material.chip_load_min, Some(0.15));
        assert_eq!(material.chip_load_max, Some(0.4));
    }

    #[test]
    fn test_material_properties_default_values() {
        let material =
            MaterialProperties::new("Test", MaterialType::Wood, MaterialSubtype::Hardwood);

        // Check that optional fields are None by default
        assert!(material.tensile_strength.is_none());
        assert!(material.compressive_strength.is_none());
        assert!(material.specific_heat.is_none());
        assert!(material.melting_point.is_none());
        assert!(material.thermal_expansion.is_none());
        assert!(material.cutting_speed.is_none());
        assert!(material.feed_rate.is_none());
        assert!(material.spindle_speed.is_none());
        assert!(material.depth_of_cut.is_none());
        assert!(material.stepover.is_none());
        assert!(material.recommended_coating.is_none());
        assert!(material.chip_load_min.is_none());
        assert!(material.chip_load_max.is_none());
        assert!(material.dust_class.is_none());
        assert!(material.respiratory_protection.is_none());
    }
}
