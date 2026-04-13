use std::{cell::RefCell, fmt::Write};

use wired_prelude::{wired_math::types::Vec3, wired_scene::types::Color};
use wired_schemas::SCHEMA_BEACON;

use crate::{
    unavi::{
        shapes::api::{Cuboid, Cylinder, Torus},
        vui_module::api::{ModuleEvent, VuiModule},
    },
    wired::{
        scene::{
            context::{create_document, remove_document, self_document},
            types::{
                Collider, ColliderCylinder, Document, Material, Mesh, Node, PrimitiveTopology,
                RigidBodyKind,
            },
        },
        wds::{
            context::get_wds,
            types::{QueryFilter, QueryFuture},
        },
    },
};

wired_prelude::generate_script!(Script);

const NAME: &str = "Nav";

const BASE_H: f32 = 0.016;
const BASIN_HEIGHT: f32 = 0.18;
const BASIN_RADIUS: f32 = 0.52;
const BASIN_X: f32 = 0.58;
const BASIN_Y: f32 = -0.10;
const ICON_MINOR_R: f32 = 0.008;
const ICON_MAJOR_R: f32 = 0.02;
const LIP_H: f32 = 0.036;
const LIP_T: f32 = 0.012;
const LIP_Y: f32 = BASE_H * 0.5 + LIP_H * 0.5;
const RING_COLLIDER_HEIGHT: f32 = RING_THICKNESS * 2.0;
const RING_COLLIDER_RADIUS: f32 = RING_RADIUS + 0.06;
const RING_RADIUS: f32 = 0.56;
const RING_THICKNESS: f32 = 0.040;
const TABLE_D: f32 = 0.64;
const TABLE_W: f32 = 1.00;
const X_LIP_X: f32 = TABLE_W * 0.5 - LIP_T * 0.5;
const Z_LIP_Z: f32 = TABLE_D * 0.5 - LIP_T * 0.5;

struct Script {
    _icon_mesh: Mesh,
    _nodes: Vec<Node>,
    beacon_query: RefCell<Option<QueryFuture>>,
    beacons: RefCell<Vec<Document>>,
    color_mat: Material,
    module: VuiModule,
    ring: Node,
    root: Node,
}

impl GuestScript for Script {
    fn new() -> Self {
        let doc = self_document();
        let mut nodes = Vec::new();

        let color_mat = doc.create_material();
        color_mat.set_double_sided(true);

        let root = doc.create_node();
        root.set_scale(Vec3::ZERO);

        let filter_table = make_filter_table(&doc, &color_mat, &mut nodes);
        filter_table.set_translation(Vec3::new(BASIN_X, 0.0, 0.0));
        root.add_child(&filter_table);
        nodes.push(filter_table);

        let basin = make_basin(&doc, &mut nodes);
        basin.set_translation(Vec3::new(-BASIN_X, BASIN_Y, 0.0));
        root.add_child(&basin);
        nodes.push(basin);

        let ring_mat = doc.create_material();
        ring_mat.set_base_color(Color::WHITE);
        ring_mat.set_double_sided(true);

        let ring = doc.create_node();
        ring.set_mesh(Some(&Torus::new(RING_THICKNESS, RING_RADIUS).mesh()));
        ring.set_material(Some(&ring_mat));
        ring.set_collider(Some(&Collider::Cylinder(ColliderCylinder {
            height: RING_COLLIDER_HEIGHT,
            radius: RING_COLLIDER_RADIUS,
        })));
        ring.set_rigid_body(Some(RigidBodyKind::Dynamic));
        ring.set_scale(Vec3::ZERO);

        // Torus from unavi-shapes lies in XZ plane; rotate 90° around X: (x,y,z) → (x,-z,y)
        let src = Torus::new(ICON_MINOR_R, ICON_MAJOR_R).mesh();
        let rot_pos: Vec<f32> = src
            .positions()
            .unwrap_or_default()
            .chunks(3)
            .flat_map(|c| [c[0], -c[2], c[1]])
            .collect();
        let rot_nor: Vec<f32> = src
            .normals()
            .unwrap_or_default()
            .chunks(3)
            .flat_map(|c| [c[0], -c[2], c[1]])
            .collect();
        let icon_mesh = doc.create_mesh();
        icon_mesh.set_topology(PrimitiveTopology::TriangleList);
        icon_mesh.set_positions(Some(&rot_pos));
        icon_mesh.set_normals(Some(&rot_nor));
        icon_mesh.set_indices(src.indices().as_ref());
        let module = VuiModule::new(NAME, &icon_mesh);

        Self {
            _icon_mesh: icon_mesh,
            _nodes: nodes,
            beacon_query: RefCell::new(None),
            beacons: RefCell::default(),
            color_mat,
            module,
            ring,
            root,
        }
    }

