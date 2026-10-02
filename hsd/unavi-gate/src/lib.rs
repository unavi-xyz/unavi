use std::f32::consts::GOLDEN_RATIO;

use wired_guest::math::{
    Transform,
    Vec3,
};

use crate::{
    unavi::shapes::api::Cuboid,
    wired::{
        event::messaging::{
            MessageSubscription,
            Scope,
            Spatial,
        },
        portal::portals::IntentSubscription,
        scene::{
            document::{
                Document,
                script_document,
            },
            properties::{
                Material,
                Portal,
                Property,
                PropertyKey,
                RigidBody,
            },
        },
    },
};

wired_guest::generate_script!(Script);

const CHANNEL: &str = "unavi:beacon/id";
const PORTAL_PRIM_NAME: &str = "portal";
const RECEPTOR_PRIM_NAME: &str = "receptor";

const PORTAL_WIDTH: f32 = GOLDEN_RATIO;
const PORTAL_HEIGHT: f32 = PORTAL_WIDTH * GOLDEN_RATIO;

const BEAM_THICKNESS: f32 = 1.0 / (4.0 * GOLDEN_RATIO);

const PEDESTAL_HEIGHT: f32 = PORTAL_WIDTH / 2.0;
const PEDESTAL_THICKNESS: f32 = BEAM_THICKNESS * GOLDEN_RATIO;
const EVENT_RADIUS: f32 = PEDESTAL_THICKNESS;

struct Script {
    doc:         Document,
    portal_prim: (u64, u64),
    beacon_rx:   MessageSubscription,
    intent_rx:   IntentSubscription,
}

/// Interprets a beacon's 32-byte id as a `document-id`'s little-endian
/// words.
fn as_document_id(bytes: &[u8]) -> Option<(u64, u64, u64, u64)> {
    let bytes: &[u8; 32] = bytes.try_into().ok()?;
    Some((
        u64::from_le_bytes(bytes[0..8].try_into().ok()?),
        u64::from_le_bytes(bytes[8..16].try_into().ok()?),
        u64::from_le_bytes(bytes[16..24].try_into().ok()?),
        u64::from_le_bytes(bytes[24..32].try_into().ok()?),
    ))
}

impl ScriptBehavior for Script {
    fn init() -> anyhow::Result<Self> {
        let doc = script_document()?;
        let root = doc
            .roots()
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("gate has no root prim"))?;

        // Authored prims have ids baked at build time, identical on every
        // peer; a prim minted at runtime gets a per-peer id.
        let portal_prim = find_one(&doc, PORTAL_PRIM_NAME)?;
        let receptor_prim = find_one(&doc, RECEPTOR_PRIM_NAME)?;

        let material = gate_material();
        let mut batch = doc.local().set(
            portal_prim,
            Property::Portal(Portal {
                width:       PORTAL_WIDTH,
                height:      PORTAL_HEIGHT,
                destination: None,
            }),
        );
        batch = spawn_frame(batch, root, material)?;

        let pedestal_shape = Cuboid::new(Vec3::new(
            PEDESTAL_THICKNESS,
            PEDESTAL_HEIGHT,
            PEDESTAL_THICKNESS,
        ));
        let pedestal = pedestal_shape.mesh()?;
        batch = batch
            .set(pedestal, Property::Parent(Some(root)))
            .set(pedestal, Property::Collider(pedestal_shape.collider()))
            .set(pedestal, Property::RigidBody(RigidBody::static_body()))
            .set(pedestal, Property::Material(material))
            .set(
                pedestal,
                Property::Transform(Transform::from_translation(Vec3::new(
                    -PORTAL_WIDTH,
                    PEDESTAL_HEIGHT / 2.0,
                    0.0,
                ))),
            );
        batch.flush()?;

        let beacon_rx = wired::event::messaging::listen(
            &[CHANNEL.to_owned()],
            None,
            Scope::Spatial(Spatial {
                origin: wired::core::ids::PrimRef {
                    document: doc.id(),
                    prim:     receptor_prim,
                },
                radius: EVENT_RADIUS,
            }),
        )?;

