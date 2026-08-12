use wired_prelude::prelude::*;

use crate::{
    palette,
    unavi::shapes::api::Cuboid,
    wired::{
        scene::{
            api::{
                create_document_from_prefab,
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
            types::ListFuture,
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
        alpha_cutoff: None,
        alpha_mode:   None,
        base_color:   Some(color),
        double_sided: Some(true),
        emissive:     None,
        metallic:     None,
        roughness:    None,
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

/// Columns center on the table's midline; rows zigzag outward from the
/// center row so the grid stays balanced.
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
    slab.set_collider(Some(shape.collider())).ok();
    slab.set_rigid_body(Some(static_body())).ok();
    slab.set_material(Some(material(color))).ok();
    set_translation(&slab, translation);
    parent.add_child(&slab).ok();
}

/// Prefix of a registry's active-spaces view; listing it picks out activity
/// and ignores the rest.
const ACTIVE_PREFIX: &str = "active/";

/// Extracts the space hex from an `active/<rank>/<space-hex>` key; keys
/// arrive in rank order, preserving the registry's ordering.
fn active_space_hex(key: &str) -> Option<&str> {
    key.strip_prefix(ACTIVE_PREFIX)?
        .split('/')
        .nth(1)
        .filter(|s| !s.is_empty())
}

/// The authored prim whose `prefab` slot every beacon copies, kept at zero
/// scale so the template itself never shows.
const TEMPLATE_PRIM_NAME: &str = "beacon_template";

/// Lists the spaces currently occupied per the registries this client
/// follows, laid out on a table in front of the player.
pub struct Nav {
    root:         Prim,
    beacon_lists: Vec<ListFuture>,
    seen:         Vec<String>,
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
            beacon_lists: Vec::new(),
            seen: Vec::new(),
            beacons: Vec::new(),
        }
    }

    pub fn open(&mut self, placement: Transform) -> anyhow::Result<()> {
        self.root.set_xform(Some(Xform {
            translation: placement.translation,
            rotation:    placement.rotation,
            scale:       Vec3::ONE,
        }))?;
        let wds = get_wds()?;
        self.beacon_lists = wds
            .registries()
            .iter()
            .map(|registry| wds.list(registry, ACTIVE_PREFIX))
            .collect();
        Ok(())
    }

    pub fn close(&mut self) -> anyhow::Result<()> {
        self.root.set_xform(Some(hidden()))?;
        self.beacon_lists.clear();
        self.seen.clear();

        for doc in self.beacons.drain(..) {
            remove_document(&doc.id())?;
        }
        Ok(())
    }

    pub fn fixed_update(&mut self) -> anyhow::Result<()> {
        let mut spaces = Vec::new();
        for i in (0..self.beacon_lists.len()).rev() {
            let Some(result) = self.beacon_lists[i].poll() else {
                continue;
            };
            self.beacon_lists.remove(i);
            match result {
                Ok(entries) => {
                    for entry in entries {
                        if let Some(space) = active_space_hex(&entry.key) {
                            spaces.push(space.to_string());
                        }
                    }
                }
                Err(()) => eprintln!("nav: registry active-space list error"),
            }
        }

        if spaces.is_empty() {
            return Ok(());
        }

        let doc = self_document()?;
        let Some(template) = doc
            .prims()
            .into_iter()
            .find(|p| p.name().is_some_and(|n| n == TEMPLATE_PRIM_NAME))
            .and_then(|p| p.prefab())
        else {
            eprintln!("nav: gauntlet HSD missing {TEMPLATE_PRIM_NAME} prim");
            return Ok(());
        };

        for space in spaces {
            if self.seen.contains(&space) {
                continue;
            }
            self.seen.push(space.clone());

            // Beacons sync per document, and an instance under one of our
            // prims has no document to sync.
            let beacon = create_document_from_prefab(&template)?;
            let prim = beacon.create_prim()?;
            prim.set_name(Some(&space))?;
            set_translation(&prim, grid_offset(self.beacons.len()));

            beacon.set_anchor(Some(&self.root))?;

            self.beacons.push(beacon);
        }
        Ok(())
    }
}

impl Default for Nav {
    fn default() -> Self {
        Self::new()
    }
}
