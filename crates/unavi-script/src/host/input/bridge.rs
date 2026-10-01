//! Turns the engine's pointer events into the input events listeners hear.

use bevy::{
    input::mouse::AccumulatedMouseScroll,
    picking::{
        backend::HitData,
        events::{
            Enter,
            Leave,
            Pointer as PickPointer,
            Press,
            Scroll,
        },
        pointer::{
            PointerButton as PickButton,
            PointerId as PickPointerId,
        },
    },
    platform::collections::HashMap,
    prelude::*,
};
use hsd::id::{
    DocId,
    PrimId,
};
use unavi_input::{
    action::{
        Action as InputAction,
        ActionState,
    },
    pointer::{
        GripPressed,
        GripReleased,
        PointerAim,
        PointerAnchor,
        PointerKind,
        PointerPressed,
        PointerReleased,
        ray_of,
    },
};

use crate::host::{
    input::{
        Action,
        Button,
        Hit,
        InputEvent,
        PrimTargets,
        Ray,
    },
    shared_state::input_listeners::{
        InputListeners,
        Target,
    },
};

/// The prims each button was pressed on, so its release reaches the listeners
/// that heard the press.
///
/// The engine sends a release to whatever is hovered, which is not where the
/// press went: a pointer can be dragged clear of what it pressed, and a prim
/// that hands a held body to the engine gives up its own collider on the way.
/// Without this a listener hears a press it is never told the end of.
#[derive(Resource, Default)]
pub struct Pressing(HashMap<(PointerKind, Button), Vec<(DocId, PrimId)>>);

fn kind_of(id: PickPointerId) -> Option<PointerKind> {
    PointerKind::ALL.into_iter().find(|kind| kind.id() == id)
}

fn ray_of_kind(kind: PointerKind, pointers: &Query<(&PointerAnchor, &GlobalTransform)>) -> Ray {
    pointers.iter().find(|(anchor, _)| anchor.0 == kind).map_or(
        Ray {
            origin:    Vec3::ZERO,
            direction: Vec3::NEG_Z,
        },
        |(_, transform)| ray_of(transform).into(),
    )
}

fn hit_of(hit: &HitData, target: Option<(DocId, PrimId)>) -> Option<Hit> {
    Some(Hit {
        target,
        position: hit.position?,
        normal: hit.normal.unwrap_or(Vec3::Y),
        distance: hit.depth,
    })
}

/// Delivers a picking event at `entity` to the listeners on that prim,
/// answering which prim it was.
///
/// The engine propagates a pointer event up the hierarchy itself, so an event
/// on a leaf collider reaches a listener on any ancestor prim without a walk of
/// our own. The hit names the leaf.
fn deliver(
    entity: Entity,
    original: Entity,
    pointer: PickPointerId,
    action: Action,
    hit: &HitData,
    targets: &PrimTargets,
    pointers: &Query<(&PointerAnchor, &GlobalTransform)>,
    listeners: &InputListeners,
) -> Option<(DocId, PrimId)> {
    let kind = kind_of(pointer)?;
    let (doc, prim) = targets.of(entity)?;
    let target = Target::Prim(doc, prim);
    if listeners.is_heard(target) {
        listeners.send(
            target,
            InputEvent {
                pointer: kind,
                action,
                ray: ray_of_kind(kind, pointers),
                hit: hit_of(hit, targets.of(original)),
            },
        );
    }
    Some((doc, prim))
}

const fn button_of(button: PickButton) -> Option<Button> {
    match button {
        PickButton::Primary => Some(Button::Trigger),
        PickButton::Secondary => Some(Button::Grip),
        PickButton::Middle => None,
    }
}

pub fn bridge_press(
    trigger: On<PickPointer<Press>>,
    targets: PrimTargets,
    pointers: Query<(&PointerAnchor, &GlobalTransform)>,
    listeners: Res<InputListeners>,
    mut pressing: ResMut<Pressing>,
) {
    let Some(button) = button_of(trigger.event.button) else {
        return;
    };
    let Some(kind) = kind_of(trigger.pointer_id) else {
        return;
    };
    let original = trigger.original_event_target();
    let Some(target) = deliver(
        trigger.entity,
        original,
        trigger.pointer_id,
        Action::Pressed(button),
        &trigger.event.hit,
        &targets,
        &pointers,
        &listeners,
    ) else {
        return;
    };

    // The event walks up the hierarchy, so the first step of a press is where
    // its record starts and every ancestor after it joins the same one.
    let waiting = pressing.0.entry((kind, button)).or_default();
    if original == trigger.entity {
        waiting.clear();
    }
    waiting.push(target);
}

