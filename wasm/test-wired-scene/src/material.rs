use crate::{
    bindings::wired::{
        log::api::{log, LogLevel},
        scene::material::{create_material, list_materials, remove_material, Material},
    },
    property_tests::{test_property, Property},
};

impl Property for Material {
    fn id(&self) -> u32 {
        self.id()
    }
}

pub fn test_material_api() {
    log(LogLevel::Debug, "testing material");
    test_property(list_materials, create_material, remove_material);
}
