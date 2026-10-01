use avian3d::prelude::{
    Position,
    Rotation,
};
use bevy::prelude::*;
use hsd::{
    attributes::xform::XformAttr,
    property::Payload,
};
use unavi_physics::finite;
use unavi_util::hierarchy::global_transform;

#[derive(Component, Debug, Clone, Copy)]
pub struct XformData(pub XformAttr);

/// Removal resets `Transform` rather than removing it, since `Prim` requires
/// it.
pub fn apply(
    commands: &mut Commands,
    prim: Entity,
    payload: Option<&[u8]>,
) -> Result<(), postcard::Error> {
    match payload {
        Some(payload) => {
            commands
                .entity(prim)
                .insert(XformData(XformAttr::decode(payload)?));
        }
        None => {
            commands
                .entity(prim)
                .remove::<XformData>()
                .insert(Transform::default());
        }
    }
    Ok(())
}

/// `None` on any non-finite component. A zero scale is valid and hides the
/// prim.
fn checked(attr: &XformAttr) -> Option<Transform> {
    Some(Transform {
        translation: finite::vec3(attr.translation)?,
        rotation:    finite::quat(attr.rotation)?,
        scale:       finite::vec3(attr.scale)?,
    })
}

pub fn apply_xform(
    changed: Query<(Entity, &XformData), Changed<XformData>>,
    mut transforms: Query<&mut Transform>,
    mut physics: Query<(Option<&mut Position>, Option<&mut Rotation>)>,
    parents: Query<&ChildOf>,
) {
    for (entity, data) in &changed {
        let Some(xform) = checked(&data.0) else {
            warn!(?entity, xform = ?data.0, "xform is not a transform; ignoring");
            continue;
        };

        {
            let Ok(mut transform) = transforms.get_mut(entity) else {
                warn!(?entity, "xform: Transform not found");
                continue;
            };
            *transform = xform;
        }

        let Ok((position, rotation)) = physics.get_mut(entity) else {
            continue;
        };
        if position.is_none() && rotation.is_none() {
            continue;
        }

        let global = global_transform(entity, &transforms.as_readonly(), &parents);
        if let Some(mut p) = position {
            p.0 = global.translation;
        }
        if let Some(mut r) = rotation {
            r.0 = global.rotation;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn attr() -> XformAttr {
        XformAttr::default()
    }

    #[test]
    fn a_finite_xform_passes_through() {
        let xform = checked(&XformAttr {
            translation: [1.0, 2.0, 3.0],
            rotation:    [0.0, 0.0, 0.0, 1.0],
            scale:       [2.0; 3],
        })
        .expect("finite");
        assert_eq!(xform.translation, Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(xform.scale, Vec3::splat(2.0));
    }

    #[test]
    fn a_zero_scale_is_kept() {
        let xform = checked(&XformAttr {
            scale: [0.0; 3],
            ..attr()
        })
        .expect("a hidden prim is not a broken one");
        assert_eq!(xform.scale, Vec3::ZERO);
    }

    #[test]
    fn a_non_finite_component_rejects_the_whole_xform() {
        for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert!(
                checked(&XformAttr {
                    translation: [bad, 0.0, 0.0],
                    ..attr()
                })
                .is_none(),
                "translation {bad} was accepted"
            );
            assert!(
                checked(&XformAttr {
                    scale: [1.0, bad, 1.0],
                    ..attr()
                })
                .is_none(),
                "scale {bad} was accepted"
            );
        }
        assert!(
            checked(&XformAttr {
                rotation: [0.0; 4],
                ..attr()
            })
            .is_none(),
            "the zero quaternion was accepted"
        );
    }
}
