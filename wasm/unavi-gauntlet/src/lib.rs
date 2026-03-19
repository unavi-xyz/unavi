use std::{
    cell::{Cell, RefCell},
    time::SystemTime,
};

use crate::{
    exports::wired::script::guest_api::{Guest, GuestScript},
    unavi::shapes::api::Sphere,
    wired::{
        agent::{context::local_agent, types::BoneName},
        input::{system_api::system_input_listener, types::InputListener},
        math::types::Vec3,
        scene::{context::self_document, types::Node},
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
    gauntlets: [Gauntlet; 2],
    input: InputListener,
    render_time: Cell<SystemTime>,
}

struct Gauntlet {
    bone: RefCell<Option<Node>>,
    core: Node,
    open: Cell<bool>,
    target: BoneName,
}

const CORE_RADIUS: f32 = 0.05;
const OPEN_SPEED_SECONDS: f32 = 0.25;

impl GuestScript for Script {
    fn new() -> Self {
        let doc = self_document();

        let gauntlets = [BoneName::LeftHand, BoneName::RightHand].map(|target| {
            let g = Gauntlet {
                bone: RefCell::new(None),
                open: Cell::new(false),
                core: doc.create_node(),
                target,
            };
            g.core.set_scale(Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            });

            // TODO: scale based on avatar size
            let mesh = Sphere::new(CORE_RADIUS).mesh();
            g.core.set_mesh(Some(mesh));

            g
        });

        let input = system_input_listener();

        Self {
            gauntlets,
            input,
            render_time: Cell::new(SystemTime::now()),
        }
    }

    fn tick(&self) {
        while let Some(event) = self.input.poll() {
            println!("{event:#?}");
        }

        for g in &self.gauntlets {
            let bone_ref = g.bone.borrow();
            if bone_ref.is_none() {
                let agent = local_agent();
                let node = agent.bone(g.target);
                if let Some(n) = &node {
                    n.add_child(g.core.clone());
                }
                *g.bone.borrow_mut() = node;
                return;
            }

            // TODO take input (b in VR, tab in desktop)

            // TODO toggle open
        }
    }

    fn render(&self) {
        let delta = self
            .render_time
            .get()
            .elapsed()
            .expect("elapsed")
            .as_secs_f32();
        self.render_time.set(SystemTime::now());

        for g in &self.gauntlets {
            let mut s = g.core.scale();

            let inc = if g.open.get() {
                delta / OPEN_SPEED_SECONDS
            } else {
                -delta / OPEN_SPEED_SECONDS
            };

            s.x += inc;
            s.y += inc;
            s.z += inc;

            s.x = s.x.clamp(0.0, 1.0);
            s.y = s.y.clamp(0.0, 1.0);
            s.z = s.z.clamp(0.0, 1.0);

            g.core.set_scale(s);
        }
    }

    fn drop(&self) {}
}

export!(World);
