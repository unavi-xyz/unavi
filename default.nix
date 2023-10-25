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
  native =
    rustPlatform.buildRustPackage (common // { pname = "unavi-native"; });

  server =
    rustPlatform.buildRustPackage (common // { pname = "unavi-server"; });

  web = rustPlatformWasm.buildRustPackage (common // {
    pname = "unavi-web";
    buildPhase = ''
      cargo leptos build --release
    '';
    installPhase = ''
      cp -r target $out/web
    '';
  });
}
