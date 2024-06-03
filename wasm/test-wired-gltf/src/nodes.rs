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

    let node = create_node();
    let node_2 = create_node();

    node.add_child(&node_2);

    let children = node.children();
    if !children.contains(&node_2) {
        let err = format!(
            "node children: {:?} does not contain node: {}",
            children.into_iter().map(|c| c.id()).collect::<Vec<_>>(),
            node_2.id()
        );
        panic_log(&err);
    }

    let parent = node_2.parent().expect("parent not found");
    if parent != node {
        let err = format!("found parent {:?}, expected {:?}", parent, node);
        panic_log(&err);
    }

    node.remove_child(&node_2);

    let children = node.children();
    if !children.is_empty() {
        let err = format!(
            "node children not empty: {:?}",
            children.into_iter().map(|c| c.id()).collect::<Vec<_>>()
        );
        panic_log(&err);
    }

    // TODO: wasm_component_layer panics when returning None https://github.com/DouglasDwyer/wasm_component_layer/issues/16
    // let parent = node_2.parent();
    // if parent.is_some() {
    //     let err = format!("parent is Some: {:?}", parent);
    //     panic_log(&err);
    // }

    let found_nodes = list_nodes();
    if found_nodes.len() != 2 {
        let err = format!("found list_nodes len: {}, expected 2", found_nodes.len());
        panic_log(&err);
    }
}
