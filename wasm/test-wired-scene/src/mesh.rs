use crate::bindings::wired::{
    log::api::{log, LogLevel},
    scene::{material::Material, mesh::Mesh},
};

pub fn test_mesh_api() {
    log(LogLevel::Debug, "testing mesh");

    let mesh = Mesh::new();

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
