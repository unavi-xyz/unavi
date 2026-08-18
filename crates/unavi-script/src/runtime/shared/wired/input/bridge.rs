use bevy::{
    ecs::system::SystemParam,
    input::mouse::AccumulatedMouseScroll,
    picking::{
        backend::HitData,
        events::{
            Enter,
            Leave,
            Pointer,
            Press,
            Release,
            Scroll,
        },
        pointer::PointerId as BevyPointerId,
    },
    prelude::*,
};
use bevy_hsd::{
    HsdChild,
    HsdDocId,
    Prim,
};
use hsd::id::{
    DocId,
    PrimId,
};
use unavi_input::{
    action::{
        Action,
        ActionState,
    },
    pointer::{
        PointerAnchor,
        PointerKind,
        PointerPressed,
        PointerReleased,
        ray_of,
    },
};

use crate::runtime::shared::wired::input::{
    listener::InputQueue,
    types::{
        Hit,
        InputAction,
        InputEvent,
        PointerId,
        Ray,
    },
};

#[derive(Component)]
pub struct GlobalInputListener {
    pub queue: InputQueue,
}

#[derive(Component)]
pub struct InputListener {
    pub target_doc:  DocId,
    pub target_prim: PrimId,
    pub queue:       InputQueue,
}

/// Everything needed to turn a Bevy pointer event into a delivered one.
#[derive(SystemParam)]
pub struct PrimDelivery<'w, 's> {
    pointers:  Query<'w, 's, (&'static PointerAnchor, &'static GlobalTransform)>,
    listeners: Query<'w, 's, &'static InputListener>,
    prims:     Query<'w, 's, (&'static Prim, &'static HsdChild)>,
    docs:      Query<'w, 's, &'static HsdDocId>,
}

impl PrimDelivery<'_, '_> {
    /// Delivers to every listener registered on the prim the event landed on.
    ///
    /// Bevy propagates `Pointer<E>` up the hierarchy itself, so an event on a
    /// leaf collider reaches a listener on any ancestor prim without a walk of
    /// our own.
    fn send(&self, entity: Entity, pointer: BevyPointerId, action: InputAction, hit: &HitData) {
        let Some(kind) = kind_of(pointer) else {
            return;
        };
        let Ok((prim, doc)) = self.prims.get(entity) else {
            return;
        };
        let Ok(doc_id) = self.docs.get(doc.0) else {
            return;
        };

        let event = InputEvent {
            pointer: kind.into(),
            action,
            ray: self.ray(kind),
            hit: hit_of(hit),
        };

        for listener in &self.listeners {
            if listener.target_prim == prim.0 && listener.target_doc == doc_id.0 {
                listener.queue.push(event);
            }
        }
    }

    fn ray(&self, kind: PointerKind) -> Ray {
        ray_of_kind(kind, &self.pointers)
    }
}

fn kind_of(id: BevyPointerId) -> Option<PointerKind> {
    PointerKind::ALL.into_iter().find(|kind| kind.id() == id)
}

fn ray_of_kind(kind: PointerKind, pointers: &Query<(&PointerAnchor, &GlobalTransform)>) -> Ray {
    pointers.iter().find(|(anchor, _)| anchor.0 == kind).map_or(
        Ray {
            origin: Vec3::ZERO,
            dir:    Vec3::NEG_Z,
        },
        |(_, transform)| ray_of(transform).into(),
    )
}

fn hit_of(hit: &HitData) -> Option<Hit> {
    Some(Hit {
        position: hit.position?,
        normal:   hit.normal.unwrap_or(Vec3::Y),
        distance: hit.depth,
    })
}

pub fn bridge_press(trigger: On<Pointer<Press>>, delivery: PrimDelivery) {
    delivery.send(
        trigger.entity,
        trigger.pointer_id,
        InputAction::Press,
        &trigger.event.hit,
    );
}

pub fn bridge_release(trigger: On<Pointer<Release>>, delivery: PrimDelivery) {
    delivery.send(
        trigger.entity,
        trigger.pointer_id,
        InputAction::Release,
        &trigger.event.hit,
    );
}

pub fn bridge_enter(trigger: On<Pointer<Enter>>, delivery: PrimDelivery) {
    delivery.send(
        trigger.entity,
        trigger.pointer_id,
        InputAction::Enter,
        &trigger.event.hit,
    );
}

pub fn bridge_leave(trigger: On<Pointer<Leave>>, delivery: PrimDelivery) {
    delivery.send(
        trigger.entity,
        trigger.pointer_id,
        InputAction::Leave,
        &trigger.event.hit,
    );
}

pub fn bridge_scroll(trigger: On<Pointer<Scroll>>, delivery: PrimDelivery) {
    let turned = Vec2::new(trigger.event.x, trigger.event.y);
    delivery.send(
        trigger.entity,
        trigger.pointer_id,
        InputAction::Scroll(turned),
        &trigger.event.hit,
    );
}

fn to_global(event: InputEvent, listeners: &Query<&GlobalInputListener>) {
    for listener in listeners {
        listener.queue.push(event);
    }
}

/// The global half of press and release. A press that hit nothing has no
/// entity event to ride, and a global listener wants it either way.
pub fn bridge_global_presses(
    mut pressed: MessageReader<PointerPressed>,
    mut released: MessageReader<PointerReleased>,
    listeners: Query<&GlobalInputListener>,
) {
    for press in pressed.read() {
        to_global(
            InputEvent {
                pointer: press.kind.into(),
                action:  InputAction::Press,
                ray:     press.ray.into(),
                hit:     press.hit.map(Into::into),
            },
            &listeners,
        );
    }

    for release in released.read() {
        to_global(
            InputEvent {
                pointer: release.kind.into(),
                action:  InputAction::Release,
                ray:     release.ray.into(),
                hit:     release.hit.map(Into::into),
            },
            &listeners,
        );
    }
}

/// Scroll reaches a global listener with nothing under the pointer, which is
/// how a shell hears the wheel while aimed at empty space.
pub fn bridge_global_scroll(
    scroll: Res<AccumulatedMouseScroll>,
    pointers: Query<(&PointerAnchor, &GlobalTransform)>,
    listeners: Query<&GlobalInputListener>,
) {
    if scroll.delta == Vec2::ZERO {
        return;
    }

    to_global(
        InputEvent {
            pointer: PointerId::Screen,
            action:  InputAction::Scroll(scroll.delta),
            ray:     ray_of_kind(PointerKind::Screen, &pointers),
            hit:     None,
        },
        &listeners,
    );
}

/// The menu button is aimed at nothing, so it only ever reaches a global
/// listener — but it still says which hand pressed it.
pub fn bridge_menu(
    state: Res<ActionState>,
    pointers: Query<(&PointerAnchor, &GlobalTransform)>,
    listeners: Query<&GlobalInputListener>,
) {
    for kind in PointerKind::ALL {
        let action = if state.just_pressed(Action::Menu(kind)) {
            InputAction::MenuPress
        } else if state.just_released(Action::Menu(kind)) {
            InputAction::MenuRelease
        } else {
            continue;
        };

        to_global(
            InputEvent {
                pointer: kind.into(),
                action,
                ray: ray_of_kind(kind, &pointers),
                hit: None,
            },
            &listeners,
        );
    }
}
