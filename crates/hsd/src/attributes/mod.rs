use serde::{
    Serialize,
    de::DeserializeOwned,
};

pub mod collider;
pub mod gravity_scale;
pub mod image;
pub mod material;
pub mod material_graph;
pub mod mesh;
pub mod name;
pub mod portal;
pub mod rigid_body;
pub mod spawn;
pub mod xform;

/// A postcard payload under a string key.
///
/// The set is open: a new kind is one module that names itself, with no shared
/// struct to edit, and a payload no client recognizes still stores, syncs and
/// re-serves untouched.
pub trait Attribute: Serialize + DeserializeOwned {
    const KEY: &'static str;

    fn encode(&self) -> Result<Vec<u8>, postcard::Error> {
        postcard::to_stdvec(self)
    }

    fn decode(bytes: &[u8]) -> Result<Self, postcard::Error> {
        postcard::from_bytes(bytes)
    }
}

/// Bulk slots, the `b/<prim>/<slot>/` entries whose value is the data itself.
pub mod slots {
    pub const PREFAB: &str = "prefab";
    pub const SCRIPT: &str = "script";
    pub const IMAGE_DATA: &str = "image:data";
    pub const MESH_INDICES: &str = "mesh:indices";
    pub const COLLIDER_INDICES: &str = "collider:indices";
    pub const COLLIDER_VERTICES: &str = "collider:vertices";
    /// The compiled, validated node graph. Bulk, not an attribute payload —
    /// see `material_graph`'s module docs for why.
    pub const MATERIAL_GRAPH_DATA: &str = "material:graph_data";
    /// One relationship per fixed texture-sample slot a graph may use.
    #[must_use]
    pub fn material_graph_texture(slot: u8) -> String {
        format!("material:graph_texture:{slot}")
    }

    #[must_use]
    pub fn mesh_attribute(name: &str) -> String {
        format!("mesh:{name}")
    }

    #[must_use]
    pub fn mesh_attribute_name(slot: &str) -> Option<&str> {
        let name = slot.strip_prefix("mesh:")?;
        (name != "indices").then_some(name)
    }
}
