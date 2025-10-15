use gcodekit::materials::*;

#[cfg(test)]
mod material_database_tests {
    use super::*;

    #[test]
    fn test_material_database_creation() {
        let db = MaterialDatabase::default();
        assert!(!db.materials.is_empty());
    }

    #[test]
    fn test_material_database_get_material() {
        let db = MaterialDatabase::default();

        if let Some(wood) = db.get_material("Plywood") {
            assert_eq!(wood.name, "Plywood");
            assert_eq!(wood.material_type, MaterialType::Wood);
        } else {
            // If Plywood doesn't exist, just verify we can call the method
            assert!(db.get_material("NonExistent").is_none());
        }
    }

    #[test]
    fn test_material_database_add_material() {
        let mut db = MaterialDatabase::default();
        let initial_count = db.materials.len();

        let new_material = MaterialProperties::new(
            "Custom Aluminum",
            MaterialType::Metal,
            MaterialSubtype::Custom,
        );

        db.add_material(new_material);
        assert_eq!(db.materials.len(), initial_count + 1);

        let retrieved = db.get_material("Custom Aluminum");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "Custom Aluminum");
    }

    #[test]
    fn test_material_database_remove_material() {
        let mut db = MaterialDatabase::default();

        let test_material = MaterialProperties::new(
            "Test Material",
            MaterialType::Wood,
            MaterialSubtype::Custom,
        );

        db.add_material(test_material);
        assert!(db.get_material("Test Material").is_some());

        let removed = db.remove_material("Test Material");
        assert!(removed.is_some());
        assert!(db.get_material("Test Material").is_none());
    }

    #[test]
    fn test_material_database_list_materials() {
        let db = MaterialDatabase::default();
        let materials = db.list_materials();

        assert!(!materials.is_empty());

        // Check that all returned materials are actually in the database
        for material_name in materials {
            assert!(db.get_material(material_name).is_some());
        }
    }

    #[test]
    fn test_material_database_get_by_type() {
        let mut db = MaterialDatabase::default();

        // Add some test materials
        db.add_material(MaterialProperties::new(
            "Test Wood 1",
            MaterialType::Wood,
            MaterialSubtype::Custom,
        ));
        db.add_material(MaterialProperties::new(
            "Test Wood 2",
            MaterialType::Wood,
            MaterialSubtype::Custom,
        ));
        db.add_material(MaterialProperties::new(
            "Test Metal",
            MaterialType::Metal,
            MaterialSubtype::Custom,
        ));

        let woods = db.get_materials_by_type(&MaterialType::Wood);
        assert!(woods.len() >= 2); // At least our two test woods

        let metals = db.get_materials_by_type(&MaterialType::Metal);
        assert!(metals.len() >= 1); // At least our test metal
    }
}

#[cfg(test)]
mod material_properties_tests {
    use super::*;

    #[test]
    fn test_material_properties_creation() {
        let material = MaterialProperties::new(
            "Oak",
            MaterialType::Wood,
            MaterialSubtype::Hardwood,
        );

        assert_eq!(material.name, "Oak");
        assert_eq!(material.material_type, MaterialType::Wood);
        assert_eq!(material.subtype, MaterialSubtype::Hardwood);
        assert_eq!(material.density, 0.0);
        assert_eq!(material.hardness, 0.0);
    }

    #[test]
    fn test_material_properties_builder_pattern() {
        let material = MaterialProperties::new(
            "Aluminum 6061",
            MaterialType::Metal,
            MaterialSubtype::Aluminum,
        )
        .with_density(2700.0)
        .with_hardness(95.0)
        .with_cutting_speed(200.0)
        .with_recommended_feed_rate(150.0)
        .with_recommended_spindle_speed(12000.0);

        assert_eq!(material.name, "Aluminum 6061");
        assert_eq!(material.density, 2700.0);
        assert_eq!(material.hardness, 95.0);
        assert_eq!(material.cutting_speed, 200.0);
        assert_eq!(material.recommended_feed_rate, 150.0);
        assert_eq!(material.recommended_spindle_speed, 12000.0);
    }

    #[test]
    fn test_material_properties_with_tool_recommendations() {
        let material = MaterialProperties::new(
            "Stainless Steel",
            MaterialType::Metal,
            MaterialSubtype::Steel,
        )
        .with_recommended_tool_material("Carbide")
        .with_recommended_tool_coating("TiN");

        assert_eq!(material.recommended_tool_material, "Carbide");
        assert_eq!(material.recommended_tool_coating, "TiN");
    }

    #[test]
    fn test_material_properties_with_chip_load() {
        let material = MaterialProperties::new(
            "Brass",
            MaterialType::Metal,
            MaterialSubtype::Custom,
        )
        .with_chip_load_range(0.002, 0.006);

        assert_eq!(material.chip_load_min, 0.002);
        assert_eq!(material.chip_load_max, 0.006);
    }

