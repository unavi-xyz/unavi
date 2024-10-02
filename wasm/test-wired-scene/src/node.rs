use crate::{
    bindings::wired::{
        log::api::{log, LogLevel},
        scene::{
            mesh::Mesh,
            node::{Node, Transform},
        },
    },
    panic_log,
};

pub fn test_node_api() {
    log(LogLevel::Debug, "testing node");

    let node = Node::new();
    let node_2 = Node::new();

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

    // TODO: Can't return none<resource>?
    // let parent = node_2.parent();
    // if parent.is_some() {
    //     let err = format!("parent is Some: {:?}", parent);
    //     panic_log(&err);
    // }

    let transform = node.transform();
    if transform != Transform::default() {
        let err = format!(
            "transform {:?} does not match expected {:?}",
            transform,
            Transform::default()
        );
        panic_log(&err);
    }

    let mesh = Mesh::new();
    let node = Node::new();
    node.set_mesh(Some(&mesh));

    let found_mesh = node.mesh();
    if found_mesh.is_none() {
        let err = "found mesh is none".to_string();
        panic_log(&err);
    }

    let found_mesh = found_mesh.unwrap();
    if found_mesh != mesh {
        let err = format!(
            "found mesh {} does not match expected {}",
            found_mesh.id(),
            mesh.id()
        );
        panic_log(&err);
    }
}
