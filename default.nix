{ lib, pkgs, system, build_inputs, native_build_inputs, makeRustPlatform }:
let
  rustBin = pkgs.rust-bin.stable.latest.default.override {
    targets = [ "wasm32-unknown-unknown" ];
  };

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
  unavi-app = rustPlatform.buildRustPackage (common // {
    pname = "unavi-app";
    buildAndTestSubdir = "crates/unavi-app";
    postInstall = ''
      cp -r ./assets $out/bin
    '';
  });

  unavi-server = rustPlatform.buildRustPackage (common // {
    pname = "unavi-server";
    buildAndTestSubdir = "crates/unavi-server";
  });

  unavi-system = rustPlatform.buildRustPackage (common // {
    pname = "unavi-system";
    buildAndTestSubdir = "wasm/unavi-system";
    cargoBuildFlags = "--target wasm32-unknown-unknown";
  });

  unavi-ui = rustPlatform.buildRustPackage (common // {
    pname = "unavi-ui";
    buildAndTestSubdir = "wasm/unavi-ui";
    cargoBuildFlags = "--target wasm32-unknown-unknown";
  });

  wired-script = rustPlatform.buildRustPackage (common // {
    pname = "wired-script";
    buildAndTestSubdir = "wasm/wired-script";
    cargoBuildFlags = "--target wasm32-unknown-unknown";
  });

  web = rustPlatform.buildRustPackage (common // {
    pname = "web";
    buildAndTestSubdir = "crates/unavi-app";
    cargoBuildFlags = "--target wasm32-unknown-unknown";
    buildPhase = "trunk build --release";
    installPhase = ''
      mkdir -p $out/web
      cp -r ./crates/unavi-app/dist/* $out/web
    '';
  });
}