    #[test]
    fn test_material_properties_clone() {
        let material1 = MaterialProperties::new(
            "Test Material",
            MaterialType::Plastic,
            MaterialSubtype::Custom,
        )
        .with_density(1200.0);

        let material2 = material1.clone();

        assert_eq!(material1.name, material2.name);
        assert_eq!(material1.material_type, material2.material_type);
        assert_eq!(material1.density, material2.density);
    }
}

#[cfg(test)]
mod material_type_tests {
    use super::*;

    #[test]
    fn test_material_type_enum() {
        let types = vec![
            MaterialType::Wood,
            MaterialType::Metal,
            MaterialType::Plastic,
            MaterialType::Composite,
            MaterialType::Stone,
            MaterialType::Foam,
        ];

        for mat_type in types {
            let cloned = mat_type.clone();
            assert_eq!(mat_type, cloned);
        }
    }

    #[test]
    fn test_material_type_debug() {
        assert_eq!(format!("{:?}", MaterialType::Wood), "Wood");
        assert_eq!(format!("{:?}", MaterialType::Metal), "Metal");
        assert_eq!(format!("{:?}", MaterialType::Plastic), "Plastic");
    }

    #[test]
    fn test_material_subtype_enum() {
        let subtypes = vec![
            MaterialSubtype::Hardwood,
            MaterialSubtype::Softwood,
            MaterialSubtype::Steel,
            MaterialSubtype::Aluminum,
            MaterialSubtype::Brass,
            MaterialSubtype::Custom,
        ];

        for subtype in subtypes {
            let cloned = subtype.clone();
            assert_eq!(subtype, cloned);
        }
    }

    #[test]
    fn test_material_type_default() {
        let mat_type = MaterialType::default();
        assert_eq!(mat_type, MaterialType::Wood);
    }
}

#[cfg(test)]
mod material_recommendations_tests {
    use super::*;

    #[test]
    fn test_wood_recommendations() {
        let wood = MaterialProperties::new(
            "Maple",
            MaterialType::Wood,
            MaterialSubtype::Hardwood,
        )
        .with_cutting_speed(300.0)
        .with_recommended_feed_rate(200.0)
        .with_recommended_spindle_speed(15000.0);

        assert!(wood.cutting_speed > 0.0);
        assert!(wood.recommended_feed_rate > 0.0);
        assert!(wood.recommended_spindle_speed > 0.0);
    }

    #[test]
    fn test_metal_recommendations() {
        let metal = MaterialProperties::new(
            "Mild Steel",
            MaterialType::Metal,
            MaterialSubtype::Steel,
        )
        .with_cutting_speed(100.0)
        .with_recommended_feed_rate(80.0)
        .with_recommended_spindle_speed(8000.0)
        .with_chip_load_range(0.001, 0.003);

        assert!(metal.cutting_speed > 0.0);
        assert!(metal.chip_load_min > 0.0);
        assert!(metal.chip_load_max > metal.chip_load_min);
    }

    #[test]
    fn test_plastic_recommendations() {
        let plastic = MaterialProperties::new(
            "Acrylic",
            MaterialType::Plastic,
            MaterialSubtype::Custom,
        )
        .with_cutting_speed(250.0)
        .with_recommended_feed_rate(150.0);

        assert!(plastic.cutting_speed > 0.0);
        assert!(plastic.recommended_feed_rate > 0.0);
    }
}

#[cfg(test)]
mod material_validation_tests {
    use super::*;

    #[test]
    fn test_material_properties_validation() {
        let material = MaterialProperties::new(
            "Test",
            MaterialType::Wood,
            MaterialSubtype::Custom,
        )
        .with_density(600.0)
        .with_hardness(50.0)
        .with_cutting_speed(300.0)
        .with_recommended_feed_rate(200.0)
        .with_recommended_spindle_speed(15000.0);

        // All values should be non-negative
        assert!(material.density >= 0.0);
        assert!(material.hardness >= 0.0);
        assert!(material.cutting_speed >= 0.0);
        assert!(material.recommended_feed_rate >= 0.0);
        assert!(material.recommended_spindle_speed >= 0.0);
    }

    #[test]
    fn test_chip_load_range_validation() {
        let material = MaterialProperties::new(
            "Test",
            MaterialType::Metal,
            MaterialSubtype::Custom,
        )
        .with_chip_load_range(0.002, 0.006);

        assert!(material.chip_load_min <= material.chip_load_max);
        assert!(material.chip_load_min >= 0.0);
        assert!(material.chip_load_max >= 0.0);
    }
}
