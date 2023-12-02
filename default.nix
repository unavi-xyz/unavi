{ lib, pkgs, system, build_inputs, native_build_inputs, makeRustPlatform }:
let
  rustBin = pkgs.rust-bin.selectLatestNightlyWith (toolchain:
    toolchain.default.override { targets = [ "wasm32-unknown-unknown" ]; });

  rustPlatform = makeRustPlatform {
    cargo = rustBin;
    rustc = rustBin;
  };

  common = {
    version = "0.0.0";
    src = ./.;
    cargoLock.lockFile = ./Cargo.lock;
    PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";

    buildInputs = build_inputs;
    nativeBuildInputs = native_build_inputs;

    LD_LIBRARY_PATH = lib.makeLibraryPath build_inputs;
    LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
  };
in {
  unavi_native =
    rustPlatform.buildRustPackage (common // { pname = "unavi-native"; });

  unavi_server =
    rustPlatform.buildRustPackage (common // { pname = "unavi-server"; });

  unavi_web = rustPlatform.buildRustPackage (common // {
    pname = "unavi-web";
    buildPhase = ''
      cargo leptos build --release
    '';
    installPhase = ''
      mkdir -p $out/web
      cp -r target $out/web
    '';
  });
}