    fn tick(&self) {
        while let Some(event) = self.module.poll() {
            match event {
                ModuleEvent::Activate(t) => {
                    self.root.set_translation(t.translation);
                    self.root.set_rotation(t.rotation);
                    self.root.set_scale(t.scale);
                    self.ring.set_translation(Vec3 {
                        x: t.translation.x - BASIN_X,
                        y: t.translation.y + 0.5,
                        z: t.translation.z,
                    });
                    // TODO fix ring position, grab, add phys joint
                    // self.ring.set_scale(Vec3::ONE);

                    let fut = get_wds().query(Some(&QueryFilter {
                        creator: None,
                        schemas: Some(vec![SCHEMA_BEACON.hash.as_bytes().to_vec()]),
                    }));
                    *self.beacon_query.borrow_mut() = Some(fut);
                }
                ModuleEvent::Deactivate => {
                    self.root.set_scale(Vec3::ZERO);
                    self.ring.set_scale(Vec3::ZERO);
                    *self.beacon_query.borrow_mut() = None;

                    for doc in self.beacons.borrow().as_slice() {
                        remove_document(&doc.id());
                    }
                }
                ModuleEvent::SetColor(color) => {
                    self.color_mat.set_base_color(color);
                }
            }
        }

        let mut remove_query = false;

        if let Some(fut) = self.beacon_query.borrow().as_ref()
            && let Some(result) = fut.poll()
        {
            match result {
                Ok(ids) => {
                    for id in ids {
                        let id_hex =
                            id.iter()
                                .fold(String::with_capacity(id.len() * 2), |mut acc, b| {
                                    write!(&mut acc, "{b:02x}").expect("write byte");
                                    acc
                                });
                        println!("found beacon: {id_hex}");

                        let Ok(doc) =
                            create_document().map_err(|err| eprintln!("create doc error: {err}"))
                        else {
                            continue;
                        };
                        let doc_id_hex = doc.id().iter().fold(
                            String::with_capacity(id.len() * 2),
                            |mut acc, b| {
                                write!(&mut acc, "{b:02x}").expect("write byte");
                                acc
                            },
                        );
                        println!("+ spawned doc: {doc_id_hex}");

                        let beacon_size = 0.15;

                        let cuboid = Cuboid::new(beacon_size, beacon_size, beacon_size);
                        cuboid.set_doc(doc.clone());

                        let node = doc.create_node();
                        node.set_mesh(Some(&cuboid.mesh()));

                        let mat = doc.create_material();
                        mat.set_roughness(0.4);
                        mat.set_metallic(0.7);
                        let color = generate_beacon_color(id);
                        mat.set_base_color(color);
                        node.set_material(Some(&mat));

                        node.set_collider(Some(&cuboid.collider()));
                        node.set_rigid_body(Some(RigidBodyKind::Dynamic));

                        let mut pos = self.root.translation();
                        pos.x += BASIN_X;
                        pos.y += BASIN_Y + (beacon_size * 2.0);
                        node.set_translation(pos);

                        node.set_name(Some(&format!("Beacon {id_hex}")));

                        self.beacons.borrow_mut().push(doc);
                    }

                    remove_query = true;
                }
                Err(()) => eprintln!("wds query error"),
            }
        }

        if remove_query {
            *self.beacon_query.borrow_mut() = None;
        }
    }

