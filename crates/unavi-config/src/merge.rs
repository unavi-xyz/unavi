//! Which fields a profile gives a value.

use std::collections::BTreeMap;

use anyhow::{
    Result,
    bail,
};
use secretspec::{
    Config,
    Profile,
};

/// Every value `profile` declares, inheriting `default_profile` unless it opts
/// out.
///
/// A field with no value is absent, reaching the binary as an environment
/// lookup rather than a compiled-in value. A profile-wide `defaults.default`
/// is refused: it would give every field a value, compiling in whatever the
/// build environment holds for each, secrets included.
pub fn defaults(
    config: &Config,
    profile: &str,
    default_profile: &str,
) -> Result<BTreeMap<String, String>> {
    let selected = config.profiles.get(profile);
    let mut values = BTreeMap::new();

    if profile != default_profile && inherits(selected) {
        collect(config.profiles.get(default_profile), &mut values)?;
    }
    collect(selected, &mut values)?;

    Ok(values)
}

fn inherits(profile: Option<&Profile>) -> bool {
    profile
        .and_then(|profile| profile.defaults.as_ref())
        .and_then(|defaults| defaults.inherit)
        .unwrap_or(true)
}

fn collect(profile: Option<&Profile>, values: &mut BTreeMap<String, String>) -> Result<()> {
    let Some(profile) = profile else {
        return Ok(());
    };

    if profile
        .defaults
        .as_ref()
        .is_some_and(|defaults| defaults.default.is_some())
    {
        bail!(
            "a profile-wide `defaults.default` would compile every field's build environment in; give each public field its own default"
        );
    }

    for (name, field) in &profile.secrets {
        if field.composed.is_some() {
            bail!("field '{name}' is composed, which only a runtime resolve can expand");
        }
        if field.required == Some(true) && field.default.is_some() {
            bail!("field '{name}' is required yet has a default; drop one");
        }

        if let Some(value) = field.default.as_deref() {
            values.insert(name.clone(), value.to_owned());
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    const MANIFEST: &str = r#"
        [project]
        name = "test"
        revision = "1.0"

        [profiles.default]
        HOST = { default = "remote", description = "d" }
        TOKEN = { required = false, description = "d" }

        [profiles.development]
        HOST = { default = "localhost" }

        [profiles.sealed]
        defaults = { inherit = false }
        HOST = { default = "sealed" }

    "#;

    fn defaults_for(profile: &str) -> BTreeMap<String, String> {
        let config = Config::from_str(MANIFEST).expect("parse manifest");
        defaults(&config, profile, "default").expect("merge profile")
    }

    #[test]
    fn a_secret_without_a_default_has_no_value() {
        assert_eq!(defaults_for("default").get("TOKEN"), None);
    }

    #[test]
    fn a_profile_overrides_only_what_it_declares() {
        assert_eq!(
            defaults_for("development").get("HOST").map(String::as_str),
            Some("localhost")
        );
    }

    #[test]
    fn opting_out_of_inheritance_drops_the_default_profile() {
        assert_eq!(defaults_for("sealed").keys().collect::<Vec<_>>(), ["HOST"]);
    }

    #[test]
    fn a_profile_wide_default_is_refused() {
        let manifest = format!(
            "{MANIFEST}\n[profiles.filled]\ndefaults = {{ default = \"stand-in\" }}\nTOKEN = {{ required = false }}\n"
        );
        let config = Config::from_str(&manifest).expect("parse manifest");
        assert!(defaults(&config, "filled", "default").is_err());
    }

    #[test]
    fn a_required_field_with_a_default_is_refused() {
        let manifest = r#"
            [project]
            name = "test"
            revision = "1.0"

            [profiles.default]
            HOST = { default = "remote", required = true, description = "d" }
        "#;
        let config = Config::from_str(manifest).expect("parse manifest");
        assert!(defaults(&config, "default", "default").is_err());
    }
}
