fn main() {
    println!("cargo:rerun-if-changed=../../assets/icon-nobg.png");
    std::fs::copy("../../assets/icon-nobg.png", "assets/icon-nobg.png")
        .expect("copy icon-nobg.png");

    if std::env::var("CARGO_CFG_TARGET_OS").expect("CARGO_CFG_TARGET_OS not set") == "windows" {
        let _ = embed_resource::compile("../../assets/icon-rounded.rc", embed_resource::NONE);
    }
}
