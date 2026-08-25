use std::collections::HashMap;

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
            Scroll,
        },
        pointer::{
            PointerButton as BevyPointerButton,
            PointerId as BevyPointerId,
        },
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

use crate::runtime::shared::wired::input::{
    listener::InputQueue,
    types::{
        Hit,
        InputAction,
        InputEvent,
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

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Button {
    Trigger,
    Grip,
}

/// The prims each button was pressed on, so its release reaches the listeners
/// that heard the press.
///
/// Bevy sends `Pointer<Release>` to whatever is hovered, which is not where
/// the press went: a pointer can be dragged clear of what it pressed, and a
/// prim that hands a held body to the engine gives up its own collider on the
/// way. Without this a listener hears a press it is never told the end of.
#[derive(Resource, Default)]
pub struct Pressing(HashMap<(PointerKind, Button), Vec<(DocId, PrimId)>>);

/// Everything needed to turn a Bevy pointer event into a delivered one.
#[derive(SystemParam)]
pub struct PrimDelivery<'w, 's> {
    pointers:  Query<'w, 's, (&'static PointerAnchor, &'static GlobalTransform)>,
    listeners: Query<'w, 's, &'static InputListener>,
    prims:     Query<'w, 's, (&'static Prim, &'static HsdChild)>,
    docs:      Query<'w, 's, &'static HsdDocId>,
}

impl PrimDelivery<'_, '_> {
    /// Delivers to every listener registered on the prim the event landed on,
    /// reporting which prim that was.
    ///
    /// Bevy propagates `Pointer<E>` up the hierarchy itself, so an event on a
    /// leaf collider reaches a listener on any ancestor prim without a walk of
    /// our own.
    fn send(
        &self,
        entity: Entity,
        pointer: BevyPointerId,
        action: InputAction,
        hit: &HitData,
    ) -> Option<(DocId, PrimId)> {
        let kind = kind_of(pointer)?;
        let (prim, doc) = self.prims.get(entity).ok()?;
        let doc_id = self.docs.get(doc.0).ok()?;

        let event = InputEvent {
            pointer: kind,
            action,
            ray: self.ray(kind),
            hit: hit_of(hit),
        };

        for listener in &self.listeners {
            if listener.target_prim == prim.0 && listener.target_doc == doc_id.0 {
                listener.queue.push(event);
            }
        }
        Some((doc_id.0, prim.0))
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

/// The trigger presses as primary and the grip as secondary, so both reach a
/// prim's listener through the picking the trigger already used.
const fn pressing(button: BevyPointerButton) -> Option<(Button, InputAction)> {
    match button {
        BevyPointerButton::Primary => Some((Button::Trigger, InputAction::Press)),
        BevyPointerButton::Secondary => Some((Button::Grip, InputAction::GripPress)),
        BevyPointerButton::Middle => None,
    }
}

pub fn bridge_press(
    trigger: On<Pointer<Press>>,
    delivery: PrimDelivery,
    mut pressing_on: ResMut<Pressing>,
) {
    let Some((button, down)) = pressing(trigger.event.button) else {
        return;
    };
    let Some(kind) = kind_of(trigger.pointer_id) else {
        return;
    };
    let Some(target) = delivery.send(trigger.entity, trigger.pointer_id, down, &trigger.event.hit)
    else {
        return;
    };

    // The event walks up the hierarchy, so the first step of a press is where
    // its record starts and every ancestor after it is added to the same one.
    let waiting = pressing_on.0.entry((kind, button)).or_default();
    if trigger.original_event_target() == trigger.entity {
        waiting.clear();
    }
    waiting.push(target);
}

/// Ends each press on the prims that heard it, wherever the pointer has since
/// wandered — see [`Pressing`].
pub fn bridge_releases(
    mut released: MessageReader<PointerReleased>,
    mut let_go: MessageReader<GripReleased>,
    mut pressing_on: ResMut<Pressing>,
    listeners: Query<&InputListener>,
) {
    let mut end = |aim: &PointerAim, button: Button, action: InputAction| {
        let Some(waiting) = pressing_on.0.remove(&(aim.kind, button)) else {
            return;
        };
        let event = aimed(aim, action);
        for listener in &listeners {
            if waiting
                .iter()
                .any(|(doc, prim)| listener.target_doc == *doc && listener.target_prim == *prim)
            {
                listener.queue.push(event);
            }
        }
    };

    for release in released.read() {
        end(release, Button::Trigger, InputAction::Release);
    }
    for release in let_go.read() {
        end(release, Button::Grip, InputAction::GripRelease);
    }
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

fn aimed(aim: &PointerAim, action: InputAction) -> InputEvent {
    InputEvent {
        pointer: aim.kind,
        action,
        ray: aim.ray.into(),
        hit: aim.hit.map(Into::into),
    }
}

/// The global half of both buttons. A press that hit nothing has no entity
/// event to ride, and a global listener wants it either way.
pub fn bridge_global_presses(
    mut pressed: MessageReader<PointerPressed>,
    mut released: MessageReader<PointerReleased>,
    mut gripped: MessageReader<GripPressed>,
    mut let_go: MessageReader<GripReleased>,
    listeners: Query<&GlobalInputListener>,
) {
    for press in pressed.read() {
        to_global(aimed(press, InputAction::Press), &listeners);
    }
    for release in released.read() {
        to_global(aimed(release, InputAction::Release), &listeners);
    }
    for press in gripped.read() {
        to_global(aimed(press, InputAction::GripPress), &listeners);
    }
    for release in let_go.read() {
        to_global(aimed(release, InputAction::GripRelease), &listeners);
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
            pointer: PointerKind::Screen,
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
                pointer: kind,
                action,
                ray: ray_of_kind(kind, &pointers),
                hit: None,
            },
            &listeners,
        );
    }
}

#[cfg(test)]
mod tests {
    use bevy::math::Dir3;

    use super::*;

    fn app() -> App {
        let mut app = App::new();
        app.init_resource::<Pressing>()
            .add_message::<PointerReleased>()
            .add_message::<GripReleased>()
            .add_systems(Update, bridge_releases);
        app
    }

    fn listener(app: &mut App, doc: DocId, prim: PrimId) -> InputQueue {
        let queue = InputQueue::default();
        app.world_mut().spawn(InputListener {
            target_doc:  doc,
            target_prim: prim,
            queue:       queue.clone(),
        });
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

    #[test]
    fn a_release_reaches_the_prim_that_heard_the_press() {
        let (doc, prim) = (DocId([1; 32]), PrimId::new());
        let mut app = app();
        let queue = listener(&mut app, doc, prim);
        pressed_on(&mut app, Button::Grip, doc, prim);

        app.world_mut().write_message(GripReleased(aim()));
        app.update();

        assert_eq!(
            queue.pop().map(|event| event.action),
            Some(InputAction::GripRelease),
            "the prim the grip closed on is told the grip opened, wherever \
             the pointer has since wandered — it may have handed its own \
             collider to the engine and be under nothing at all"
        );
    }

    #[test]
    fn a_prim_that_never_heard_the_press_is_not_told_of_the_release() {
        let prim = PrimId::new();
        let mut app = app();
        let elsewhere = listener(&mut app, DocId([2; 32]), prim);
        pressed_on(&mut app, Button::Trigger, DocId([1; 32]), prim);

        app.world_mut().write_message(PointerReleased(aim()));
        app.update();

        assert!(
            elsewhere.pop().is_none(),
            "a release ends a press; a prim with no press pending has nothing \
             to end"
        );
    }

    #[test]
    fn the_two_buttons_end_separately() {
        let (doc, prim) = (DocId([1; 32]), PrimId::new());
        let mut app = app();
        let queue = listener(&mut app, doc, prim);
        pressed_on(&mut app, Button::Grip, doc, prim);

        app.world_mut().write_message(PointerReleased(aim()));
        app.update();
        assert!(
            queue.pop().is_none(),
            "letting the trigger go says nothing about a hand that is still closed"
        );

        app.world_mut().write_message(GripReleased(aim()));
        app.update();
        assert_eq!(
            queue.pop().map(|event| event.action),
            Some(InputAction::GripRelease)
        );
    }

    #[test]
    fn a_press_is_ended_once() {
        let (doc, prim) = (DocId([1; 32]), PrimId::new());
        let mut app = app();
        let queue = listener(&mut app, doc, prim);
        pressed_on(&mut app, Button::Trigger, doc, prim);

        for _ in 0..2 {
            app.world_mut().write_message(PointerReleased(aim()));
            app.update();
        }

        assert_eq!(
            queue.pop().map(|event| event.action),
            Some(InputAction::Release)
        );
        assert!(queue.pop().is_none(), "a second release ends nothing");
    }
}
