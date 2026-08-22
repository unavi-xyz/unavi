fn main() {
    println!("cargo:rerun-if-changed=../../assets/icon-nobg.png");
    std::fs::copy("../../assets/icon-nobg.png", "assets/icon-nobg.png")
        .expect("copy icon-nobg.png");
}
