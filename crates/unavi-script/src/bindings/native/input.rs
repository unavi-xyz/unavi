//! `wired:input`.

use unavi_input::pointer::PointerKind;
use wasmtime::component::Resource;

use super::{
    HostCtx,
    convert,
    generated::wired::{
        core::ids::PrimId,
        input::{
            device,
            targeted,
            types::{
                self,
                Action,
                Button,
                Hit,
                HostInputSubscription,
                InputEvent,
                Pointer,
                PointerKind as WitPointerKind,
            },
        },
    },
};
use crate::{
    error::ScriptError,
    host::{
        input::{
            self,
            InputSubscription,
        },
        scene::DocumentRes,
    },
};

const fn pointer_kind(kind: PointerKind) -> WitPointerKind {
    match kind {
        PointerKind::Screen => WitPointerKind::Screen,
        PointerKind::LeftHand => WitPointerKind::LeftHand,
        PointerKind::RightHand => WitPointerKind::RightHand,
    }
}

const fn button(button: input::Button) -> Button {
    match button {
        input::Button::Trigger => Button::Trigger,
        input::Button::Grip => Button::Grip,
        input::Button::Menu => Button::Menu,
    }
}

fn hit(hit: input::Hit) -> Hit {
    Hit {
        target:   hit.target.map(convert::wit_prim_ref),
        position: convert::wit_vec3(hit.position),
        normal:   convert::wit_vec3(hit.normal),
        distance: hit.distance,
    }
}

fn event(event: input::InputEvent) -> InputEvent {
    InputEvent {
        pointer: pointer_kind(event.pointer),
        action:  match event.action {
            input::Action::Pressed(b) => Action::Pressed(button(b)),
            input::Action::Released(b) => Action::Released(button(b)),
            input::Action::Scroll(delta) => Action::Scroll(convert::wit_vec2(delta)),
            input::Action::Entered => Action::Entered,
            input::Action::Left => Action::Left,
        },
        ray:     convert::wit_ray(event.ray.origin, event.ray.direction),
        hit:     event.hit.map(hit),
    }
}

impl types::Host for HostCtx {}

impl HostInputSubscription for HostCtx {
    fn drain(
        &mut self,
        subscription: Resource<InputSubscription>,
        max: u32,
    ) -> wasmtime::Result<Vec<InputEvent>> {
        Ok(input::drain(&self.host, subscription.rep(), max)?
            .into_iter()
            .map(event)
            .collect())
    }

    fn dropped(&mut self, subscription: Resource<InputSubscription>) -> wasmtime::Result<u64> {
        Ok(input::dropped(&self.host, subscription.rep())?)
    }

    fn drop(&mut self, subscription: Resource<InputSubscription>) -> wasmtime::Result<()> {
        Ok(input::close(&mut self.host, subscription.rep())?)
    }
}

impl targeted::Host for HostCtx {
    fn listen(
        &mut self,
        doc: Resource<DocumentRes>,
        prim: PrimId,
    ) -> Result<Resource<InputSubscription>, ScriptError> {
        input::listen_targeted(&mut self.host, doc.rep(), convert::prim_id(prim))
            .map(Resource::new_own)
    }
}

impl device::Host for HostCtx {
    fn listen(&mut self) -> Result<Resource<InputSubscription>, ScriptError> {
        input::listen_device(&mut self.host).map(Resource::new_own)
    }

    fn pointers(&mut self) -> Result<Vec<Pointer>, ScriptError> {
        Ok(input::pointers(&self.host)?
            .into_iter()
            .map(|pointer| Pointer {
                kind:    pointer_kind(pointer.kind),
                active:  pointer.active,
                ray:     convert::wit_ray(pointer.ray.origin, pointer.ray.direction),
                trigger: pointer.trigger,
                grip:    pointer.grip,
                axis:    convert::wit_vec2(pointer.axis),
                hit:     pointer.hit.map(hit),
            })
            .collect())
    }
}
