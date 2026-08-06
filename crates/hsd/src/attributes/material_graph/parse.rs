use ron::extensions::Extensions;

use super::ShaderGraph;

fn ron_options() -> ron::Options {
    ron::Options::default()
        .with_default_extension(Extensions::IMPLICIT_SOME)
        .with_default_extension(Extensions::UNWRAP_NEWTYPES)
}

/// Parses `.hss` authoring source into a [`ShaderGraph`].
///
/// The authoritative RON surface for the format: every graph reaches a
/// material through [`Self::encode`]'d slot content, but authors write `.hss`
/// and the test suite exercises this syntax directly.
pub fn parse(src: &str) -> Result<ShaderGraph, ron::error::SpannedError> {
    ron_options().from_str(src)
}
