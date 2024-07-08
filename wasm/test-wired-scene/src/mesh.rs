use crate::{
    bindings::wired::{
        log::api::{log, LogLevel},
        scene::{
            gltf::Gltf,
            material::Material,
            mesh::{Mesh, Primitive},
        },
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

    let document = Gltf::new();

    test_property(
        Mesh::new,
        |v| document.add_mesh(v),
        || document.list_meshes(),
        |v| document.remove_mesh(&v),
    );

    log(LogLevel::Debug, "testing primitive");
    let mesh = Mesh::new();
    test_property(
        || mesh.create_primitive(),
        |_| {},
        || mesh.list_primitives(),
        |v| mesh.remove_primitive(v),
    );

    let primitive = mesh.create_primitive();
    primitive.set_indices(&[0, 1, 2]);
    primitive.set_normals(&[0.0, 1.0, 2.0]);
    primitive.set_positions(&[0.0, 1.0, 2.0]);
    primitive.set_uvs(&[0.0, 1.0]);

    let material = Material::new();
    primitive.set_material(Some(&material));

    // let found_material = primitive.material();
    // if found_material != material {}
}
