use std::fmt::Debug;

use serde::{
    Serialize,
    de::DeserializeOwned,
};

pub mod name;
pub mod value;

use crate::property::name::PropName;

/// A postcard-encoded field value.
pub trait Payload: Serialize + DeserializeOwned {
    fn encode(&self) -> Result<Vec<u8>, postcard::Error> {
        postcard::to_stdvec(self)
    }

    fn decode(bytes: &[u8]) -> Result<Self, postcard::Error> {
        postcard::from_bytes(bytes)
    }
}

impl<T: Serialize + DeserializeOwned> Payload for T {}

/// A payload addressable at a fixed name.
pub trait Property: Payload + Debug {
    const NAME: PropName;

    /// A debug view of one stored payload. A property holding bulk bytes
    /// overrides this to summarize rather than print the buffer.
    #[must_use]
    fn render(payload: &[u8]) -> String {
        Self::decode(payload).map_or_else(
            |err| format!("<undecodable: {err}>"),
            |value| format!("{value:?}"),
        )
    }
}

/// Renders a bulk payload as its size, the debug view for buffer fields.
#[must_use]
pub fn render_bytes(len: usize) -> String {
    format!("<{len} bytes>")
}

/// Defines a relationship property: a marker type plus the `PropName` const
/// that names it. The caller must have `Serialize` and `Deserialize` in scope.
#[macro_export]
macro_rules! relationship {
    ($(#[$meta:meta])* $ty:ident => $konst:ident, $path:expr) => {
        $(#[$meta])*
        #[derive(
            Debug,
            Clone,
            Copy,
            Default,
            PartialEq,
            Eq,
            Serialize,
            Deserialize,
        )]
        pub struct $ty;

        impl $crate::property::Property for $ty {
            const NAME: $crate::property::name::PropName = $crate::prop_name!($path);
        }

        pub const $konst: $crate::property::name::PropName =
            <$ty as $crate::property::Property>::NAME;
    };
}
