use loro::{Container, LoroMap, ValueOrContainer};
use lorosurgeon::{
    Hydrate, HydrateError, MaybeMissing, Reconcile, ReconcileError, reconcile::PropReconciler,
};
use serde::{Deserialize, Serialize};

pub mod asset;
pub mod collider;
pub mod image;
pub mod material;
pub mod mesh;
pub mod name;
pub mod rigid_body;
pub mod script;
pub mod value_array;
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

#[derive(Reconcile, Hydrate, Default, Clone, Serialize, Deserialize)]
#[loro(default)]
#[serde(default)]
pub struct Attributes {
    #[serde(skip_serializing_if = "MaybeMissing::is_missing")]
    pub asset: MaybeMissing<asset::AssetAttr>,
    #[serde(skip_serializing_if = "MaybeMissing::is_missing")]
    pub collider: MaybeMissing<collider::ColliderAttr>,
    #[serde(skip_serializing_if = "MaybeMissing::is_missing")]
    pub image: MaybeMissing<image::ImageAttr>,
    #[serde(skip_serializing_if = "MaybeMissing::is_missing")]
    pub material: MaybeMissing<material::MaterialAttr>,
    #[serde(skip_serializing_if = "MaybeMissing::is_missing")]
    pub mesh: MaybeMissing<mesh::MeshAttr>,
    #[serde(skip_serializing_if = "MaybeMissing::is_missing")]
    pub name: MaybeMissing<name::NameAttr>,
    #[serde(skip_serializing_if = "MaybeMissing::is_missing")]
    pub rigid_body: MaybeMissing<rigid_body::RigidBodyAttr>,
    #[serde(skip_serializing_if = "MaybeMissing::is_missing")]
    pub script: MaybeMissing<script::ScriptAttr>,
    #[serde(skip_serializing_if = "MaybeMissing::is_missing")]
    pub xform: MaybeMissing<xform::XformAttr>,
}

impl Attributes {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.asset.is_missing()
            && self.collider.is_missing()
            && self.image.is_missing()
            && self.material.is_missing()
            && self.mesh.is_missing()
            && self.name.is_missing()
            && self.rigid_body.is_missing()
            && self.script.is_missing()
            && self.xform.is_missing()
    }
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
