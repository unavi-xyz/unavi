use std::fmt;

use serde::{
    Deserialize,
    Serialize,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(transparent)]
pub struct HydratedDid(pub String);

impl fmt::Display for HydratedDid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(feature = "loro")]
mod xdid_impls {
    use std::str::FromStr;

    use xdid::core::did::Did;

    use super::HydratedDid;

    impl From<Did> for HydratedDid {
        fn from(d: Did) -> Self {
            Self(d.to_string())
        }
    }

    impl From<&Did> for HydratedDid {
        fn from(d: &Did) -> Self {
            Self(d.to_string())
        }
    }

    impl PartialEq<Did> for HydratedDid {
        fn eq(&self, other: &Did) -> bool {
            self.0 == other.to_string()
        }
    }

    impl HydratedDid {
        /// Parse the inner string back into an `xdid::Did`.
        pub fn parse(&self) -> Result<Did, <Did as FromStr>::Err> {
            Did::from_str(&self.0)
        }
    }
}

#[cfg(feature = "loro")]
mod loro_impls {
    use loro_surgeon::{
        Hydrate,
        Reconcile,
        error::{
            HydrateError,
            ReconcileError,
        },
        reconcile::{
            NoKey,
            Reconciler,
        },
    };

    use super::HydratedDid;

    impl Hydrate for HydratedDid {
        fn hydrate_string(s: &str) -> Result<Self, HydrateError> {
            Ok(Self(s.to_string()))
        }
    }

    impl Reconcile for HydratedDid {
        type Key = NoKey;

        fn reconcile<R: Reconciler>(&self, r: R) -> Result<(), ReconcileError> {
            self.0.reconcile(r)
        }
    }
}
