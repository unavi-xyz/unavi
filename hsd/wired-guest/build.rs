use std::{
    env,
    error::Error,
    fs,
    path::Path,
};

#[path = "../../assets/tokens_gen.rs"]
mod tokens_gen;

/// Relative to the crate root for a local checkout.
const DEFAULT_ASSETS: &str = "../../assets";

fn main() -> Result<(), Box<dyn Error>> {
    let assets = env::var("UNAVI_ASSETS").unwrap_or_else(|_| DEFAULT_ASSETS.to_string());
    let assets = Path::new(&assets);
    let css = assets.join("tokens.css");
    let toml = assets.join("tokens.toml");
    println!("cargo:rerun-if-changed={}", css.display());
    println!("cargo:rerun-if-changed={}", toml.display());
    println!("cargo:rerun-if-env-changed=UNAVI_ASSETS");

    let module = tokens_gen::generate(&fs::read_to_string(css)?, &fs::read_to_string(toml)?)?;
    let out_dir = env::var("OUT_DIR")?;
    fs::write(Path::new(&out_dir).join("tokens.rs"), module)?;
    Ok(())
}
