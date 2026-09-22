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
pub mod parent;
pub mod portal;
pub mod reference;
pub mod rigid_body;
pub mod script;
pub mod spawn;
pub mod text;
pub mod xform;

pub trait Attribute: Serialize + DeserializeOwned {
    const KEY: &'static str;

    fn encode(&self) -> Result<Vec<u8>, postcard::Error> {
        postcard::to_stdvec(self)
    }

    fn decode(bytes: &[u8]) -> Result<Self, postcard::Error> {
        postcard::from_bytes(bytes)
    }
}
