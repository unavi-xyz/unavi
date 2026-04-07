use rand::{Rng, distr::Alphanumeric};
use smol_str::SmolStr;

/// Max byte length for an inline [`SmolStr`].
const MAX_INLINE: usize = 23;

pub fn gen_id() -> SmolStr {
    rand::rng()
        .sample_iter(Alphanumeric)
        .take(MAX_INLINE)
        .map(char::from)
        .collect::<SmolStr>()
}
