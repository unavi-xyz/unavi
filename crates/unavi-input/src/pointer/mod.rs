use bevy::{
    camera::NormalizedRenderTarget,
    input::mouse::AccumulatedMouseScroll,
    picking::pointer::{
        Location,
        PointerAction,
        PointerButton,
        PointerId,
        PointerInput,
        PointerInteraction,
        PointerLocation,
    },
    prelude::*,
    window::{
        PrimaryWindow,
        WindowRef,
    },
};
use uuid::Uuid;

use crate::{
    action::{
        Action,
        ActionState,
    },
    capture::Captured,
    config::InputConfig,
};

pub mod backend;

const POINTER_NAMESPACE: u128 = 0x554E_4156_495F_504F_494E_5445_5200_0000;

/// Short of the window on both axes, so no node and no camera claims it.
const OFF_SCREEN: Vec2 = Vec2::splat(-1.0);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PointerKind {
    Screen,
    LeftHand,
    RightHand,
}

impl PointerKind {
    pub const ALL: [Self; Self::COUNT] = [Self::Screen, Self::LeftHand, Self::RightHand];
    pub const COUNT: usize = 3;

    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Self::Screen => 0,
            Self::LeftHand => 1,
            Self::RightHand => 2,
        }
    }

    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Screen => "screen",
            Self::LeftHand => "left_hand",
            Self::RightHand => "right_hand",
        }
    }

    /// A stable id, because `bevy_picking` keys hover and press state by it and
    /// a pointer entity may be respawned when the rig is.
    #[must_use]
    pub const fn id(self) -> PointerId {
        PointerId::Custom(Uuid::from_u128(POINTER_NAMESPACE + self.index() as u128))
    }
}

/// Marks an entity as carrying a pointer. Whatever it is parented to aims it:
/// the tracked head on desktop, a grip pose in VR.
#[derive(Component, Clone, Copy)]
#[require(Transform, Visibility)]
pub struct PointerAnchor(pub PointerKind);

/// How far the pointer can reach, in metres.
#[derive(Component, Clone, Copy)]
pub struct PointerReach(pub f32);

/// Where a pointer's ray met the world.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PointerHit {
    pub entity:   Entity,
    pub position: Vec3,
    pub normal:   Vec3,
    pub distance: f32,
}

/// What a pointer was aimed at when one of its buttons moved.
///
/// Unlike Bevy's `Pointer<Press>` these fire even when the ray hit nothing,
/// which is what lets a listener hear a press aimed at empty space, and what
/// lets a script answer a grip by making something grabbable after the fact.
#[derive(Clone, Copy)]
pub struct PointerAim {
    pub kind:    PointerKind,
    pub pointer: Entity,
    pub ray:     Ray3d,
    pub reach:   f32,
    pub hit:     Option<PointerHit>,
}

/// The trigger went down: acting on whatever is pointed at.
#[derive(Message, Clone, Copy, Deref)]
pub struct PointerPressed(pub PointerAim);

#[derive(Message, Clone, Copy, Deref)]
pub struct PointerReleased(pub PointerAim);

/// The grip closed: taking hold of whatever is pointed at.
#[derive(Message, Clone, Copy, Deref)]
pub struct GripPressed(pub PointerAim);

#[derive(Message, Clone, Copy, Deref)]
pub struct GripReleased(pub PointerAim);

pub fn attach_pointers(
    trigger: On<Add, PointerAnchor>,
    anchors: Query<&PointerAnchor>,
    config: Res<InputConfig>,
    mut commands: Commands,
) {
    let Ok(anchor) = anchors.get(trigger.entity) else {
        return;
    };
    commands
        .entity(trigger.entity)
        .insert((anchor.0.id(), PointerReach(config.tuning.pointer_reach)));
}

/// Bevy drops pointer events for a pointer with no location.
///
/// A ray aimed by a tracked hand has no place on a render target, so every
/// pointer is simply parked at the window's centre and aimed by its transform
/// instead. While something else holds the input they are parked outside the
/// window instead of nowhere: a located pointer that covers no node hovers
/// nothing and still carries the release that unwinds whatever it was on.
pub fn locate_pointers(
    windows: Query<(Entity, &Window), With<PrimaryWindow>>,
    pointers: Query<&mut PointerLocation, With<PointerAnchor>>,
    captured: Res<Captured>,
) {
    let Ok((entity, window)) = windows.single() else {
        return;
    };
    let Some(target) = WindowRef::Primary.normalize(Some(entity)) else {
        return;
    };
    let location = Location {
        target:   NormalizedRenderTarget::Window(target),
        position: if captured.0 {
            OFF_SCREEN
        } else {
            window.size() / 2.0
        },
    };

    for mut pointer in pointers {
        if pointer.location.as_ref() != Some(&location) {
            pointer.location = Some(location.clone());
        }
    }
}

