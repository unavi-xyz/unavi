use crate::{
    bindings::wired::gltf::node::{create_node, list_nodes, remove_node},
    panic_log,
};

pub fn test_nodes() {
    let node = create_node();
    let found_nodes = list_nodes();

    if found_nodes.len() != 1 {
        let err = format!("found list_nodes len: {}, expected 1", found_nodes.len());
        panic_log(&err);
    }

    if found_nodes[0].id() != node.id() {
        let err = format!(
            "found node id: {}, expected: {}",
            found_nodes[0].id(),
            node.id()
        );
        panic_log(&err);
    };

    remove_node(node);
    let found_nodes = list_nodes();

    if found_nodes.len() != 0 {
        let err = format!("found list_nodes len: {}, expected 0", found_nodes.len());
        panic_log(&err);
    }
}
