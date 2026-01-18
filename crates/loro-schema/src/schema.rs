//! Schema types for Loro document validation.

use std::collections::BTreeMap;

use blake3::Hash;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

/// Schema defining how to validate a Loro container.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Schema {
    id: SmolStr,
    version: u32,
    /// Allowed layout of the data.
    layout: Field,
}

impl Schema {
    /// Serialize to bytes using postcard.
    ///
    /// # Errors
    ///
    /// Errors if postcard could not serialize the struct.
    pub fn to_bytes(&self) -> postcard::Result<Vec<u8>> {
        postcard::to_stdvec(self)
    }

    /// Deserialize from bytes using postcard.
    ///
    /// # Errors
    ///
    /// Errors if postcard could not deserialize the struct.
    pub fn from_bytes(bytes: &[u8]) -> postcard::Result<Self> {
        postcard::from_bytes(bytes)
    }

    /// Compute content-addressed ID.
    ///
    /// # Errors
    ///
    /// Errors if postcard could not serialize the struct.
    pub fn id(&self) -> postcard::Result<Hash> {
        let bytes = self.to_bytes()?;
        Ok(blake3::hash(&bytes))
    }

    /// Get the root layout field.
    #[must_use]
    pub const fn layout(&self) -> &Field {
        &self.layout
    }
}

/// Field type definitions for schema validation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Field {
    /// Accept any value.
    Any,
    /// Binary data (Vec<u8>).
    Binary,
    /// Boolean value.
    Bool,
    /// 64-bit floating point.
    F64,
    /// 64-bit signed integer.
    I64,
    /// Ordered list of homogeneous items.
    List(Box<Self>),
    /// Homogeneous map: any string keys, all values must match inner type.
    Map(Box<Self>),
    /// List with reorder/move support.
    MovableList(Box<Self>),
    /// Optional value (can be null).
    Optional(Box<Self>),
    /// ACL-restricted field with authorization rules.
    Restricted {
        actions: Vec<Action>,
        value: Box<Self>,
    },
    /// String value.
    String,
    /// Fixed keys with heterogeneous types per field.
    Struct(BTreeMap<SmolStr, Box<Self>>),
    /// Tree structure: each node's metadata must match inner type.
    Tree(Box<Self>),
}

/// An authorization action specifying who can perform what operations.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Action {
    /// Who is authorized.
    pub who: Who,
    /// What operations they can perform.
    pub can: Vec<Can>,
}

/// Specifies who is authorized for an action.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Who {
    /// Anyone can perform the action.
    Anyone,
    /// Authorization is determined by a path reference (e.g., "acl.manage").
    Path(SmolStr),
}

/// Types of operations that can be authorized.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Can {
    /// Create new values.
    Create,
    /// Delete existing values.
    Delete,
    /// Update existing values.
    Update,
}
