use loro::{Container, LoroMap, ValueOrContainer};
use lorosurgeon::{
    Hydrate, HydrateError, MaybeMissing, Reconcile, ReconcileError, reconcile::PropReconciler,
};

pub mod asset;
pub mod collider;
pub mod image;
pub mod material;
pub mod mesh;
pub mod name;
pub mod rigid_body;
pub mod script;
pub mod xform;

pub const ATTRIBUTES_KEY: &str = "attributes";
pub const RELATIONSHIPS_KEY: &str = "relationships";

pub trait Attribute: Reconcile + Hydrate {
    const KEY: &str;

    /// Hydrate this attribute from the inner attributes map (i.e. the map
    /// returned by [`attributes_map`]).
    fn attr_hydrate(attrs: &LoroMap) -> Result<Self, HydrateError> {
        lorosurgeon::hydrate_prop(attrs, Self::KEY)
    }

    fn attr_reconcile(&self, attrs: LoroMap) -> Result<(), ReconcileError> {
        let rec = PropReconciler::map_put(attrs, Self::KEY.to_string());
        self.reconcile(rec)
    }
}

#[derive(Reconcile, Hydrate, Default)]
#[loro(default)]
pub struct Attributes {
    pub asset: MaybeMissing<asset::AssetAttr>,
    pub collider: MaybeMissing<collider::ColliderAttr>,
    pub image: MaybeMissing<image::ImageAttr>,
    pub material: MaybeMissing<material::MaterialAttr>,
    pub mesh: MaybeMissing<mesh::MeshAttr>,
    pub name: MaybeMissing<name::NameAttr>,
    pub rigid_body: MaybeMissing<rigid_body::RigidBodyAttr>,
    pub script: MaybeMissing<script::ScriptAttr>,
    pub xform: MaybeMissing<xform::XformAttr>,
}

/// Returns the inner `attributes` map from a prim's meta map, if present.
#[must_use]
pub fn attributes_map(prim_meta: &LoroMap) -> Option<LoroMap> {
    match prim_meta.get(ATTRIBUTES_KEY)? {
        ValueOrContainer::Container(Container::Map(m)) => Some(m),
        _ => None,
    }
}

/// Returns the inner `relationships` map from a prim's meta map, if present.
#[must_use]
pub fn relationships_map(prim_meta: &LoroMap) -> Option<LoroMap> {
    match prim_meta.get(RELATIONSHIPS_KEY)? {
        ValueOrContainer::Container(Container::Map(m)) => Some(m),
        _ => None,
    }
}

/// Hydrate an attribute from a prim's meta map.
pub fn hydrate_attr<A: Attribute>(prim_meta: &LoroMap) -> Result<A, HydrateError> {
    let attrs = attributes_map(prim_meta).ok_or_else(|| HydrateError::missing(ATTRIBUTES_KEY))?;
    A::attr_hydrate(&attrs)
}