pub fn bridge_enter(
    trigger: On<PickPointer<Enter>>,
    targets: PrimTargets,
    pointers: Query<(&PointerAnchor, &GlobalTransform)>,
    listeners: Res<InputListeners>,
) {
    deliver(
        trigger.entity,
        trigger.original_event_target(),
        trigger.pointer_id,
        Action::Entered,
        &trigger.event.hit,
        &targets,
        &pointers,
        &listeners,
    );
}

pub fn bridge_leave(
    trigger: On<PickPointer<Leave>>,
    targets: PrimTargets,
    pointers: Query<(&PointerAnchor, &GlobalTransform)>,
    listeners: Res<InputListeners>,
) {
    deliver(
        trigger.entity,
        trigger.original_event_target(),
        trigger.pointer_id,
        Action::Left,
        &trigger.event.hit,
        &targets,
        &pointers,
        &listeners,
    );
}

pub fn bridge_scroll(
    trigger: On<PickPointer<Scroll>>,
    targets: PrimTargets,
    pointers: Query<(&PointerAnchor, &GlobalTransform)>,
    listeners: Res<InputListeners>,
) {
    deliver(
        trigger.entity,
        trigger.original_event_target(),
        trigger.pointer_id,
        Action::Scroll(Vec2::new(trigger.event.x, trigger.event.y)),
        &trigger.event.hit,
        &targets,
        &pointers,
        &listeners,
    );
}

fn aimed(aim: &PointerAim, action: Action, targets: &PrimTargets) -> InputEvent {
    InputEvent {
        pointer: aim.kind,
        action,
        ray: aim.ray.into(),
        hit: aim.hit.map(|hit| targets.hit(hit)),
    }
}

/// Ends each press on the prims that heard it, wherever the pointer has since
/// wandered (see [`Pressing`]), and tells device listeners of both buttons.
pub fn bridge_buttons(
    mut pressed: MessageReader<PointerPressed>,
    mut released: MessageReader<PointerReleased>,
    mut gripped: MessageReader<GripPressed>,
    mut let_go: MessageReader<GripReleased>,
    mut pressing: ResMut<Pressing>,
    targets: PrimTargets,
    listeners: Res<InputListeners>,
) {
    for press in pressed.read() {
        listeners.send(
            Target::Device,
            aimed(press, Action::Pressed(Button::Trigger), &targets),
        );
    }
    for press in gripped.read() {
        listeners.send(
            Target::Device,
            aimed(press, Action::Pressed(Button::Grip), &targets),
        );
    }

    let mut end = |aim: &PointerAim, button: Button| {
        let event = aimed(aim, Action::Released(button), &targets);
        listeners.send(Target::Device, event);
        for (doc, prim) in pressing.0.remove(&(aim.kind, button)).unwrap_or_default() {
            listeners.send(Target::Prim(doc, prim), event);
        }
    };
    for release in released.read() {
        end(release, Button::Trigger);
    }
    for release in let_go.read() {
        end(release, Button::Grip);
    }
}

/// Scroll reaches device listeners with nothing under the pointer, which is
/// how a shell hears the wheel while aimed at empty space.
pub fn bridge_device_scroll(
    scroll: Res<AccumulatedMouseScroll>,
    pointers: Query<(&PointerAnchor, &GlobalTransform)>,
    listeners: Res<InputListeners>,
) {
    if scroll.delta == Vec2::ZERO || !listeners.is_heard(Target::Device) {
        return;
    }
    listeners.send(
        Target::Device,
        InputEvent {
            pointer: PointerKind::Screen,
            action:  Action::Scroll(scroll.delta),
            ray:     ray_of_kind(PointerKind::Screen, &pointers),
            hit:     None,
        },
    );
}

