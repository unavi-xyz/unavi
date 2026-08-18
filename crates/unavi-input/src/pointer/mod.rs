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
    config::InputConfig,
};

pub mod backend;
pub mod claims;

const POINTER_NAMESPACE: u128 = 0x554E_4156_495F_504F_494E_5445_5200_0000;

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

/// A bound grab went down. Unlike Bevy's `Pointer<Press>` this fires even when
/// the ray hit nothing, which is what lets a script answer a grab by making
/// something grabbable after the fact.
#[derive(Message, Clone, Copy)]
pub struct PointerPressed {
    pub kind:    PointerKind,
    pub pointer: Entity,
    pub ray:     Ray3d,
    pub reach:   f32,
    pub hit:     Option<PointerHit>,
}

#[derive(Message, Clone, Copy)]
pub struct PointerReleased {
    pub kind:    PointerKind,
    pub pointer: Entity,
    pub ray:     Ray3d,
    pub reach:   f32,
    pub hit:     Option<PointerHit>,
}

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
/// instead.
pub fn locate_pointers(
    windows: Query<(Entity, &Window), With<PrimaryWindow>>,
    pointers: Query<&mut PointerLocation, With<PointerAnchor>>,
) {
    let Ok((entity, window)) = windows.single() else {
        return;
    };
    let Some(target) = WindowRef::Primary.normalize(Some(entity)) else {
        return;
    };
    let location = Location {
        target:   NormalizedRenderTarget::Window(target),
        position: window.size() / 2.0,
    };

    for mut pointer in pointers {
        if pointer.location.as_ref() != Some(&location) {
            pointer.location = Some(location.clone());
        }
    }
}

/// Turns bound grabs into pointer presses. Bevy's own mouse and touch readers
/// are off, so what the config calls a grab is the only thing that can press.
pub fn emit_pointer_input(
    state: Res<ActionState>,
    pointers: Query<(&PointerAnchor, &PointerLocation)>,
    scroll: Res<AccumulatedMouseScroll>,
    mut input: MessageWriter<PointerInput>,
) {
    for (anchor, location) in pointers {
        let Some(location) = location.location().cloned() else {
            continue;
        };
        let kind = anchor.0;

        if state.just_pressed(Action::Grab(kind)) {
            input.write(PointerInput::new(
                kind.id(),
                location.clone(),
                PointerAction::Press(PointerButton::Primary),
            ));
        }

        if state.just_released(Action::Grab(kind)) {
            input.write(PointerInput::new(
                kind.id(),
                location.clone(),
                PointerAction::Release(PointerButton::Primary),
            ));
        }

        if kind == PointerKind::Screen && scroll.delta != Vec2::ZERO {
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

/// Reports the same presses again once hit-testing has caught up, so anything
/// needing the *missed* press has one place to read it.
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
) {
    for (entity, anchor, reach, transform, interaction) in pointers {
        let kind = anchor.0;
        let down = state.just_pressed(Action::Grab(kind));
        let up = state.just_released(Action::Grab(kind));
        if !down && !up {
            continue;
        }

        let ray = ray_of(transform);
        let hit = nearest_hit(interaction);

        if down {
            pressed.write(PointerPressed {
                kind,
                pointer: entity,
                ray,
                reach: reach.0,
                hit,
            });
        }
        if up {
            released.write(PointerReleased {
                kind,
                pointer: entity,
                ray,
                reach: reach.0,
                hit,
            });
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

#[must_use]
pub fn nearest_hit(interaction: &PointerInteraction) -> Option<PointerHit> {
    let (entity, data) = interaction.get_nearest_hit()?;
    Some(PointerHit {
        entity:   *entity,
        position: data.position?,
        normal:   data.normal.unwrap_or(Vec3::Y),
        distance: data.depth,
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
