use std::collections::BTreeMap;

use anyhow::{
    Result,
    bail,
};
use secretspec::{
    Config,
    Profile,
};

/// Every value `profile` declares, applying the two manifest rules that decide
/// one: a profile inherits `default_profile` unless it opts out, and a
/// profile's own `defaults.default` stands in for the secrets that declare no
/// value of their own.
///
/// A secret with no value at all is absent, and reaches the binary as an
/// environment lookup rather than as a compiled-in value.
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

    let fallback = profile
        .defaults
        .as_ref()
        .and_then(|defaults| defaults.default.as_deref());

    for (name, secret) in &profile.secrets {
        if secret.composed.is_some() {
            bail!("secret '{name}' is composed, which only a runtime resolve can expand");
        }

        if let Some(value) = secret.default.as_deref().or(fallback) {
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

        [profiles.filled]
        defaults = { default = "stand-in" }
        TOKEN = { required = false }
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
    fn a_profile_default_stands_in_for_an_undeclared_value() {
        assert_eq!(
            defaults_for("filled").get("TOKEN").map(String::as_str),
            Some("stand-in")
        );
    }
}
