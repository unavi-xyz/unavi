fn main() {
    let target = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    if target != "wasm32" {
        return;
    }

    let dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();

    println!("cargo:rerun-if-changed={dir}/runtime.ts");
    println!("cargo:rerun-if-changed={dir}/package.json");

    let status = std::process::Command::new("npm")
        .args(["install", "--prefix", &dir, "--silent"])
        .status()
        .expect("npm install failed");
    assert!(status.success(), "npm install failed");

    let input = format!("{dir}/runtime.ts");
    let outfile = format!("{dir}/dist/runtime.js");
    let status = std::process::Command::new("esbuild")
        .args([
            &input,
            "--bundle",
            "--format=esm",
            "--platform=browser",
            "--external:node:fs/promises",
            &format!("--outfile={outfile}"),
        ])
        .status()
        .expect("esbuild failed");
    assert!(status.success(), "esbuild failed");
}
