//! `ToInlineValue` — convert scalar Rust values to inline [`LoroValue`]s.
//!
//! Atomic `[T; N]` arrays write a single `LoroValue::List` via this trait,
//! so the element type must be representable as a non-container `LoroValue`.

use loro::LoroValue;

pub trait ToInlineValue {
    fn to_inline(&self) -> LoroValue;
}

impl ToInlineValue for bool {
    fn to_inline(&self) -> LoroValue {
        LoroValue::Bool(*self)
    }
}

macro_rules! impl_inline_int_from {
    ($($t:ty),*) => {
        $(
            impl ToInlineValue for $t {
                fn to_inline(&self) -> LoroValue {
                    LoroValue::I64(i64::from(*self))
                }
            }
        )*
    };
}

impl_inline_int_from!(i8, i16, i32, i64, u8, u16, u32);

// u64/usize don't fit into `From<_> for i64`; the cast wraps above i64::MAX.
#[allow(clippy::cast_possible_wrap)]
impl ToInlineValue for u64 {
    fn to_inline(&self) -> LoroValue {
        LoroValue::I64(*self as i64)
    }
}

#[allow(clippy::cast_possible_wrap)]
impl ToInlineValue for usize {
    fn to_inline(&self) -> LoroValue {
        LoroValue::I64(*self as i64)
    }
}

impl ToInlineValue for f32 {
    fn to_inline(&self) -> LoroValue {
        LoroValue::Double(f64::from(*self))
    }
}

impl ToInlineValue for f64 {
    fn to_inline(&self) -> LoroValue {
        LoroValue::Double(*self)
    }
}

impl ToInlineValue for String {
    fn to_inline(&self) -> LoroValue {
        LoroValue::String(self.as_str().into())
    }
}

impl ToInlineValue for &str {
    fn to_inline(&self) -> LoroValue {
        LoroValue::String((*self).into())
    }
}

impl<T: ToInlineValue> ToInlineValue for Option<T> {
    fn to_inline(&self) -> LoroValue {
        self.as_ref().map_or(LoroValue::Null, ToInlineValue::to_inline)
    }
}

impl<T: ToInlineValue> ToInlineValue for &T {
    fn to_inline(&self) -> LoroValue {
        (*self).to_inline()
    }
}
