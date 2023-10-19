{ lib, pkgs, system, build_inputs, native_build_inputs, makeRustPlatform }:
let
  wasmTarget = "wasm32-unknown-unknown";

  rustBin = pkgs.rust-bin.stable.latest.default;
  rustBinWasm = rustBin.override { targets = [ wasmTarget ]; };

  rustPlatform = makeRustPlatform {
    cargo = rustBin;
    rustc = rustBin;
  };

  rustPlatformWasm = makeRustPlatform {
    cargo = rustBinWasm;
    rustc = rustBinWasm;
  };

  common = {
    version = "0.0.0";
    src = ./.;
    cargoLock.lockFile = ./Cargo.lock;
    PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";

    buildInputs = build_inputs;
    nativeBuildInputs = native_build_inputs;

    LD_LIBRARY_PATH = lib.makeLibraryPath build_inputs;
  };
in {
  app = rustPlatform.buildRustPackage (common // { pname = "unavi-app"; });

  wasm = rustPlatformWasm.buildRustPackage (common // {
    pname = "unavi-wasm";

    buildPhase = ''
      cargo build --release --target=${wasmTarget} -p unavi-wasm
    '';

    installPhase = ''
      mkdir -p $out/lib
      cp target/${wasmTarget}/release/*.wasm $out/lib/
      wasm-bindgen target/${wasmTarget}/release/unavi_wasm.wasm --out-dir $out/pkg --browser --weak-refs --reference-types
    '';
  });
}
