use std::{
    env,
    fs,
    path::Path,
};

use base64::Engine;

const LOGO: &str = "../../assets/icon-nobg.png";

fn main() {
    println!("cargo:rerun-if-changed={LOGO}");

    let png = fs::read(LOGO).expect("read logo");
    let encoded = base64::engine::general_purpose::STANDARD.encode(png);

    let out = Path::new(&env::var("OUT_DIR").expect("OUT_DIR")).join("logo.uri");
    fs::write(out, format!("data:image/png;base64,{encoded}")).expect("write logo data uri");
}
