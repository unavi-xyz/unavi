use std::{
    cell::{Cell, RefCell},
    time::SystemTime,
};

use crate::{
    exports::wired::script::guest_api::{Guest, GuestScript},
    unavi::shapes::api::Sphere,
    wired::{
        agent::{context::local_agent, types::BoneName},
        input::{
            system_api::system_input_listener,
            types::{InputAction, InputDevice, InputListener},
        },
        math::types::{Quat, Transform, Vec3},
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
    gauntlets: [Gauntlet; 3],
    input: InputListener,
    render_time: Cell<SystemTime>,
}

struct Gauntlet {
    bone: RefCell<Option<Node>>,
    core: Node,
    open: Cell<bool>,
    pressed: Cell<bool>,
    target: BoneName,
}

const CORE_DISTANCE: f32 = 2.0;
const CORE_RADIUS: f32 = 0.04;
const OPEN_SPEED_SECONDS: f32 = 0.05;

impl GuestScript for Script {
    fn new() -> Self {
        let doc = self_document();

        let mat = doc.create_material();
        mat.set_base_color(&[0.9, 0.95, 1.0, 0.9]);

        // TODO: scale mesh based on avatar size
        let mesh = Sphere::new(CORE_RADIUS).mesh();

        let gauntlets = [BoneName::Head, BoneName::LeftHand, BoneName::RightHand].map(|target| {
            let g = Gauntlet {
                bone: RefCell::new(None),
                open: Cell::new(false),
                pressed: Cell::new(false),
                core: doc.create_node(),
                target,
            };
            g.core.set_translation(Vec3 {
                x: 0.0,
                y: 0.0,
                z: -CORE_DISTANCE,
            });
            g.core.set_scale(Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            });
            g.core.set_mesh(Some(&mesh));
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
            let idx = match event.device {
                InputDevice::Keyboard => 0,
                InputDevice::LeftHand => 1,
                InputDevice::RightHand => 2,
            };

            let pressed = match event.action {
                InputAction::MenuDown => true,
                InputAction::MenuUp => false,
                _ => continue,
            };

            let g = &self.gauntlets[idx];

            let was_pressed = g.pressed.get();
            g.pressed.set(pressed);

            // Toggle menu on button down.
            if !was_pressed && pressed {
                let was_open = g.open.get();
                let open = !was_open;
                g.open.set(open);

                // Detach node while open, to prevent movement.
                if open {
                    let mut tr = g.core.global_transform();
                    if let Some(p) = g.core.parent() {
                        p.remove_child(&g.core);
                        // Replace rotation, which is NaN if scale is 0.
                        tr.rotation = p.rotation();
                    }
                    g.core.set_transform(tr);
                } else {
                    // TODO move after close
                    g.core.set_transform(Transform {
                        scale: Vec3 {
                            x: 1.0,
                            y: 1.0,
                            z: 1.0,
                        },
                        rotation: Quat {
                            x: 0.0,
                            y: 0.0,
                            z: 0.0,
                            w: 1.0,
                        },
                        translation: Vec3 {
                            x: 0.0,
                            y: 0.0,
                            z: -CORE_DISTANCE,
                        },
                    });

                    if let Some(bone_ref) = g.bone.borrow().as_ref() {
                        bone_ref.add_child(&g.core);
                    }
                }
            }
        }

        for g in &self.gauntlets {
            let mut bone_ref = g.bone.borrow_mut();
            if bone_ref.is_none() {
                let agent = local_agent();
                let node = agent.bone(g.target);
                if let Some(n) = &node {
                    n.add_child(&g.core);
                }
                *bone_ref = node;
                return;
            }
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
