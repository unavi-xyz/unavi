use ron::extensions::Extensions;

use super::ShaderGraph;

fn ron_options() -> ron::Options {
    ron::Options::default()
        .with_default_extension(Extensions::IMPLICIT_SOME)
        .with_default_extension(Extensions::UNWRAP_NEWTYPES)
}

/// Parses `.hss` authoring source into a [`ShaderGraph`]: the authoritative
/// RON surface for the format.
pub fn parse(src: &str) -> Result<ShaderGraph, ron::error::SpannedError> {
    ron_options().from_str(src)
}
