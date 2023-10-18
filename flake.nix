{
  description = "A flake for building a Rust workspace using buildRustPackage.";

  inputs = {
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.follows = "rust-overlay/flake-utils";
    nixpkgs.follows = "rust-overlay/nixpkgs";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        build_inputs = with pkgs; [
          # Rust
          rust-analyzer
          rust-bin.stable.latest.default

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
          # Rust
          cargo-auditable
          pkg-config
        ];

        code = pkgs.callPackage ./. {
          inherit pkgs system build_inputs native_build_inputs;
        };
      in rec {
        packages = {
          app = code.app;
          wasm = code.wasm;
          all = pkgs.symlinkJoin {
            name = "all";
            paths = with code; [ app wasm ];
          };
          default = packages.all;
        };

        devShell = pkgs.mkShell {
          buildInputs = build_inputs;
          nativeBuildInputs = native_build_inputs;

          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath build_inputs;
        };
      });
}
