use std::{
    path::{Path, PathBuf},
    process::Command,
};

use unavi_constants::assets::WASM_ASSETS_DIR;

const ASSETS_DIR: &str = "assets";
const WASM_PROFILE: &str = "release-wasm";
const WASM_TARGET: &str = "wasm32-wasip1";

fn main() {
    let wasm_dir: PathBuf = ["..", "..", "wasm"].iter().collect();

    // Build components.
    let wasm_out = PathBuf::from(ASSETS_DIR).join(WASM_ASSETS_DIR);
    std::fs::create_dir_all(wasm_out.clone()).unwrap();
    std::fs::remove_dir_all(wasm_out.clone()).unwrap();
    std::fs::create_dir_all(wasm_out.clone()).unwrap();

    for entry in std::fs::read_dir(&wasm_dir).expect("Failed to read directory") {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();

        if path.is_dir() {
            println!("Building {:?}", path);

            let output = Command::new("cargo")
                .args([
                    "component",
                    "build",
                    "--profile",
                    WASM_PROFILE,
                    "--target",
                    WASM_TARGET,
                ])
                .current_dir(&path)
                .output()
                .expect("Failed to buld with cargo-component");

            if !output.status.success() {
                panic!(
                    "Building the subdirectory failed: {:?}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }

            let subdir_name = path.file_name().unwrap().to_str().unwrap();
            let mut wasm_name = subdir_name.replace('-', "_");
            wasm_name.push_str(".wasm");

            let dst = wasm_out.join(Path::new(&wasm_name));

            let path: PathBuf = ["..", "..", "target", WASM_TARGET, WASM_PROFILE, &wasm_name]
                .iter()
                .collect();
            std::fs::copy(path, &dst).expect("Failed to copy the WASM file {}");
        }
    }

    // Compose components.
    // We must manually specify the order of composition via this array.
    for name in [
        "unavi_vscreen.wasm",
        "unavi_ui.wasm",
        "unavi_shapes.wasm",
        "unavi_scene.wasm",
        "unavi_layout.wasm",
    ] {
        let path_a: PathBuf = [wasm_out.to_str().unwrap(), name].iter().collect();

        for entry_b in std::fs::read_dir(wasm_out.clone()).expect("Failed to read directory") {
            let entry_b = entry_b.expect("Failed to read directory entry");
            let path_b = entry_b.path();

            let path_a = path_a.as_os_str().to_str().unwrap();
            let path_b = path_b.as_os_str().to_str().unwrap();

            let output = Command::new("wac")
                .args(["plug", "--plug", path_a, path_b])
                .output()
                .expect("Failed to run wac plug");

            if output.status.success() {
                println!("Plugged {} -> {}", path_a, path_b);
                std::fs::write(path_b, output.stdout).unwrap();
            } else if String::from_utf8_lossy(&output.stderr).contains("no matching imports") {
                continue;
            } else {
                panic!("{}", String::from_utf8_lossy(&output.stderr));
            }
        }
    }

    // Organize assets dir.
    for entry in std::fs::read_dir(wasm_out.clone()).expect("Failed to read directory") {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();
        let path_str = path.as_os_str().to_str().unwrap();

        let mut split = path_str.split('_');

        let namespace = split.next().unwrap();
        std::fs::create_dir_all(namespace).unwrap();

        let mut package = String::default();
        split.enumerate().for_each(|(i, s)| {
            if i > 0 {
                package.push('-');
            }

            package.push_str(s);
        });

        let new_path = Path::new(namespace).join(package);
        println!(
            "Moving {} -> {}",
            path.as_os_str().to_str().unwrap(),
            new_path.as_os_str().to_str().unwrap()
        );

        std::fs::copy(path.clone(), new_path).unwrap();
        std::fs::remove_file(path).unwrap();
    }

    println!("cargo:rerun-if-changed={}", wasm_dir.to_str().unwrap());
    println!(
        "cargo:rustc-env=TARGET={}",
        std::env::var("TARGET").unwrap()
    );
}
