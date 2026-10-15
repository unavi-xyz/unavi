fn main() {
    let target = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    if target != "wasm32" {
        return;
    }

    let dir = std::env::var("CARGO_MANIFEST_DIR").expect("no cargo manifest dir");

    println!("cargo:rerun-if-changed={dir}/runtime.ts");
    println!("cargo:rerun-if-changed={dir}/package.json");
    println!("cargo:rerun-if-changed={dir}/package-lock.json");

    // Deterministic from the lockfile, unlike `npm install`: it never
    // resolves a newer version and never reaches the network once
    // `node_modules` matches `package-lock.json`.
    let status = std::process::Command::new("npm")
        .args(["ci", "--prefix", &dir, "--silent"])
        .status()
        .expect("npm ci failed");
    assert!(status.success(), "npm ci failed");

    let input = format!("{dir}/runtime.ts");
    let outfile = format!("{dir}/dist/runtime.js");
    let esbuild = format!("{dir}/node_modules/.bin/esbuild");
    let status = std::process::Command::new(esbuild)
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
