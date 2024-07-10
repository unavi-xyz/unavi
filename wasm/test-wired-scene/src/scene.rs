use crate::{
    bindings::wired::{
        log::api::{log, LogLevel},
        scene::{gltf::Gltf, scene::Scene},
    },
    property_tests::{test_property, Property},
};

impl Property for Scene {
    fn id(&self) -> u32 {
        self.id()
    }
}

pub fn test_scene_api() {
    log(LogLevel::Debug, "testing scene");

    let document = Gltf::new();

    test_property(
        Scene::new,
        |v| document.add_scene(v),
        || document.list_scenes(),
        |v| document.remove_scene(&v),
    );
}