/// The menu button is aimed at nothing, so it only reaches device listeners,
/// but it still says which hand pressed it.
pub fn bridge_menu(
    state: Res<ActionState>,
    pointers: Query<(&PointerAnchor, &GlobalTransform)>,
    listeners: Res<InputListeners>,
) {
    for kind in PointerKind::ALL {
        let action = if state.just_pressed(InputAction::Menu(kind)) {
            Action::Pressed(Button::Menu)
        } else if state.just_released(InputAction::Menu(kind)) {
            Action::Released(Button::Menu)
        } else {
            continue;
        };
        listeners.send(
            Target::Device,
            InputEvent {
                pointer: kind,
                action,
                ray: ray_of_kind(kind, &pointers),
                hit: None,
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use bevy::math::Dir3;

    use super::*;
    use crate::host::queue::Queue;

    fn app() -> (App, InputListeners) {
        let mut app = App::new();
        let listeners = InputListeners::default();
        app.init_resource::<Pressing>()
            .insert_resource(listeners.clone())
            .add_message::<PointerPressed>()
            .add_message::<PointerReleased>()
            .add_message::<GripPressed>()
            .add_message::<GripReleased>()
            .add_systems(Update, bridge_buttons);
        (app, listeners)
    }

    fn listen(listeners: &InputListeners, target: Target) -> Arc<Queue<InputEvent>> {
        let queue = Arc::new(Queue::default());
        let _ = listeners.open(target, Arc::clone(&queue));
        queue
    }

    fn aim() -> PointerAim {
        PointerAim {
            kind:    PointerKind::Screen,
            pointer: Entity::PLACEHOLDER,
            ray:     Ray3d::new(Vec3::ZERO, Dir3::NEG_Z),
            reach:   5.0,
            hit:     None,
        }
    }

    fn pressed_on(app: &mut App, button: Button, doc: DocId, prim: PrimId) {
        app.world_mut()
            .resource_mut::<Pressing>()
            .0
            .insert((PointerKind::Screen, button), vec![(doc, prim)]);
    }

    fn actions(queue: &Queue<InputEvent>) -> Vec<Action> {
        queue
            .drain(8)
            .into_iter()
            .map(|event| event.action)
            .collect()
    }

    #[test]
    fn a_release_reaches_the_prim_that_heard_the_press() {
        let (doc, prim) = (DocId([1; 32]), PrimId::new());
        let (mut app, listeners) = app();
        let queue = listen(&listeners, Target::Prim(doc, prim));
        pressed_on(&mut app, Button::Grip, doc, prim);

        app.world_mut().write_message(GripReleased(aim()));
        app.update();

        assert_eq!(
            actions(&queue),
            vec![Action::Released(Button::Grip)],
            "the prim the grip closed on is told it opened, wherever the \
             pointer has since wandered"
        );
    }

    #[test]
    fn a_prim_that_never_heard_the_press_is_not_told_of_the_release() {
        let prim = PrimId::new();
        let (mut app, listeners) = app();
        let elsewhere = listen(&listeners, Target::Prim(DocId([2; 32]), prim));
        pressed_on(&mut app, Button::Trigger, DocId([1; 32]), prim);

        app.world_mut().write_message(PointerReleased(aim()));
        app.update();

        assert_eq!(actions(&elsewhere).len(), 0);
    }

    #[test]
    fn the_two_buttons_end_separately_and_once() {
        let (doc, prim) = (DocId([1; 32]), PrimId::new());
        let (mut app, listeners) = app();
        let queue = listen(&listeners, Target::Prim(doc, prim));
        pressed_on(&mut app, Button::Grip, doc, prim);

        app.world_mut().write_message(PointerReleased(aim()));
        app.update();
        assert!(
            actions(&queue).is_empty(),
            "letting the trigger go says nothing about a hand still closed"
        );

        for _ in 0..2 {
            app.world_mut().write_message(GripReleased(aim()));
            app.update();
        }
        assert_eq!(actions(&queue), vec![Action::Released(Button::Grip)]);
    }

    #[test]
    fn a_device_listener_hears_every_button() {
        let (mut app, listeners) = app();
        let queue = listen(&listeners, Target::Device);

        app.world_mut().write_message(PointerPressed(aim()));
        app.world_mut().write_message(PointerReleased(aim()));
        app.update();

        assert_eq!(
            actions(&queue),
            vec![
                Action::Pressed(Button::Trigger),
                Action::Released(Button::Trigger)
            ]
        );
    }
}
