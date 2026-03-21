use std::{
    cell::{Cell, RefCell},
    time::SystemTime,
};

use wired_prelude::wired_math::types::Vec3;

use crate::{
    unavi::shapes::api::Sphere,
    wired::{
        agent::{
            context::{local_agent, local_camera},
            types::BoneName,
        },
        input::{
            system_api::system_input_listener,
            types::{InputAction, InputDevice, InputListener},
        },
        scene::{context::self_document, types::Node},
    },
};

wired_prelude::generate_script!(Script);

struct Script {
    gauntlets: [Gauntlet; 3],
    input: InputListener,
    render_time: Cell<SystemTime>,
}

enum Target {
    Bone(BoneName),
    Camera,
}

struct Gauntlet {
    bone: RefCell<Option<Node>>,
    core: Node,
    open: Cell<bool>,
    pressed: Cell<bool>,
    target: Target,
}

const CORE_RADIUS: f32 = 0.04;
const OPEN_SPEED_SECONDS: f32 = 0.05;
const Z_OFFSET: f32 = -0.5;

impl GuestScript for Script {
    fn new() -> Self {
        let doc = self_document();

        let mat = doc.create_material();
        mat.set_base_color(&[0.9, 0.95, 1.0, 0.9]);

        // TODO: scale mesh based on avatar size
        let mesh = Sphere::new(CORE_RADIUS).mesh();

        let gauntlets = [
            Target::Camera,
            Target::Bone(BoneName::LeftHand),
            Target::Bone(BoneName::RightHand),
        ]
        .map(|target| {
            let g = Gauntlet {
                bone: RefCell::new(None),
                open: Cell::new(false),
                pressed: Cell::new(false),
                core: doc.create_node(),
                target,
            };
            g.core.set_scale(Vec3::ZERO);
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
        for g in &self.gauntlets {
            let mut bone_ref = g.bone.borrow_mut();
            if bone_ref.is_none() {
                let node = match g.target {
                    Target::Camera => Some(local_camera()),
                    Target::Bone(b) => local_agent().bone(b),
                };
                *bone_ref = node;
                return;
            }
        }

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

                if open && let Some(bone) = g.bone.borrow().as_ref() {
                    let mut tr = bone.global_transform();
                    tr.translation += tr.rotation * Vec3::new(0.0, 0.0, Z_OFFSET);
                    tr.scale = Vec3::ZERO;
                    g.core.set_transform(tr);
                }
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
