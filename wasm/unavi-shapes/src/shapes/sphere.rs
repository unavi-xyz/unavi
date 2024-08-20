use std::{
    cell::{Cell, RefCell},
    f32::consts::PI,
};

use hexasphere::shapes::IcoSphere;

use crate::bindings::{
    exports::unavi::shapes::api::{
        GuestSphere, Sphere as ExportSphere, SphereIco, SphereKind, SphereUv,
    },
    wired::{
        log::api::{log, LogLevel},
        physics::types::{Collider, Shape},
        scene::{mesh::Mesh, node::Node},
    },
};

const MAX_SUBDIVISIONS: u8 = 6;

pub struct Sphere {
    kind: RefCell<SphereKind>,
    radius: Cell<f32>,
}

impl GuestSphere for Sphere {
    fn new_ico(radius: f32) -> ExportSphere {
        ExportSphere::new(Sphere {
            kind: RefCell::new(SphereKind::Ico(SphereIco { subdivisions: 3 })),
            radius: Cell::new(radius),
        })
    }
    fn new_uv(radius: f32) -> ExportSphere {
        ExportSphere::new(Sphere {
            kind: RefCell::new(SphereKind::Uv(SphereUv {
                sectors: 32,
                stacks: 18,
            })),
            radius: Cell::new(radius),
        })
    }

    fn kind(&self) -> SphereKind {
        *self.kind.borrow()
    }
    fn set_kind(&self, value: SphereKind) {
        self.kind.replace(value);
    }

    fn radius(&self) -> f32 {
        self.radius.get()
    }
    fn set_radius(&self, value: f32) {
        self.radius.set(value);
    }

    fn to_mesh(&self) -> Mesh {
        let radius = self.radius();
        match self.kind() {
            SphereKind::Ico(SphereIco { subdivisions }) => {
                if subdivisions > MAX_SUBDIVISIONS {
                    log(
                        LogLevel::Warn,
                        &format!(
                            "Too many ico sphere subdivisions ({})! Limiting to {}",
                            subdivisions, MAX_SUBDIVISIONS
                        ),
                    );
                }

                let subdivisions = subdivisions.min(MAX_SUBDIVISIONS);
                create_ico_mesh(radius, subdivisions as usize)
            }
            SphereKind::Uv(SphereUv { sectors, stacks }) => {
                create_uv_mesh(radius, sectors as usize, stacks as usize)
            }
        }
    }
    fn to_node(&self) -> crate::bindings::exports::unavi::shapes::api::Node {
        let node = Node::new();
        node.set_mesh(Some(&self.to_mesh()));
        node
    }
    fn to_physics_node(&self) -> Node {
        let node = self.to_node();
        node.set_collider(Some(&Collider::new(Shape::Sphere(self.radius()))));
        node
    }
}

fn create_ico_mesh(radius: f32, subdivisions: usize) -> Mesh {
    let generated = IcoSphere::new(subdivisions, |point| {
        let inclination = point.y.acos();
        let azimuth = point.z.atan2(point.x);

        let norm_inclination = inclination / std::f32::consts::PI;
        let norm_azimuth = 0.5 - (azimuth / std::f32::consts::TAU);

        [norm_azimuth, norm_inclination]
    });

    let raw_points = generated.raw_points();

    let vertices = raw_points
        .iter()
        .map(|&p| (p * radius).into())
        .collect::<Vec<[f32; 3]>>();

    let normals = raw_points
        .iter()
        .copied()
        .map(Into::into)
        .collect::<Vec<[f32; 3]>>();

    let uvs = generated.raw_data().to_owned();

    let mut indices = Vec::with_capacity(generated.indices_per_main_triangle() * 20);

    for i in 0..20 {
        generated.get_indices(i, &mut indices);
    }

    let mesh = Mesh::new();
    let primitive = mesh.create_primitive();

    let normals = normals.into_iter().flatten().collect::<Vec<_>>();
    let vertices = vertices.into_iter().flatten().collect::<Vec<_>>();
    let uvs = uvs.into_iter().flatten().collect::<Vec<_>>();

    primitive.set_indices(&indices);
    primitive.set_normals(&normals);
    primitive.set_positions(&vertices);
    primitive.set_uvs(&uvs);

    mesh
}

fn create_uv_mesh(radius: f32, sectors: usize, stacks: usize) -> Mesh {
    let sectors_f32 = sectors as f32;
    let stacks_f32 = stacks as f32;
    let length_inv = 1. / radius;
    let sector_step = 2. * PI / sectors_f32;
    let stack_step = PI / stacks_f32;

    let mut vertices: Vec<[f32; 3]> = Vec::with_capacity(stacks * sectors);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(stacks * sectors);
    let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(stacks * sectors);
    let mut indices: Vec<u32> = Vec::with_capacity(stacks * sectors * 2 * 3);

    for i in 0..stacks + 1 {
        let stack_angle = PI / 2. - (i as f32) * stack_step;
        let xy = radius * stack_angle.cos();
        let z = radius * stack_angle.sin();

        for j in 0..sectors + 1 {
            let sector_angle = (j as f32) * sector_step;
            let x = xy * sector_angle.cos();
            let y = xy * sector_angle.sin();

            vertices.push([x, y, z]);
            normals.push([x * length_inv, y * length_inv, z * length_inv]);
            uvs.push([(j as f32) / sectors_f32, (i as f32) / stacks_f32]);
        }
    }

    // indices
    //  k1--k1+1
    //  |  / |
    //  | /  |
    //  k2--k2+1
    for i in 0..stacks {
        let mut k1 = i * (sectors + 1);
        let mut k2 = k1 + sectors + 1;
        for _j in 0..sectors {
            if i != 0 {
                indices.push(k1 as u32);
                indices.push(k2 as u32);
                indices.push((k1 + 1) as u32);
            }
            if i != stacks - 1 {
                indices.push((k1 + 1) as u32);
                indices.push(k2 as u32);
                indices.push((k2 + 1) as u32);
            }
            k1 += 1;
            k2 += 1;
        }
    }

    let normals = normals.into_iter().flatten().collect::<Vec<_>>();
    let vertices = vertices.into_iter().flatten().collect::<Vec<_>>();
    let uvs = uvs.into_iter().flatten().collect::<Vec<_>>();

    let mesh = Mesh::new();
    let primitive = mesh.create_primitive();

    primitive.set_indices(&indices);
    primitive.set_normals(&normals);
    primitive.set_positions(&vertices);
    primitive.set_uvs(&uvs);

    mesh
}
