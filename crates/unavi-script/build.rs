fn main() {
    let target = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    if target != "wasm32" {
        return;
    }

    let dir = std::env::var("CARGO_MANIFEST_DIR").expect("no cargo manifest dir");

    println!("cargo:rerun-if-changed={dir}/runtime.ts");
    println!("cargo:rerun-if-changed={dir}/transpile-worker.ts");
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

    // `runtime.js` is the wasm-bindgen snippet. `transpile-worker.js` must sit
    // beside it so `import.meta.url` in
    // `new Worker(new URL("./transpile-worker.js", import.meta.url))`
    // resolves in the served snippet directory.
    let esbuild = format!("{dir}/node_modules/.bin/esbuild");
    let status = std::process::Command::new(esbuild)
        .args([
            &format!("{dir}/runtime.ts"),
            &format!("{dir}/transpile-worker.ts"),
            "--bundle",
            "--format=esm",
            "--platform=browser",
            "--external:node:fs/promises",
            &format!("--outdir={dir}/dist"),
        ])
        .status()
        .expect("esbuild failed");
    assert!(status.success(), "esbuild failed");

    // The worker is a second entry point, so a build that drops it stays
    // green while the browser 404s it. Fail here instead.
    for output in ["runtime.js", "transpile-worker.js"] {
        let path = format!("{dir}/dist/{output}");
        assert!(
            std::path::Path::new(&path).exists(),
            "esbuild did not emit {output}"
        );
    }
}
