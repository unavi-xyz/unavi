{
  description = "A flake for building a Rust workspace using buildRustPackage.";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";

    rust-overlay.inputs.flake-utils.follows = "flake-utils";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { self, flake-utils, nixpkgs, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        wasmTarget = "wasm32-unknown-unknown";
        rustBin = pkgs.rust-bin.stable.latest.default;
        rustBinWasm = rustBin.override { targets = [ wasmTarget ]; };

        build_inputs = with pkgs; [
          # Bevy
          alsa-lib
          udev
          vulkan-loader

          libxkbcommon
          wayland

          xorg.libX11
          xorg.libXcursor
          xorg.libXi
          xorg.libXrandr
        ];

        native_build_inputs = with pkgs; [
          # Leptos
          binaryen
          cargo-leptos
          openssl

          # Rust
          cargo-auditable
          pkg-config
          wasm-bindgen-cli
        ];

        code = pkgs.callPackage ./. {
          inherit pkgs system build_inputs native_build_inputs;
        };
      in rec {
        packages = code // {
          all = pkgs.symlinkJoin {
            name = "all";
            paths = with code; [ native server web ];
          };

          default = packages.all;
          override = packages.all;
          overrideDerivation = packages.all;
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs;
            [
              # Rust
              cargo-watch
              rust-analyzer
              rustBinWasm
            ] ++ build_inputs;
          nativeBuildInputs = native_build_inputs;

          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath build_inputs;
        };
      });
}
