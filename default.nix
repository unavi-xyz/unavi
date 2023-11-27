{ lib, pkgs, nixpkgs, system, makeRustPlatform, rust-overlay }:
let
  rustPkgs = import nixpkgs {
    inherit system;
    overlays = [ (import rust-overlay) ];
  };

  rustVersion = "1.73.0";
  wasmTarget = "wasm32-unknown-unknown";

  rustWithWasmTarget =
    rustPkgs.rust-bin.stable.${rustVersion}.default.override {
      targets = [ wasmTarget ];
    };

  rustPlatformWasm = makeRustPlatform {
    cargo = rustWithWasmTarget;
    rustc = rustWithWasmTarget;
  };

  common = {
    version = "0.0.0";
    src = ./.;
    cargoLock.lockFile = ./Cargo.lock;

    buildInputs = with pkgs; [
      # Bevy
      alsa-lib
      libxkbcommon
      udev
      vulkan-loader
      wayland
      xorg.libX11
      xorg.libXcursor
      xorg.libXi
      xorg.libXrandr
      zstd
    ];

    nativeBuildInputs = with pkgs; [ pkg-config cargo-auditable wasm-pack ];

    PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
  };
in {
  app = pkgs.rustPlatform.buildRustPackage (common // { pname = "unavi-app"; });

  wasm = rustPlatformWasm.buildRustPackage (common // {
    pname = "unavi-wasm";

    buildPhase = ''
      cargo build --release --target=${wasmTarget}
    '';
    installPhase = ''
      mkdir -p $out/lib
      cp target/wasm32-unknown-unknown/release/*.wasm $out/lib/
    '';
  });
}
