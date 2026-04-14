use wired_prelude::wired_math::types::Vec3;

use crate::{
    unavi::shapes::api::Cuboid,
    wired::scene::{context::self_document, types::RigidBodyKind},
};

wired_prelude::generate_script!(Script);

const BEAM_THICKNESS: f32 = 0.2;
const PODIUM_HEIGHT: f32 = 0.6;
const PORTAL_HEIGHT: f32 = 2.5;
const PORTAL_WIDTH: f32 = 1.0;

struct Script;

impl GuestScript for Script {
    fn new() -> Self {
        let doc = self_document();

        let pole = Cuboid::new(BEAM_THICKNESS, PORTAL_HEIGHT, BEAM_THICKNESS);

        let node_l = doc.create_node();
        node_l.set_mesh(Some(&pole.mesh()));
        node_l.set_collider(Some(&pole.collider()));
        node_l.set_rigid_body(Some(RigidBodyKind::Fixed));
        node_l.set_translation(Vec3::new(-PORTAL_WIDTH / 2.0, 0.0, 0.0));

        let node_r = doc.create_node();
        node_r.set_mesh(Some(&pole.mesh()));
        node_r.set_collider(Some(&pole.collider()));
        node_r.set_rigid_body(Some(RigidBodyKind::Fixed));
        node_r.set_translation(Vec3::new(PORTAL_WIDTH / 2.0, 0.0, 0.0));

        let beam = Cuboid::new(
            BEAM_THICKNESS.mul_add(2.0, PORTAL_WIDTH),
            BEAM_THICKNESS,
            BEAM_THICKNESS,
        );

        let node_t = doc.create_node();
        node_t.set_mesh(Some(&beam.mesh()));
        node_t.set_collider(Some(&beam.collider()));
        node_t.set_rigid_body(Some(RigidBodyKind::Fixed));

        let podium = Cuboid::new(BEAM_THICKNESS, PODIUM_HEIGHT, BEAM_THICKNESS);

        let receptor = doc.create_node();
        receptor.set_translation(Vec3::new(-PORTAL_WIDTH, 0.0, 0.0));
        receptor.set_mesh(Some(&podium.mesh()));
        receptor.set_collider(Some(&podium.collider()));
        receptor.set_rigid_body(Some(RigidBodyKind::Fixed));

        Self
    }

    fn tick(&self) {}

    fn render(&self) {}

    fn drop(&self) {}
}
