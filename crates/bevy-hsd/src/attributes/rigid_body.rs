use avian3d::prelude::{AngularDamping, Friction, LinearDamping, Mass, Restitution, RigidBody};
use bevy::prelude::*;
use hsd::{
    HSD_CONTAINER_ID,
    attributes::{
        Attribute, hydrate_attr,
        rigid_body::{RigidBodyAttr, RigidBodyKind},
    },
};
use loro::{ContainerID, Index, TreeID, ValueOrContainer, event::Diff};

use crate::{
    attributes::{
        ApplyEvent, AttrDataEvent, AttributeParser, DocContext, ParseError,
        collider::DisabledRigidBody,
        util::{shallow_map_updated_keys, valid_nonneg, valid_positive},
    },
    diff::HsdDiffEvent,
};

#[derive(Debug)]
pub enum RigidBodyEvent {
    Rebuild(RigidBodyAttr),
}

pub struct RigidBodyParser;

impl AttributeParser for RigidBodyParser {
    fn key(&self) -> &'static str {
        RigidBodyAttr::KEY
    }

    fn lifecycle(
        &self,
        commands: &mut Commands,
        prim: Entity,
        value: Option<ValueOrContainer>,
    ) -> Result<(), ParseError> {
        if value.is_none() {
            commands
                .entity(prim)
                .remove::<RigidBody>()
                .remove::<DisabledRigidBody>()
                .remove::<Friction>()
                .remove::<Restitution>()
                .remove::<Mass>()
                .remove::<LinearDamping>()
                .remove::<AngularDamping>();
        }
        Ok(())
    }

    fn parse(
        &self,
        ctx: &DocContext,
        prim: TreeID,
        path: &[(ContainerID, Index)],
        diff: Diff,
    ) -> Result<(), ParseError> {
        let tree = ctx.doc.get_tree(&*HSD_CONTAINER_ID);
        let meta = tree.get_meta(prim)?;

        let attr: RigidBodyAttr = hydrate_attr(&meta)?;

        let keys = shallow_map_updated_keys(path, diff)?;
        if keys.is_empty() {
            return Ok(());
        }

        ctx.tx
            .send(HsdDiffEvent::AttrData {
                prim,
                data: AttrDataEvent::RigidBody(RigidBodyEvent::Rebuild(attr)),
            })
            .map_err(|_| ParseError::SendDiff)?;
        Ok(())
    }
}

pub fn apply_rigid_body(trigger: On<ApplyEvent<RigidBodyEvent>>, mut commands: Commands) {
    let ent = trigger.entity;
    let RigidBodyEvent::Rebuild(attr) = &trigger.value;

    // Wait for kind to be committed; never fabricate one (a default Dynamic
    // would let static meshes fall for a frame).
    let Some(kind) = attr.kind.as_ref() else {
        return;
    };

    let rb = match kind {
        RigidBodyKind::Dynamic => RigidBody::Dynamic,
        RigidBodyKind::Kinematic => RigidBody::Kinematic,
        RigidBodyKind::Static => RigidBody::Static,
    };
    commands.entity(ent).insert(rb);

    if let Some(&v) = attr.friction.as_ref() {
        if valid_nonneg(v) {
            commands.entity(ent).insert(Friction::new(v as f32));
        } else {
            warn!("rigid_body: friction must be finite and >= 0 (got {v})");
        }
    }

    if let Some(&v) = attr.restitution.as_ref() {
        if valid_nonneg(v) {
            commands.entity(ent).insert(Restitution::new(v as f32));
        } else {
            warn!("rigid_body: restitution must be finite and >= 0 (got {v})");
        }
    }

    if let Some(&v) = attr.mass.as_ref() {
        if valid_positive(v) {
            commands.entity(ent).insert(Mass(v as f32));
        } else {
            warn!("rigid_body: mass must be finite and > 0 (got {v})");
        }
    }

    if let Some(&v) = attr.linear_damping.as_ref() {
        if valid_nonneg(v) {
            commands.entity(ent).insert(LinearDamping(v as f32));
        } else {
            warn!("rigid_body: linear_damping must be finite and >= 0 (got {v})");
        }
    }

    if let Some(&v) = attr.angular_damping.as_ref() {
        if valid_nonneg(v) {
            commands.entity(ent).insert(AngularDamping(v as f32));
        } else {
            warn!("rigid_body: angular_damping must be finite and >= 0 (got {v})");
        }
    }
}
