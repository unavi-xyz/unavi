use std::{
    cell::{Cell, RefCell},
    time::SystemTime,
};

use crate::{
    exports::wired::script::guest_api::{Guest, GuestScript},
    unavi::shapes::api::Cylinder,
    wired::{
        agent::{context::local_agent, types::BoneName},
        math::types::Vec3,
        scene::{
            context::self_document,
            types::{Collider, ColliderCylinder, Node},
        },
    },
};

wit_bindgen::generate!({
    generate_all,
});

struct World;

impl Guest for World {
    type Script = Script;
}

struct Script {
    bone: RefCell<Option<Node>>,
    head: RefCell<Option<Node>>,
    open: Cell<bool>,
    render_time: Cell<SystemTime>,
    screen: Node,
}

const HEIGHT: f32 = 0.004;
const HOVER: f32 = 0.05;
const RADIUS: f32 = 0.03;

const OPEN_DISTANCE: f32 = 0.5;
const OPEN_SPEED: f32 = 1.6;
const Y_FACTOR: f32 = 4.0;

impl GuestScript for Script {
    fn new() -> Self {
        let doc = self_document();

        let screen = {
            let mesh = Cylinder::new(RADIUS, HEIGHT).mesh();

            let mat = doc.create_material();
            mat.set_base_color(&[1.0, 1.0, 1.0, 0.95]);
            mat.set_metallic(0.8);
            mat.set_roughness(0.8);

            let node = doc.create_node();
            node.set_mesh(Some(mesh));
            node.set_material(Some(mat));
            node.set_scale(Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            });
            node.set_translation(Vec3 {
                x: 0.0,
                y: HOVER,
                z: 0.0,
            });

            let col = Collider::Cylinder(ColliderCylinder {
                half_height: HEIGHT,
                radius: RADIUS,
            });
            node.set_collider(Some(&col));

            node
        };

        Self {
            bone: RefCell::new(None),
            head: RefCell::new(None),
            open: Cell::new(false),
            render_time: Cell::new(SystemTime::now()),
            screen,
        }
    }

    fn tick(&self) {
        let head_ref = self.head.borrow();
        let Some(head) = head_ref.as_ref() else {
            drop(head_ref);
            let agent = local_agent();
            let head = agent.bone(BoneName::Head);
            *self.head.borrow_mut() = head;
            return;
        };

        let bone_ref = self.bone.borrow();
        let Some(bone) = bone_ref.as_ref() else {
            drop(bone_ref);
            let agent = local_agent();
            let bone = agent.bone(BoneName::LeftHand);
            if let Some(n) = &bone {
                n.add_child(self.screen.clone());
            }
            *self.bone.borrow_mut() = bone;
            return;
        };

        // If bone comes close to head, show vscreen.
        // TODO: percentage-based distance, compared to avatar height
        let tr_head = head.global_transform();
        let tr_bone = bone.global_transform();

        let d_x = tr_head.translation.x - tr_bone.translation.x;
        let d_y = tr_head.translation.y - tr_bone.translation.y;
        let d_z = tr_head.translation.z - tr_bone.translation.z;
        let d = (d_x.abs() + d_y.abs().mul_add(Y_FACTOR, d_z.abs())) / 3.0;

        let open = d < OPEN_DISTANCE;
        self.open.set(open);
    }

    fn render(&self) {
        let delta = self
            .render_time
            .get()
            .elapsed()
            .expect("elapsed")
            .as_secs_f32();
        self.render_time.set(SystemTime::now());

        let mut s = self.screen.scale();

        let inc = if self.open.get() {
            delta * OPEN_SPEED
        } else {
            -delta * OPEN_SPEED
        };

        s.x += inc;
        s.y += inc;
        s.z += inc;

        s.x = s.x.clamp(0.0, 1.0);
        s.y = s.y.clamp(0.0, 1.0);
        s.z = s.z.clamp(0.0, 1.0);

        self.screen.set_scale(s);
    }

    fn drop(&self) {}
}

export!(World);
