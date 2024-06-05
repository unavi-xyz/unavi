use crate::{
    bindings::wired::{
        gltf::mesh::{create_mesh, list_meshes, remove_mesh, Mesh, Primitive},
        log::api::{log, LogLevel},
    },
    property_tests::{test_property, Property},
};

impl Property for Mesh {
    fn id(&self) -> u32 {
        self.id()
    }
}

impl Property for Primitive {
    fn id(&self) -> u32 {
        self.id()
    }
}

pub fn test_mesh_api() {
    log(LogLevel::Debug, "testing mesh");
    test_property(list_meshes, create_mesh, remove_mesh);

    log(LogLevel::Debug, "testing primitive");
    let mesh = create_mesh();
    let list = || mesh.list_primitives();
    let create = || mesh.create_primitive();
    let remove = |v| mesh.remove_primitive(v);
    test_property(list, create, remove);
}
