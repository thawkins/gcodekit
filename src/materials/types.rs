use serde::{Deserialize, Serialize};

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

#[allow(clippy::upper_case_acronyms)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_material_type_equality() {
        assert_eq!(MaterialType::Wood, MaterialType::Wood);
        assert_eq!(MaterialType::Plastic, MaterialType::Plastic);
        assert_eq!(MaterialType::Metal, MaterialType::Metal);
        assert_eq!(MaterialType::Composite, MaterialType::Composite);
        assert_eq!(MaterialType::Stone, MaterialType::Stone);
        assert_eq!(MaterialType::Foam, MaterialType::Foam);
        assert_eq!(MaterialType::Other, MaterialType::Other);
    }

    #[test]
    fn test_material_subtype_equality() {
        assert_eq!(MaterialSubtype::ABS, MaterialSubtype::ABS);
        assert_eq!(MaterialSubtype::Hardwood, MaterialSubtype::Hardwood);
        assert_eq!(MaterialSubtype::Aluminum, MaterialSubtype::Aluminum);
        assert_eq!(MaterialSubtype::CarbonFiber, MaterialSubtype::CarbonFiber);
        assert_eq!(MaterialSubtype::Granite, MaterialSubtype::Granite);
        assert_eq!(MaterialSubtype::EVA, MaterialSubtype::EVA);
        assert_eq!(MaterialSubtype::Custom, MaterialSubtype::Custom);
    }

    #[test]
    fn test_material_type_serialization() {
        let material_type = MaterialType::Metal;
        let serialized = serde_json::to_string(&material_type).expect("serialization failed");
        assert_eq!(serialized, "\"Metal\"");

        let deserialized: MaterialType =
            serde_json::from_str(&serialized).expect("deserialization failed");
        assert_eq!(deserialized, MaterialType::Metal);
    }

    #[test]
    fn test_material_subtype_serialization() {
        let subtype = MaterialSubtype::ABS;
        let serialized = serde_json::to_string(&subtype).expect("serialization failed");
        assert_eq!(serialized, "\"ABS\"");

        let deserialized: MaterialSubtype =
            serde_json::from_str(&serialized).expect("deserialization failed");
        assert_eq!(deserialized, MaterialSubtype::ABS);
    }

    #[test]
    fn test_all_material_types_serialize_deserialize() {
        let types = vec![
            MaterialType::Wood,
            MaterialType::Plastic,
            MaterialType::Metal,
            MaterialType::Composite,
            MaterialType::Stone,
            MaterialType::Foam,
            MaterialType::Other,
        ];

        for material_type in types {
            let serialized = serde_json::to_string(&material_type).expect("serialization failed");
            let deserialized: MaterialType =
                serde_json::from_str(&serialized).expect("deserialization failed");
            assert_eq!(deserialized, material_type);
        }
    }

    #[test]
    fn test_all_material_subtypes_serialize_deserialize() {
        let subtypes = vec![
            MaterialSubtype::Hardwood,
            MaterialSubtype::Softwood,
            MaterialSubtype::Plywood,
            MaterialSubtype::MDF,
            MaterialSubtype::ParticleBoard,
            MaterialSubtype::ABS,
            MaterialSubtype::PLA,
            MaterialSubtype::PETG,
            MaterialSubtype::Nylon,
            MaterialSubtype::Polycarbonate,
            MaterialSubtype::Acrylic,
            MaterialSubtype::PVC,
            MaterialSubtype::HDPE,
            MaterialSubtype::Aluminum,
            MaterialSubtype::Steel,
            MaterialSubtype::StainlessSteel,
            MaterialSubtype::Brass,
            MaterialSubtype::Copper,
            MaterialSubtype::Titanium,
            MaterialSubtype::Magnesium,
            MaterialSubtype::CarbonFiber,
            MaterialSubtype::Fiberglass,
            MaterialSubtype::Kevlar,
            MaterialSubtype::Granite,
            MaterialSubtype::Marble,
            MaterialSubtype::Limestone,
            MaterialSubtype::Slate,
            MaterialSubtype::Quartz,
            MaterialSubtype::EVA,
            MaterialSubtype::Polyurethane,
            MaterialSubtype::Polystyrene,
            MaterialSubtype::Custom,
        ];

        for subtype in subtypes {
            let serialized = serde_json::to_string(&subtype).expect("serialization failed");
            let deserialized: MaterialSubtype =
                serde_json::from_str(&serialized).expect("deserialization failed");
            assert_eq!(deserialized, subtype);
        }
    }
}
