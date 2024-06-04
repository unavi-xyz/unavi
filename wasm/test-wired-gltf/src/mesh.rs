use crate::{
    bindings::wired::gltf::mesh::{create_mesh, list_meshes, remove_mesh, Mesh},
    property_tests::{test_property, Property},
};

impl Property for Mesh {
    fn id(&self) -> u32 {
        self.id()
    }
}

pub fn test_mesh_api() {
    test_property(list_meshes, create_mesh, remove_mesh);
}
