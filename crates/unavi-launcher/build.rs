fn main() {
    // Only embed icon on Windows targets.
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        embed_resource::compile(
            "../../assets/unavi-rounded-inverse.rc",
            embed_resource::NONE,
        );
    }
}
