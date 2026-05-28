use loro::LoroDoc;
use wired_prelude::prelude::*;
use wired_records::beacon::BeaconRecord;
use wired_schemas::SCHEMA_BEACON;

use crate::{
    unavi::{
        shapes::api::{
            Cuboid,
            Cylinder,
            Torus,
        },
        vui_module::api::{
            ModuleEvent,
            VuiModule,
        },
    },
    wired::{
        scene::{
            api::{
                load_hsd,
                remove_document,
                self_document,
            },
            types::{
                Collider,
                ColliderCylinder,
                Document,
                Material,
                Prim,
                RigidBody,
                RigidBodyKind,
                Xform,
            },
        },
        wds::{
            api::get_wds,
            types::{
                QueryFilter,
                QueryFuture,
                ReadFuture,
                WdsRecord,
            },
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

const IDENTITY_QUAT: Quat = Quat {
    x: 0.0,
    y: 0.0,
    z: 0.0,
    w: 1.0,
};

const fn xform_translation(translation: Vec3) -> Xform {
    Xform {
        translation,
        rotation: IDENTITY_QUAT,
        scale: Vec3::splat(1.0),
    }
}

fn set_translation(prim: &Prim, translation: Vec3) {
    prim.set_xform(Some(xform_translation(translation)));
}

fn set_scale(prim: &Prim, scale: Vec3) {
    prim.set_xform(Some(Xform {
        translation: Vec3::splat(0.0),
        rotation: IDENTITY_QUAT,
        scale,
    }));
}

const fn material(base_color: Option<Color>, double_sided: bool) -> Material {
    Material {
        alpha_cutoff: None,
        alpha_mode: None,
        base_color,
        base_color_texture: None,
        double_sided: Some(double_sided),
        emissive: None,
        emissive_texture: None,
        metallic: None,
        metallic_roughness_texture: None,
        normal_texture: None,
        occlusion_texture: None,
        roughness: None,
    }
}

const fn static_body() -> RigidBody {
    RigidBody {
        kind:            RigidBodyKind::Static,
        angular_damping: None,
        friction:        None,
        linear_damping:  None,
        mass:            None,
        restitution:     None,
    }
}

const fn dynamic_body() -> RigidBody {
    RigidBody {
        kind:            RigidBodyKind::Dynamic,
        angular_damping: None,
        friction:        None,
        linear_damping:  None,
        mass:            None,
        restitution:     None,
    }
}

struct Script {
    _icon:        Prim,
    _prims:       Vec<Prim>,
    beacon_query: Option<QueryFuture>,
    beacon_reads: Vec<ReadFuture>,
    beacons:      Vec<Document>,
    color:        Color,
    module:       VuiModule,
    ring:         Prim,
    root:         Prim,
    themed_prims: Vec<Prim>,
}

impl ScriptBehavior for Script {
    fn init() -> Self {
        let doc = self_document();
        let mut prims = Vec::new();
        let mut themed_prims = Vec::new();

        let color = Color::WHITE;

        let root = doc.create_prim();
        set_scale(&root, Vec3::splat(0.0));

        let filter_table = make_filter_table(&doc, color, &mut prims, &mut themed_prims);
        set_translation(&filter_table, Vec3::new(BASIN_X, 0.0, 0.0));
        root.add_child(&filter_table);
        prims.push(filter_table);

        let basin = make_basin(&doc, color, &mut prims, &mut themed_prims);
        set_translation(&basin, Vec3::new(-BASIN_X, BASIN_Y, 0.0));
        root.add_child(&basin);
        prims.push(basin);

        let ring = Torus::new(RING_THICKNESS, RING_RADIUS).mesh();
        ring.set_material(Some(&material(Some(color), true)));
        ring.set_collider(Some(&Collider::Cylinder(ColliderCylinder {
            height: RING_COLLIDER_HEIGHT,
            radius: RING_COLLIDER_RADIUS,
        })));
        ring.set_rigid_body(Some(dynamic_body()));
        set_scale(&ring, Vec3::splat(0.0));
        themed_prims.push(ring.clone());

        let icon = Torus::new(ICON_MINOR_R, ICON_MAJOR_R).mesh();
        let module = VuiModule::new(NAME, &icon);

        Self {
            _icon: icon,
            _prims: prims,
            beacon_query: None,
            beacon_reads: Vec::new(),
            beacons: Vec::new(),
            color,
            module,
            ring,
            root,
            themed_prims,
        }
    }

    fn tick(&mut self) {
        while let Some(event) = self.module.poll() {
            match event {
                ModuleEvent::Activate(t) => {
                    self.root.set_xform(Some(Xform {
                        translation: t.translation,
                        rotation:    t.rotation,
                        scale:       t.scale,
                    }));
                    set_translation(
                        &self.ring,
                        Vec3 {
                            x: t.translation.x - BASIN_X,
                            y: BASIN_HEIGHT.mul_add(-0.5, t.translation.y + BASIN_Y)
                                - RING_THICKNESS,
                            z: t.translation.z,
                        },
                    );
                    // TODO grab, add phys joint

                    self.beacon_query = Some(get_wds().query(Some(&QueryFilter {
                        creator: None,
                        schemas: Some(vec![SCHEMA_BEACON.hash.as_bytes().to_vec()]),
                    })));
                }
                ModuleEvent::Deactivate => {
                    set_scale(&self.root, Vec3::splat(0.0));
                    set_scale(&self.ring, Vec3::splat(0.0));
                    self.beacon_query = None;

                    for doc in self.beacons.drain(..) {
                        remove_document(&doc.id());
                    }
                }
                ModuleEvent::SetColor(color) => {
                    self.color = color;
                    let mat = material(Some(color), true);
                    for prim in &self.themed_prims {
                        prim.set_material(Some(&mat));
                    }
                }
            }
        }

        if let Some(fut) = &self.beacon_query
            && let Some(result) = fut.poll()
        {
            self.beacon_query = None;
            match result {
                Ok(ids) => {
                    for id in ids {
                        let id = blake3::Hash::from_slice(&id).expect("valid hash");
                        println!("Reading beacon: id={id}");
                        let read_fut = get_wds().read(id.as_slice());
                        self.beacon_reads.push(read_fut);
                    }
                }
                Err(()) => eprintln!("WDS query error"),
            }
        }

        for (i, fut) in self.beacon_reads.iter().enumerate() {
            if let Some(res) = fut.poll() {
                self.beacon_reads.remove(i);

                if let Ok(record) = res
                    && let Some(beacon) = parse_beacon_record(&record)
                {
                    let space = blake3::Hash::from_bytes(*beacon.space.as_bytes());
                    println!("Found beacon: space={space}");

                    let doc = self_document();

                    let Some(beacon_asset) = doc.prims().into_iter().find_map(|p| p.asset()) else {
                        eprintln!("Nav HSD missing beacon asset child prim");
                        break;
                    };
                    let Ok(beacon_doc) = load_hsd(&beacon_asset) else {
                        eprintln!("Failed to load beacon doc");
                        break;
                    };

                    let prim = beacon_doc.create_prim();
                    prim.set_name(Some(&space.to_string()));

                    let mut pos = self
                        .root
                        .xform()
                        .map_or(Vec3::splat(0.0), |x| x.translation);
                    pos.x -= BASIN_X;
                    pos.y += BASIN_Y + 1.0;
                    set_translation(&prim, pos);

                    self.beacons.push(beacon_doc);
                }

                break;
            }
        }
    }
}

fn parse_beacon_record(record: &WdsRecord) -> Option<BeaconRecord> {
    let (_, bytes) = record.containers.iter().find(|(k, _)| k == "data")?;
    let doc = LoroDoc::new();
    doc.import(bytes).ok()?;
    BeaconRecord::load(&doc).ok()
}

fn make_filter_table(
    doc: &Document,
    color: Color,
    prims: &mut Vec<Prim>,
    themed: &mut Vec<Prim>,
) -> Prim {
    let group = doc.create_prim();
    let mat = material(Some(color), true);

    let base_shape = Cuboid::new(Vec3::new(TABLE_W, BASE_H, TABLE_D));
    let base = base_shape.mesh();
    base.set_collider(Some(&base_shape.collider()));
    base.set_rigid_body(Some(static_body()));
    base.set_material(Some(&mat));
    themed.push(base.clone());
    group.add_child(&base);
    prims.push(base);

    let x_lip_shape = Cuboid::new(Vec3::new(LIP_T, LIP_H, TABLE_D));
    for x_sign in [-1.0_f32, 1.0_f32] {
        let lip = x_lip_shape.mesh();
        lip.set_collider(Some(&x_lip_shape.collider()));
        lip.set_rigid_body(Some(static_body()));
        lip.set_material(Some(&mat));
        themed.push(lip.clone());
        set_translation(&lip, Vec3::new(x_sign * X_LIP_X, LIP_Y, 0.0));
        group.add_child(&lip);
        prims.push(lip);
    }

    let z_lip_shape = Cuboid::new(Vec3::new(TABLE_W, LIP_H, LIP_T));
    for z_sign in [-1.0_f32, 1.0_f32] {
        let lip = z_lip_shape.mesh();
        lip.set_collider(Some(&z_lip_shape.collider()));
        lip.set_rigid_body(Some(static_body()));
        lip.set_material(Some(&mat));
        themed.push(lip.clone());
        set_translation(&lip, Vec3::new(0.0, LIP_Y, z_sign * Z_LIP_Z));
        group.add_child(&lip);
        prims.push(lip);
    }

    let divider_shape = Cuboid::new(Vec3::new(LIP_T, LIP_H, TABLE_D));
    let divider = divider_shape.mesh();
    divider.set_collider(Some(&divider_shape.collider()));
    divider.set_rigid_body(Some(static_body()));
    divider.set_material(Some(&mat));
    themed.push(divider.clone());
    set_translation(&divider, Vec3::new(0.0, LIP_Y, 0.0));
    group.add_child(&divider);
    prims.push(divider);

    group
}

fn make_basin(doc: &Document, color: Color, prims: &mut Vec<Prim>, themed: &mut Vec<Prim>) -> Prim {
    let group = doc.create_prim();
    let mat = material(Some(color), true);

    let cylinder = Cylinder::new(BASIN_RADIUS, BASIN_HEIGHT);
    let dish = cylinder.mesh();
    dish.set_material(Some(&mat));
    dish.set_collider(Some(&cylinder.collider()));
    dish.set_rigid_body(Some(static_body()));
    themed.push(dish.clone());
    group.add_child(&dish);
    prims.push(dish);

    group
}
