use std::{borrow::Cow, collections::HashMap};

use loro::{LoroMapValue, LoroValue};

pub fn merge_values<'a>(prev: Cow<'a, LoroValue>, next: &'a LoroValue) -> Cow<'a, LoroValue> {
    match (prev, next) {
        (Cow::Borrowed(LoroValue::Map(p)), LoroValue::Map(n)) => {
            let out = merge_maps(p, n);
            Cow::Owned(out)
        }
        (Cow::Owned(LoroValue::Map(p)), LoroValue::Map(n)) => {
            let out = merge_maps(&p, n);
            Cow::Owned(out)
        }
        (_, n) => Cow::Borrowed(n),
    }
}

fn merge_maps(prev: &LoroMapValue, next: &LoroMapValue) -> LoroValue {
    let mut cow_out = HashMap::new();

    for (key, next_value) in next.iter() {
        if let Some(prev_value) = prev.get(key) {
            // Merge conflicting values.
            let prev_value = Cow::Borrowed(prev_value);
            let merged_value = merge_values(prev_value, next_value);
            cow_out.insert(key, merged_value);
        } else {
            // Set new values.
            cow_out.insert(key, Cow::Borrowed(next_value));
        }
    }

    let mut prev = prev.clone();
    let prev_inner = prev.make_mut();

    for (key, value) in cow_out {
        prev_inner.insert(key.clone(), value.into_owned());
    }

    LoroValue::Map(prev)
}