/// Turns the two bound buttons into pointer presses. Bevy's own mouse pointer
/// reaches nothing but the UI, so these are the only things that press on the
/// world.
///
/// The trigger presses as primary and the grip as secondary, which is how a
/// script listening on a prim hears both through the picking it already uses.
pub fn emit_pointer_input(
    state: Res<ActionState>,
    pointers: Query<(&PointerAnchor, &PointerLocation)>,
    scroll: Res<AccumulatedMouseScroll>,
    captured: Res<Captured>,
    mut input: MessageWriter<PointerInput>,
) {
    for (anchor, location) in pointers {
        let Some(location) = location.location().cloned() else {
            continue;
        };
        let kind = anchor.0;

        for (action, button) in [
            (Action::Trigger(kind), PointerButton::Primary),
            (Action::Grip(kind), PointerButton::Secondary),
        ] {
            if state.just_pressed(action) {
                input.write(PointerInput::new(
                    kind.id(),
                    location.clone(),
                    PointerAction::Press(button),
                ));
            }
            if state.just_released(action) {
                input.write(PointerInput::new(
                    kind.id(),
                    location.clone(),
                    PointerAction::Release(button),
                ));
            }
        }

        // The wheel is the one input with no press to silence: it is read
        // straight off the frame rather than out of an action.
        if kind == PointerKind::Screen && !captured.0 && scroll.delta != Vec2::ZERO {
            input.write(PointerInput::new(
                kind.id(),
                location,
                PointerAction::Scroll {
                    unit:  scroll.unit,
                    x:     scroll.delta.x,
                    y:     scroll.delta.y,
                    phase: bevy::input::touch::TouchPhase::Moved,
                },
            ));
        }
    }
}

/// Reports the same button moves again once hit-testing has caught up, so
/// anything needing the *missed* press has one place to read it.
pub fn relay_presses(
    state: Res<ActionState>,
    pointers: Query<(
        Entity,
        &PointerAnchor,
        &PointerReach,
        &GlobalTransform,
        &PointerInteraction,
    )>,
    mut pressed: MessageWriter<PointerPressed>,
    mut released: MessageWriter<PointerReleased>,
    mut gripped: MessageWriter<GripPressed>,
    mut let_go: MessageWriter<GripReleased>,
) {
    for (entity, anchor, reach, transform, interaction) in pointers {
        let kind = anchor.0;
        let trigger = Action::Trigger(kind);
        let grip = Action::Grip(kind);
        if ![trigger, grip]
            .into_iter()
            .any(|action| state.just_pressed(action) || state.just_released(action))
        {
            continue;
        }

        let aim = PointerAim {
            kind,
            pointer: entity,
            ray: ray_of(transform),
            reach: reach.0,
            hit: nearest_hit(interaction),
        };

        if state.just_pressed(trigger) {
            pressed.write(PointerPressed(aim));
        }
        if state.just_released(trigger) {
            released.write(PointerReleased(aim));
        }
        if state.just_pressed(grip) {
            gripped.write(GripPressed(aim));
        }
        if state.just_released(grip) {
            let_go.write(GripReleased(aim));
        }
    }
}

#[must_use]
pub fn ray_of(transform: &GlobalTransform) -> Ray3d {
    let (_, rotation, translation) = transform.to_scale_rotation_translation();
    Ray3d::new(
        translation,
        Dir3::new(rotation * Vec3::NEG_Z).unwrap_or(Dir3::NEG_Z),
    )
}

/// The nearest place a pointer's ray meets a surface in the world.
///
/// A hit carrying no normal is not one of those. `bevy_ui`'s backend picks
/// whatever node covers the point every pointer is parked at, and reports an
/// offset within that node — a number that would read as a position in space
/// and put the reticle, a grab, and a script's aim somewhere nothing is.
#[must_use]
pub fn nearest_hit(interaction: &PointerInteraction) -> Option<PointerHit> {
    interaction.iter().find_map(|(entity, data)| {
        Some(PointerHit {
            entity:   *entity,
            position: data.position?,
            normal:   data.normal?,
            distance: data.depth,
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_pointer_has_its_own_id() {
        let mut ids = PointerKind::ALL.map(PointerKind::id).to_vec();
        ids.dedup();
        assert_eq!(ids.len(), PointerKind::COUNT);
    }

    #[test]
    fn a_pointer_aims_where_its_anchor_faces() {
        let transform =
            GlobalTransform::from(Transform::from_xyz(1.0, 2.0, 3.0).looking_to(Vec3::X, Vec3::Y));
        let ray = ray_of(&transform);
        assert_eq!(ray.origin, Vec3::new(1.0, 2.0, 3.0));
        assert!(ray.direction.distance(Vec3::X) < 1.0e-5);
    }
}
