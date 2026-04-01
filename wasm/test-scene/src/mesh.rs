use crate::wired::scene::{context::self_document, types::{Indices, PrimitiveTopology}};

use crate::check;

pub fn test_mesh() {
    let doc  = self_document();
    let mesh = doc.create_mesh();

    check("mesh id non-empty", !mesh.id().is_empty(), true);

    mesh.set_name(Some("test-mesh"));
    check("mesh name", mesh.name().as_deref(), Some("test-mesh"));

    mesh.set_topology(PrimitiveTopology::TriangleList);
    check("mesh topology", mesh.topology(), PrimitiveTopology::TriangleList);

    mesh.set_positions(Some(&[0.0, 0.0, 0.0]));
    check("mesh positions", mesh.positions(), Some(vec![0.0_f32, 0.0, 0.0]));

    mesh.set_normals(Some(&[0.0, 1.0, 0.0]));
    check("mesh normals", mesh.normals(), Some(vec![0.0_f32, 1.0, 0.0]));

    mesh.set_indices(Some(&Indices::Full(vec![0])));
    check("mesh indices is some", mesh.indices().is_some(), true);

    mesh.set_colors(Some(&[1.0, 1.0, 1.0, 1.0]));
    check("mesh colors", mesh.colors(), Some(vec![1.0_f32, 1.0, 1.0, 1.0]));

    mesh.set_uv0(Some(&[0.0, 0.0]));
    check("mesh uv0", mesh.uv0(), Some(vec![0.0_f32, 0.0]));

    mesh.set_uv1(Some(&[0.5, 0.5]));
    check("mesh uv1", mesh.uv1(), Some(vec![0.5_f32, 0.5]));
}
