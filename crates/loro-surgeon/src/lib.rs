//! Typed serialization/deserialization for Loro CRDT documents.
//!
//! Similar to what `autosurgeon` does for `automerge`, this crate provides
//! derive macros and traits for converting Rust types to/from Loro containers.
//!
//! # Example
//!
//! ```
//! use loro::LoroDoc;
//! use loro_surgeon::{Hydrate, Reconcile};
//!
//! #[derive(Hydrate, Reconcile, Debug, PartialEq)]
//! struct User {
//!     name: String,
//!     age: i64,
//! }
//!
//! let doc = LoroDoc::new();
//! let user = User { name: "Alice".into(), age: 30 };
//! user.reconcile(&doc.get_map("user")).unwrap();
//!
//! let loaded = User::hydrate(&doc.get_map("user").get_deep_value()).unwrap();
//! assert_eq!(user, loaded);
//! ```

mod error;
mod hydrate;
mod reconcile;

pub use error::{HydrateError, ReconcileError};
pub use hydrate::Hydrate;
pub use loro_surgeon_derive::{Hydrate, Reconcile};
pub use reconcile::Reconcile;

// Re-export loro for use by derive macros.
pub use loro;

#[cfg(test)]
mod tests {
    use loro::LoroDoc;

    use super::*;

    #[derive(Hydrate, Reconcile, Debug, PartialEq)]
    struct User {
        name: String,
        age: i64,
    }

    #[test]
    fn roundtrip_simple_struct() {
        let doc = LoroDoc::new();
        let user = User {
            name: "Alice".into(),
            age: 30,
        };
        user.reconcile(&doc.get_map("user"))
            .expect("reconcile failed");

        let loaded = User::hydrate(&doc.get_map("user").get_deep_value()).expect("hydrate failed");
        assert_eq!(user, loaded);
    }

    #[derive(Hydrate, Reconcile, Debug, PartialEq)]
    struct WithOptional {
        required: String,
        optional: Option<i64>,
    }

    #[test]
    fn roundtrip_with_optional_present() {
        let doc = LoroDoc::new();
        let data = WithOptional {
            required: "test".into(),
            optional: Some(42),
        };
        data.reconcile(&doc.get_map("data"))
            .expect("reconcile failed");

        let loaded =
            WithOptional::hydrate(&doc.get_map("data").get_deep_value()).expect("hydrate failed");
        assert_eq!(data, loaded);
    }

    #[test]
    fn roundtrip_with_optional_none() {
        let doc = LoroDoc::new();
        let data = WithOptional {
            required: "test".into(),
            optional: None,
        };
        data.reconcile(&doc.get_map("data"))
            .expect("reconcile failed");

        let loaded =
            WithOptional::hydrate(&doc.get_map("data").get_deep_value()).expect("hydrate failed");
        assert_eq!(data, loaded);
    }

    #[derive(Hydrate, Reconcile, Debug, PartialEq)]
    struct WithBinary {
        name: String,
        data: Vec<u8>,
    }

    #[test]
    fn roundtrip_with_binary() {
        let doc = LoroDoc::new();
        let data = WithBinary {
            name: "test".into(),
            data: vec![1, 2, 3, 4, 5],
        };
        data.reconcile(&doc.get_map("data"))
            .expect("reconcile failed");

        let loaded =
            WithBinary::hydrate(&doc.get_map("data").get_deep_value()).expect("hydrate failed");
        assert_eq!(data, loaded);
    }

    #[derive(Hydrate, Reconcile, Debug, PartialEq)]
    struct WithRename {
        #[loro(rename = "user_name")]
        name: String,
    }

    #[test]
    fn roundtrip_with_rename() {
        let doc = LoroDoc::new();
        let data = WithRename {
            name: "Alice".into(),
        };
        data.reconcile(&doc.get_map("data"))
            .expect("reconcile failed");

        // Verify the renamed key exists.
        let deep_value = doc.get_map("data").get_deep_value();
        let loro::LoroValue::Map(map) = &deep_value else {
            panic!("expected map");
        };
        assert!(map.contains_key("user_name"));
        assert!(!map.contains_key("name"));

        let loaded = WithRename::hydrate(&deep_value).expect("hydrate failed");
        assert_eq!(data, loaded);
    }
}
