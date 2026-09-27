use const_format::concatcp;
use serde::{
    Deserialize,
    Serialize,
};

use crate::{
    prop_name,
    property::{
        Payload,
        Property,
        name::{
            PropName,
            PropNameError,
        },
        render_bytes,
    },
};

pub const GROUP: &str = "mesh";

pub(crate) const STREAM_PREFIX: &str = "stream:";

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Topology {
    PointList,
    LineList,
    LineStrip,
    #[default]
    TriangleList,
    TriangleStrip,
}

impl Property for Topology {
    const NAME: PropName = prop_name!(concatcp!(GROUP, "/topology"));
}

/// Raw little-endian `u32` indices.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MeshIndices(pub Vec<u8>);

impl Property for MeshIndices {
    const NAME: PropName = prop_name!(concatcp!(GROUP, "/indices"));

    fn render(payload: &[u8]) -> String {
        match Self::decode(payload) {
            Ok(value) => render_bytes(value.0.len()),
            Err(err) => format!("<undecodable: {err}>"),
        }
    }
}

/// Raw little-endian `f32` vertex data, stored at [`stream`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MeshStream(pub Vec<u8>);

/// The field holding the vertex stream `stream` (`POSITION`, `NORMAL`,
/// `UV_0`, ...).
pub fn stream(stream: &str) -> Result<PropName, PropNameError> {
    PropName::new(GROUP, &format!("{STREAM_PREFIX}{stream}"))
}

/// The stream a mesh field holds, if it holds one.
#[must_use]
pub fn stream_of(name: &PropName) -> Option<&str> {
    if name.group() != GROUP {
        return None;
    }
    name.field()?.strip_prefix(STREAM_PREFIX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_stream_field_names_its_stream() {
        let name = stream("POSITION").expect("valid");
        assert_eq!(name.as_str(), "mesh/stream:POSITION");
        assert_eq!(stream_of(&name), Some("POSITION"));
        assert_eq!(stream_of(&Topology::NAME), None);
    }
}
