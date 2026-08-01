use blake3::Hash;
use wired_prelude::prelude::*;
use wired_records::{
    beacon::BeaconRecord,
    value::RecordValue,
};
use wired_schemas::SCHEMA_BEACON;

use crate::{
    palette,
    unavi::shapes::api::Cuboid,
    wired::{
        scene::{
            api::{
                load_hsd,
                remove_document,
                self_document,
            },
            types::{
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
            },
        },
    },
};

const TABLE_W: f32 = 0.9;
const TABLE_D: f32 = 0.7;
const TABLE_H: f32 = 0.03;
const LIP_H: f32 = 0.05;
const LIP_T: f32 = 0.016;
const LIP_Y: f32 = TABLE_H * 0.5 + LIP_H * 0.5;
const COLUMNS: usize = 4;
const SPACING: f32 = 0.16;
/// Mirrors `unavi-beacon`'s own bounding size, so beacons rest on the surface.
const BEACON_SIZE: f32 = 0.095;

const IDENTITY: Quat = Quat {
    x: 0.0,
    y: 0.0,
    z: 0.0,
    w: 1.0,
};

const fn material(color: Color) -> Material {
    Material {
        alpha_cutoff:               None,
        alpha_mode:                 None,
        base_color:                 Some(color),
        base_color_texture:         None,
        double_sided:               Some(true),
        emissive:                   None,
        emissive_texture:           None,
        metallic:                   None,
        metallic_roughness_texture: None,
        normal_texture:             None,
        occlusion_texture:          None,
        roughness:                  None,
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

const fn hidden() -> Xform {
    Xform {
        translation: Vec3::ZERO,
        rotation:    IDENTITY,
        scale:       Vec3::ZERO,
    }
}

fn set_translation(prim: &Prim, translation: Vec3) {
    prim.set_xform(Some(Xform {
        translation,
        rotation: IDENTITY,
        scale: Vec3::splat(1.0),
    }))
    .ok();
}

/// Column index centers around the table's midline; row index zigzags
/// outward from the center row (0, 1, -1, 2, -2, ...) so the grid stays
/// balanced on the table rather than growing off to one side.
fn grid_offset(index: usize) -> Vec3 {
    let col = (index % COLUMNS) as f32;
    let row = i32::try_from(index / COLUMNS).unwrap_or(i32::MAX);
    let row_half = (row + 1) / 2;
    let signed_row = if row % 2 == 1 { row_half } else { -row_half };
    Vec3::new(
        ((COLUMNS - 1) as f32).mul_add(-0.5, col) * SPACING,
        TABLE_H.mul_add(0.5, BEACON_SIZE * 0.5),
        signed_row as f32 * SPACING,
    )
}

fn add_slab(parent: &Prim, size: Vec3, translation: Vec3, color: Color) {
    let shape = Cuboid::new(size);
    let slab = shape.mesh();
    slab.set_collider(Some(&shape.collider())).ok();
    slab.set_rigid_body(Some(static_body())).ok();
    slab.set_material(Some(&material(color))).ok();
    set_translation(&slab, translation);
    parent.add_child(&slab).ok();
}

fn parse_beacon_record(bytes: &[u8]) -> Option<BeaconRecord> {
    let value: RecordValue = postcard::from_bytes(bytes).ok()?;
    value.get("beacon")?.clone().into_typed().ok()
}

/// The gauntlet's nav table: on open, queries WDS for known beacons and lays
/// them out in a grid on a small table placed in front of the player.
pub struct Nav {
    root:         Prim,
    beacon_query: Option<QueryFuture>,
    beacon_reads: Vec<ReadFuture>,
    beacons:      Vec<Document>,
}

impl Nav {
    #[must_use]
    pub fn new() -> Self {
        let doc = self_document().expect("self_document");
        let root = doc.create_prim().expect("create_prim");
        root.set_xform(Some(hidden())).ok();

        add_slab(
            &root,
            Vec3::new(TABLE_W, TABLE_H, TABLE_D),
            Vec3::ZERO,
            palette::SURFACE,
        );

        let x_lip = Vec3::new(LIP_T, LIP_H, TABLE_D);
        for sign in [-1.0_f32, 1.0] {
            add_slab(
                &root,
                x_lip,
                Vec3::new(sign * LIP_T.mul_add(-0.5, TABLE_W * 0.5), LIP_Y, 0.0),
                palette::DIM,
            );
        }
        let z_lip = Vec3::new(TABLE_W, LIP_H, LIP_T);
        for sign in [-1.0_f32, 1.0] {
            add_slab(
                &root,
                z_lip,
                Vec3::new(0.0, LIP_Y, sign * LIP_T.mul_add(-0.5, TABLE_D * 0.5)),
                palette::DIM,
            );
        }

        Self {
            root,
            beacon_query: None,
            beacon_reads: Vec::new(),
            beacons: Vec::new(),
        }
    }

    pub fn open(&mut self, placement: Transform) -> anyhow::Result<()> {
        self.root.set_xform(Some(Xform {
            translation: placement.translation,
            rotation:    placement.rotation,
            scale:       Vec3::ONE,
        }))?;
        self.beacon_query = Some(get_wds()?.query(Some(&QueryFilter {
            creator: None,
            schemas: Some(vec![SCHEMA_BEACON.hash.as_bytes().to_vec()]),
        })));
        Ok(())
    }

    pub fn close(&mut self) -> anyhow::Result<()> {
        self.root.set_xform(Some(hidden()))?;
        self.beacon_query = None;
        self.beacon_reads.clear();
        for doc in self.beacons.drain(..) {
            remove_document(&doc.id())?;
        }
        Ok(())
    }

    pub fn fixed_update(&mut self) -> anyhow::Result<()> {
        if let Some(fut) = &self.beacon_query
            && let Some(result) = fut.poll()
        {
            self.beacon_query = None;
            match result {
                Ok(ids) => {
                    for id in ids {
                        let id = Hash::from_slice(&id).expect("valid hash");
                        self.beacon_reads.push(get_wds()?.read(id.as_slice()));
                    }
                }
                Err(()) => eprintln!("nav: WDS beacon query error"),
            }
        }

        for i in (0..self.beacon_reads.len()).rev() {
            let Some(res) = self.beacon_reads[i].poll() else {
                continue;
            };
            self.beacon_reads.remove(i);

            let Ok(bytes) = res else {
                continue;
            };
            let Some(beacon) = parse_beacon_record(&bytes) else {
                continue;
            };
            let space = Hash::from_bytes(*beacon.space.as_bytes());

            let doc = self_document()?;
            let Some(beacon_asset) = doc.prims().into_iter().find_map(|p| p.asset()) else {
                eprintln!("nav: gauntlet HSD missing beacon asset child prim");
                continue;
            };
            let Ok(beacon_doc) = load_hsd(&beacon_asset) else {
                eprintln!("nav: failed to load beacon doc");
                continue;
            };

            let prim = beacon_doc.create_prim()?;
            prim.set_name(Some(&space.to_string()))?;

            let root_xform = self.root.xform().unwrap_or(Xform {
                translation: Vec3::ZERO,
                rotation:    IDENTITY,
                scale:       Vec3::ONE,
            });
            let offset = grid_offset(self.beacons.len());
            let pos = root_xform.translation + root_xform.rotation * offset;
            set_translation(&prim, pos);

            self.beacons.push(beacon_doc);
        }
        Ok(())
    }
}

impl Default for Nav {
    fn default() -> Self {
        Self::new()
    }
}
