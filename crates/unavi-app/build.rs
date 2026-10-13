fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").expect("CARGO_CFG_TARGET_OS not set") == "windows" {
        let _ = embed_resource::compile("../../assets/icon-rounded.rc", embed_resource::NONE);
    }
}
