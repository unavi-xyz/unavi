use loro::LoroValue;

use crate::{
    error::ReconcileError,
    hydrate::Hydrate,
    inline::ToInlineValue,
    reconcile::{
        LoadKey,
        NoKey,
        Reconcile,
        Reconciler,
    },
};

impl Reconcile for bool {
    type Key = NoKey;
    fn reconcile<R: Reconciler>(&self, r: R) -> Result<(), ReconcileError> {
        r.boolean(*self)
    }
}

macro_rules! impl_reconcile_int_from {
    ($($t:ty),*) => {
        $(
            impl Reconcile for $t {
                type Key = NoKey;
                fn reconcile<R: Reconciler>(&self, r: R) -> Result<(), ReconcileError> {
                    r.i64(i64::from(*self))
                }
            }
        )*
    };
}

impl_reconcile_int_from!(i8, i16, i32, i64, u8, u16, u32);

// u64/usize don't fit into `From<_> for i64`; the cast wraps above i64::MAX.
#[allow(clippy::cast_possible_wrap)]
impl Reconcile for u64 {
    type Key = NoKey;
    fn reconcile<R: Reconciler>(&self, r: R) -> Result<(), ReconcileError> {
        r.i64(*self as i64)
    }
}

#[allow(clippy::cast_possible_wrap)]
impl Reconcile for usize {
    type Key = NoKey;
    fn reconcile<R: Reconciler>(&self, r: R) -> Result<(), ReconcileError> {
        r.i64(*self as i64)
    }
}

impl Reconcile for f64 {
    type Key = NoKey;
    fn reconcile<R: Reconciler>(&self, r: R) -> Result<(), ReconcileError> {
        r.f64(*self)
    }
}

impl Reconcile for f32 {
    type Key = NoKey;
    fn reconcile<R: Reconciler>(&self, r: R) -> Result<(), ReconcileError> {
        r.f64(f64::from(*self))
    }
}

impl Reconcile for String {
    type Key = NoKey;
    fn reconcile<R: Reconciler>(&self, r: R) -> Result<(), ReconcileError> {
        r.str(self)
    }
}

impl Reconcile for &str {
    type Key = NoKey;
    fn reconcile<R: Reconciler>(&self, r: R) -> Result<(), ReconcileError> {
        r.str(self)
    }
}

impl<T: Reconcile> Reconcile for Option<T> {
    type Key = NoKey;
    fn reconcile<R: Reconciler>(&self, r: R) -> Result<(), ReconcileError> {
        match self {
            Some(v) => v.reconcile(r),
            None => r.null(),
        }
    }
}

impl<T> Reconcile for Vec<T>
where
    T: Reconcile + Hydrate + PartialEq,
{
    type Key = NoKey;
    fn reconcile<R: Reconciler>(&self, r: R) -> Result<(), ReconcileError> {
        super::list::reconcile_vec(self, r)
    }
}

impl<T: ToInlineValue, const N: usize> Reconcile for [T; N] {
    type Key = NoKey;
    fn reconcile<R: Reconciler>(&self, r: R) -> Result<(), ReconcileError> {
        let items: Vec<LoroValue> = self.iter().map(ToInlineValue::to_inline).collect();
        r.inline_list(LoroValue::from(items))
    }
}

impl Reconcile for LoroValue {
    type Key = NoKey;
    fn reconcile<R: Reconciler>(&self, r: R) -> Result<(), ReconcileError> {
        match self {
            Self::Null => r.null(),
            Self::Bool(b) => r.boolean(*b),
            Self::I64(i) => r.i64(*i),
            Self::Double(f) => r.f64(*f),
            Self::String(s) => r.str(s),
            Self::Binary(b) => r.bytes(b),
            Self::List(_) => r.inline_list(self.clone()),
            Self::Map(entries) => {
                let mut map_r = r.map()?;
                for (k, v) in entries.iter() {
                    map_r.entry(k, v)?;
                }
                let keep: std::collections::HashSet<&str> =
                    entries.keys().map(std::string::String::as_str).collect();
                map_r.retain(|k| keep.contains(k))?;
                Ok(())
            }
            Self::Container(_) => Err(ReconcileError::TypeMismatch {
                expected: "value",
                found:    "container ref",
            }),
        }
    }
}

impl Reconcile for serde_json::Value {
    type Key = NoKey;
    fn reconcile<R: Reconciler>(&self, r: R) -> Result<(), ReconcileError> {
        let s = serde_json::to_string(self)?;
        r.str(&s)
    }
}

impl<T: Reconcile> Reconcile for Box<T> {
    type Key = T::Key;
    fn reconcile<R: Reconciler>(&self, r: R) -> Result<(), ReconcileError> {
        (**self).reconcile(r)
    }
    fn key(&self) -> LoadKey<Self::Key> {
        (**self).key()
    }
}

impl<T: Reconcile + ?Sized> Reconcile for &T {
    type Key = T::Key;
    fn reconcile<R: Reconciler>(&self, r: R) -> Result<(), ReconcileError> {
        (*self).reconcile(r)
    }
    fn key(&self) -> LoadKey<Self::Key> {
        (*self).key()
    }
}
