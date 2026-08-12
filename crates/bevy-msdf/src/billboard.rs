use bevy::{
    prelude::*,
    transform::TransformSystems,
};

/// Turns a body to face the viewer.
#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Billboard {
    /// Spins about the world's up axis only, so a label stays upright.
    #[default]
    Yaw,
    /// Faces the viewer from any angle, including overhead.
    Full,
}

pub fn plugin(app: &mut App) {
    app.add_systems(
        PostUpdate,
        // Before propagation, so a billboard's children inherit the facing it
        // was given this frame rather than the previous one.
        face_viewer.before(TransformSystems::Propagate),
    );
}

pub fn face_viewer(
    cameras: Query<(&GlobalTransform, &Camera)>,
    parents: Query<&GlobalTransform>,
    mut billboards: Query<(
        &mut Transform,
        &GlobalTransform,
        &Billboard,
        Option<&ChildOf>,
    )>,
) {
    let Some(eye) = cameras
        .iter()
        .find(|(_, camera)| camera.is_active)
        .map(|(transform, _)| transform.translation())
    else {
        return;
    };

    for (mut transform, global, billboard, parent) in &mut billboards {
        let to_eye = eye - global.translation();
        let Some(facing) = look(to_eye, *billboard) else {
            continue;
        };
        // A billboard is usually a child of the thing it labels, so the
        // facing has to be expressed in the parent's frame or the parent's
        // own rotation turns it back off-axis.
        let local = parent
            .and_then(|parent| parents.get(parent.parent()).ok())
            .map_or(facing, |parent| parent.rotation().inverse() * facing);
        if transform.rotation.abs_diff_eq(local, 1.0e-5) {
            continue;
        }
        transform.rotation = local;
    }
}

fn look(to_eye: Vec3, billboard: Billboard) -> Option<Quat> {
    let direction = match billboard {
        Billboard::Yaw => Vec3::new(to_eye.x, 0.0, to_eye.z),
        Billboard::Full => to_eye,
    };
    // Directly above a yaw billboard the horizontal component vanishes and
    // there is no answer; keeping the last one beats snapping to an arbitrary
    // heading.
    Dir3::new(direction)
        .ok()
        .map(|direction| Quat::from_rotation_arc(Vec3::Z, direction.as_vec3()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn facing(to_eye: Vec3, billboard: Billboard) -> Vec3 {
        look(to_eye, billboard).expect("facing") * Vec3::Z
    }

    #[test]
    fn a_billboard_turns_its_face_toward_the_viewer() {
        let to_eye = Vec3::new(1.0, 0.0, 1.0).normalize();
        assert!((facing(to_eye, Billboard::Yaw) - to_eye).length() < 1.0e-5);
    }

    #[test]
    fn a_yaw_billboard_ignores_how_high_the_viewer_stands() {
        let facing = facing(Vec3::new(0.0, 5.0, -2.0), Billboard::Yaw);
        assert!(facing.y.abs() < 1.0e-5, "a label does not tilt back");
        assert!(facing.z < 0.0);
    }

    #[test]
    fn a_full_billboard_does_not() {
        let to_eye = Vec3::new(0.0, 5.0, -2.0).normalize();
        assert!((facing(to_eye, Billboard::Full) - to_eye).length() < 1.0e-5);
    }

    #[test]
    fn a_viewer_directly_overhead_leaves_a_yaw_billboard_alone() {
        assert!(look(Vec3::Y, Billboard::Yaw).is_none());
    }

    #[test]
    fn a_viewer_at_the_billboard_leaves_it_alone() {
        assert!(look(Vec3::ZERO, Billboard::Full).is_none());
    }
}
