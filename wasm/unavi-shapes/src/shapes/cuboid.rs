use std::cell::RefCell;

use crate::bindings::{
    exports::unavi::shapes::api::{GuestCuboid, Vec3},
    wired::{
        physics::types::{Collider, Shape},
        scene::{
            mesh::{Mesh, Primitive},
            node::Node,
        },
    },
};

pub struct Cuboid {
    size: RefCell<Vec3>,
}

impl GuestCuboid for Cuboid {
    fn new(size: Vec3) -> Self {
        Self {
            size: RefCell::new(size),
        }
    }

    fn size(&self) -> Vec3 {
        *self.size.borrow()
    }
    fn set_size(&self, value: Vec3) {
        self.size.replace(value);
    }

    fn to_mesh(&self) -> Mesh {
        let mesh = Mesh::new();
        let primitive = mesh.create_primitive();

        make_cuboid(&self.size.borrow(), &primitive);

        mesh
    }
    fn to_node(&self) -> Node {
        let node = Node::new();
        node.set_mesh(Some(&self.to_mesh()));
        node
    }
    fn to_physics_node(&self) -> Node {
        let node = self.to_node();
        node.set_collider(Some(&Collider::new(Shape::Cuboid(self.size()))));
        node
    }
}

pub fn make_cuboid(size: &Vec3, primitive: &Primitive) {
    let indices = [
        0, 1, 2, 2, 3, 0, 4, 5, 6, 6, 7, 4, 8, 9, 10, 10, 11, 8, 12, 13, 14, 14, 15, 12, 16, 17,
        18, 18, 19, 16, 20, 21, 22, 22, 23, 20,
    ];

    let positions = [
        [-0.5, -0.5, 0.5],
        [0.5, -0.5, 0.5],
        [0.5, 0.5, 0.5],
        [-0.5, 0.5, 0.5],
        [-0.5, 0.5, -0.5],
        [0.5, 0.5, -0.5],
        [0.5, -0.5, -0.5],
        [-0.5, -0.5, -0.5],
        [0.5, -0.5, -0.5],
        [0.5, 0.5, -0.5],
        [0.5, 0.5, 0.5],
        [0.5, -0.5, 0.5],
        [-0.5, -0.5, 0.5],
        [-0.5, 0.5, 0.5],
        [-0.5, 0.5, -0.5],
        [-0.5, -0.5, -0.5],
        [0.5, 0.5, -0.5],
        [-0.5, 0.5, -0.5],
        [-0.5, 0.5, 0.5],
        [0.5, 0.5, 0.5],
        [0.5, -0.5, 0.5],
        [-0.5, -0.5, 0.5],
        [-0.5, -0.5, -0.5],
        [0.5, -0.5, -0.5],
    ]
    .into_iter()
    .flat_map(|v| [v[0] * size.x, v[1] * size.y, v[2] * size.z])
    .collect::<Vec<_>>();

    let normals = [
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, -1.0],
        [0.0, 0.0, -1.0],
        [0.0, 0.0, -1.0],
        [0.0, 0.0, -1.0],
        [1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [-1.0, 0.0, 0.0],
        [-1.0, 0.0, 0.0],
        [-1.0, 0.0, 0.0],
        [-1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, -1.0, 0.0],
        [0.0, -1.0, 0.0],
        [0.0, -1.0, 0.0],
        [0.0, -1.0, 0.0],
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();

    let uvs = [
        [0.0, 0.0],
        [1.0, 0.0],
        [1.0, 1.0],
        [0.0, 1.0],
        [1.0, 0.0],
        [0.0, 0.0],
        [0.0, 1.0],
        [1.0, 1.0],
        [0.0, 0.0],
        [1.0, 0.0],
        [1.0, 1.0],
        [0.0, 1.0],
        [1.0, 0.0],
        [0.0, 0.0],
        [0.0, 1.0],
        [1.0, 1.0],
        [1.0, 0.0],
        [0.0, 0.0],
        [0.0, 1.0],
        [1.0, 1.0],
        [0.0, 0.0],
        [1.0, 0.0],
        [1.0, 1.0],
        [0.0, 1.0],
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();

    primitive.set_indices(&indices);
    primitive.set_normals(&normals);
    primitive.set_positions(&positions);
    primitive.set_uvs(&uvs);
}