        let intent_rx = wired::portal::portals::intents()?;

        println!("Gate ready");

        Ok(Self {
            doc,
            portal_prim,
            beacon_rx,
            intent_rx,
        })
    }

    fn fixed_update(
        &mut self,
        _tick: exports::wired::script::lifecycle::Tick,
    ) -> anyhow::Result<()> {
        for message in self.beacon_rx.drain(8) {
            let Some(target) = as_document_id(&message.payload) else {
                continue;
            };
            let current = match self.doc.get(self.portal_prim, &PropertyKey::Portal) {
                Some(Property::Portal(portal)) => portal.destination,
                _ => None,
            };
            if current.is_some_and(|d| d.space == target) {
                continue;
            }
            wired::portal::portals::open(&self.doc, self.portal_prim, target)?;
        }

        for intent in self.intent_rx.drain(8) {
            // A gate already leading somewhere leaves the intent for another.
            let idle = match self.doc.get(self.portal_prim, &PropertyKey::Portal) {
                Some(Property::Portal(portal)) => portal.destination.is_none(),
                _ => true,
            };
            if !idle {
                continue;
            }
            if !self.intent_rx.claim(intent) {
                continue;
            }
            wired::portal::portals::pair(&self.doc, self.portal_prim, intent)?;
        }
        Ok(())
    }
}

fn spawn_frame(
    mut batch: crate::Batch<'_>,
    root: (u64, u64),
    material: Material,
) -> anyhow::Result<crate::Batch<'_>> {
    let pole = Cuboid::new(Vec3::new(BEAM_THICKNESS, PORTAL_HEIGHT, BEAM_THICKNESS));

    let pole_l = pole.mesh()?;
    batch = batch
        .set(pole_l, Property::Parent(Some(root)))
        .set(pole_l, Property::Collider(pole.collider()))
        .set(pole_l, Property::RigidBody(RigidBody::static_body()))
        .set(pole_l, Property::Material(material))
        .set(
            pole_l,
            Property::Transform(Transform::from_translation(Vec3::new(
                -PORTAL_WIDTH / 2.0 - BEAM_THICKNESS / 2.0,
                PORTAL_HEIGHT / 2.0,
                0.0,
            ))),
        );

    let pole_r = pole.mesh()?;
    batch = batch
        .set(pole_r, Property::Parent(Some(root)))
        .set(pole_r, Property::Collider(pole.collider()))
        .set(pole_r, Property::RigidBody(RigidBody::static_body()))
        .set(pole_r, Property::Material(material))
        .set(
            pole_r,
            Property::Transform(Transform::from_translation(Vec3::new(
                PORTAL_WIDTH / 2.0 + BEAM_THICKNESS / 2.0,
                PORTAL_HEIGHT / 2.0,
                0.0,
            ))),
        );

    let beam = Cuboid::new(Vec3::new(
        BEAM_THICKNESS.mul_add(2.0, PORTAL_WIDTH),
        BEAM_THICKNESS,
        BEAM_THICKNESS,
    ));

    let beam_top = beam.mesh()?;
    batch = batch
        .set(beam_top, Property::Parent(Some(root)))
        .set(beam_top, Property::Collider(beam.collider()))
        .set(beam_top, Property::RigidBody(RigidBody::static_body()))
        .set(beam_top, Property::Material(material))
        .set(
            beam_top,
            Property::Transform(Transform::from_translation(Vec3::new(
                0.0,
                PORTAL_HEIGHT + BEAM_THICKNESS / 2.0,
                0.0,
            ))),
        );

    Ok(batch)
}

const fn gate_material() -> Material {
    Material {
        base_color:   Some(wired_guest::math::Color {
            r: 0.7,
            g: 0.72,
            b: 0.78,
            a: 1.0,
        }),
        metallic:     Some(0.6),
        roughness:    Some(0.4),
        emissive:     None,
        alpha_mode:   None,
        alpha_cutoff: None,
        double_sided: None,
    }
}