    fn render(&self) {}
    fn drop(&self) {}
}

fn generate_beacon_color(hash: Vec<u8>) -> Color {
    let bytes = hash.as_slice();

    // Use first 8 bytes for hue (full spectrum)
    let hue_u64 = u64::from_le_bytes(bytes[0..8].try_into().expect("u64"));
    let h = (hue_u64 as f64 / u64::MAX as f64) as f32; // 0..1

    // Next bytes for saturation/value but clamp to nice ranges
    let s_u16 = u16::from_le_bytes(bytes[8..10].try_into().expect("u16"));
    let v_u16 = u16::from_le_bytes(bytes[10..12].try_into().expect("u16"));

    // Keep colors vivid but not neon / washed out
    let s = (f32::from(s_u16) / f32::from(u16::MAX)).mul_add(0.35, 0.55); // 0.55–0.9
    let v = (f32::from(v_u16) / f32::from(u16::MAX)).mul_add(0.30, 0.65); // 0.65–0.95

    Color::hsv(h, s, v)
}

fn make_filter_table(doc: &Document, mat: &Material, nodes: &mut Vec<Node>) -> Node {
    let group = doc.create_node();

    let base = doc.create_node();
    let base_shape = Cuboid::new(TABLE_W, BASE_H, TABLE_D);
    base.set_collider(Some(&base_shape.collider()));
    base.set_rigid_body(Some(RigidBodyKind::Fixed));
    base.set_mesh(Some(&base_shape.mesh()));
    base.set_material(Some(mat));
    group.add_child(&base);
    nodes.push(base);

    let x_lip_shape = Cuboid::new(LIP_T, LIP_H, TABLE_D);
    for x_sign in [-1.0_f32, 1.0_f32] {
        let lip = doc.create_node();
        lip.set_collider(Some(&x_lip_shape.collider()));
        lip.set_rigid_body(Some(RigidBodyKind::Fixed));
        lip.set_mesh(Some(&x_lip_shape.mesh()));
        lip.set_material(Some(mat));
        lip.set_translation(Vec3::new(x_sign * X_LIP_X, LIP_Y, 0.0));
        group.add_child(&lip);
        nodes.push(lip);
    }

    let z_lip_shape = Cuboid::new(TABLE_W, LIP_H, LIP_T);
    for z_sign in [-1.0_f32, 1.0_f32] {
        let lip = doc.create_node();
        lip.set_collider(Some(&z_lip_shape.collider()));
        lip.set_rigid_body(Some(RigidBodyKind::Fixed));
        lip.set_mesh(Some(&z_lip_shape.mesh()));
        lip.set_material(Some(mat));
        lip.set_translation(Vec3::new(0.0, LIP_Y, z_sign * Z_LIP_Z));
        group.add_child(&lip);
        nodes.push(lip);
    }

    let divider = doc.create_node();
    let divider_shape = Cuboid::new(LIP_T, LIP_H, TABLE_D);
    divider.set_collider(Some(&divider_shape.collider()));
    divider.set_rigid_body(Some(RigidBodyKind::Fixed));
    divider.set_mesh(Some(&divider_shape.mesh()));
    divider.set_material(Some(mat));
    divider.set_translation(Vec3::new(0.0, LIP_Y, 0.0));
    group.add_child(&divider);
    nodes.push(divider);

    group
}

fn make_basin(doc: &Document, nodes: &mut Vec<Node>) -> Node {
    let group = doc.create_node();

    let mat = doc.create_material();
    mat.set_base_color(Color::rgb(0.88, 0.88, 0.92));
    mat.set_double_sided(true);

    let cylinder = Cylinder::new(BASIN_RADIUS, BASIN_HEIGHT);
    let dish = doc.create_node();
    dish.set_mesh(Some(&cylinder.mesh()));
    dish.set_material(Some(&mat));
    dish.set_collider(Some(&cylinder.collider()));
    dish.set_rigid_body(Some(RigidBodyKind::Fixed));
    group.add_child(&dish);
    nodes.push(dish);

    group
}
