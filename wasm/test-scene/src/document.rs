use crate::wired::scene::context::{create_document, remove_document, self_document};
use wired_prelude::wired_math::types::Vec3;

use crate::check;

pub fn test_create_document() {
    let doc = match create_document() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("FAIL create_document: {e}");
            return;
        }
    };

    check("created doc id length", doc.id().len(), 32);

    let n = doc.create_node();
    let m = doc.create_mesh();
    let _mat = doc.create_material();
    check("created doc nodes", doc.nodes().len(), 1);
    check("created doc meshes", doc.meshes().len(), 1);
    check("created doc materials", doc.materials().len(), 1);

    n.set_translation(Vec3::new(1.0, 2.0, 3.0));
    m.set_positions(Some(&[0.0_f32, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0]));
    m.set_normals(Some(&[0.0_f32, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0]));

    doc.remove_node(&n);
    check("nodes after remove", doc.nodes().len(), 0);

    remove_document(&doc.id());
}

pub fn test_document() {
    let doc = self_document();

    check("doc id length", doc.id().len(), 32);

    let n1 = doc.create_node();
    let n2 = doc.create_node();
    check("nodes count after create", doc.nodes().len(), 2);

    let m1 = doc.create_mesh();
    check("meshes count after create", doc.meshes().len(), 1);

    let mat1 = doc.create_material();
    check("materials count after create", doc.materials().len(), 1);

    n1.add_child(&n2);
    check("roots excludes children", doc.roots().len(), 1);
    n1.remove_child(&n2);

    doc.remove_node(&n2);
    check("nodes count after remove", doc.nodes().len(), 1);
    doc.remove_mesh(&m1);
    check("meshes count after remove", doc.meshes().len(), 0);
    doc.remove_material(&mat1);
    check("materials count after remove", doc.materials().len(), 0);
}
