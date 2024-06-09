use crate::{
    bindings::wired::{
        gltf::{
            mesh::{create_mesh, list_meshes, remove_mesh, Mesh, Primitive},
            node::create_node,
        },
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

    let mesh = create_mesh();

    let node = create_node();
    node.set_mesh(Some(&mesh));
    let found_mesh = node.mesh().unwrap();
    assert_eq!(found_mesh, mesh);

    log(LogLevel::Debug, "testing primitive");
    let list = || mesh.list_primitives();
    let create = || mesh.create_primitive();
    let remove = |v| mesh.remove_primitive(v);
    test_property(list, create, remove);

    let primitive = mesh.create_primitive();
    primitive.set_indices(&[0, 1, 2]);
    primitive.set_normals(&[0.0, 1.0, 2.0]);
    primitive.set_positions(&[0.0, 1.0, 2.0]);
    primitive.set_uvs(&[0.0, 1.0, 2.0]);
}
