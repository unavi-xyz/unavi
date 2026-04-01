use crate::wired::scene::context::self_document;

use crate::check;

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
