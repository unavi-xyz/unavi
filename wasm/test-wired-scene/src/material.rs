use crate::{
    bindings::wired::{
        log::api::{log, LogLevel},
        scene::{gltf::Gltf, material::Material},
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

    let document = Gltf::new();

    test_property(
        Material::new,
        |v| document.add_material(v),
        || document.list_materials(),
        |v| document.remove_material(v),
    );
}
