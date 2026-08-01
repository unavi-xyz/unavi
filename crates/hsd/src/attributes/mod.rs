use loro::{
    Container,
    LoroMap,
    ValueOrContainer,
};
use loro_surgeon::{
    Hydrate,
    Reconcile,
    error::{
        HydrateError,
        ReconcileError,
    },
    reconcile::PropReconciler,
};
use serde::{
    Deserialize,
    Serialize,
};

pub mod asset;
pub mod collider;
pub mod gravity_scale;
pub mod image;
pub mod material;
pub mod mesh;
pub mod name;
pub mod portal;
pub mod rigid_body;
pub mod script;
pub mod spawn;
pub mod subdocument;
pub mod xform;

pub const ATTRIBUTES_KEY: &str = "attributes";
pub const RELATIONSHIPS_KEY: &str = "relationships";

pub trait Attribute: Reconcile + Hydrate {
    const KEY: &str;

    /// Hydrate this attribute from the inner attributes map (i.e. the map
    /// returned by [`attributes_map`]).
    fn attr_hydrate(attrs: &LoroMap) -> Result<Self, HydrateError> {
        loro_surgeon::hydrate::hydrate_prop(attrs, Self::KEY)
    }

    fn attr_reconcile(&self, attrs: LoroMap) -> Result<(), ReconcileError> {
        let rec = PropReconciler::map_put(attrs, Self::KEY.to_string());
        self.reconcile(rec)
    }
}

#[serde_with::skip_serializing_none]
#[derive(Reconcile, Hydrate, Default, Clone, Serialize, Deserialize)]
#[loro(default)]
#[serde(default)]
pub struct Attributes {
    pub asset:         Option<asset::AssetAttr>,
    pub collider:      Option<collider::ColliderAttr>,
    pub gravity_scale: Option<gravity_scale::GravityScaleAttr>,
    pub image:         Option<image::ImageAttr>,
    pub material:      Option<material::MaterialAttr>,
    pub mesh:          Option<mesh::MeshAttr>,
    pub name:          Option<name::NameAttr>,
    pub portal:        Option<portal::PortalAttr>,
    pub rigid_body:    Option<rigid_body::RigidBodyAttr>,
    pub script:        Option<script::ScriptAttr>,
    pub spawn:         Option<spawn::SpawnAttr>,
    pub subdocument:   Option<subdocument::SubdocumentAttr>,
    pub xform:         Option<xform::XformAttr>,
}

impl Attributes {
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.asset.is_none()
            && self.collider.is_none()
            && self.gravity_scale.is_none()
            && self.image.is_none()
            && self.material.is_none()
            && self.mesh.is_none()
            && self.name.is_none()
            && self.portal.is_none()
            && self.rigid_body.is_none()
            && self.script.is_none()
            && self.spawn.is_none()
            && self.subdocument.is_none()
            && self.xform.is_none()
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
