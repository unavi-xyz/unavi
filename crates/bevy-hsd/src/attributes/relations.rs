use bevy::{
    ecs::system::SystemParam,
    prelude::*,
};
use hsd::property::name::PropName;

use crate::{
    HsdChild,
    HsdPrimIndex,
    HsdRelationships,
};

/// Resolves a relationship property to the prim entity it names, through the
/// naming prim's document `HsdPrimIndex`.
#[derive(SystemParam)]
pub struct PrimRelations<'w, 's> {
    children:      Query<'w, 's, &'static HsdChild>,
    indices:       Query<'w, 's, &'static HsdPrimIndex>,
    relationships: Query<'w, 's, &'static HsdRelationships>,
}

impl PrimRelations<'_, '_> {
    /// The entity `name` names on `prim`, `None` if the relationship is
    /// unset or its target is not in the same document's prim index.
    pub(crate) fn target(&self, prim: Entity, name: &PropName) -> Option<Entity> {
        let rels = self.relationships.get(prim).ok()?;
        let target = rels.0.get(name)?;
        let doc = self.children.get(prim).ok()?.0;
        self.indices.get(doc).ok()?.0.get(target).copied()
    }
}
