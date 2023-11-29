{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        flake-utils.follows = "flake-utils";
        nixpkgs.follows = "nixpkgs";
      };
    };
  };

  outputs = { self, flake-utils, nixpkgs, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        rustBin = pkgs.rust-bin.nightly.latest.default.override {
          targets = [ "wasm32-unknown-unknown" ];
        };

        build_inputs = pkgs.lib.optionals pkgs.stdenv.isLinux (with pkgs; [
          # Bevy
          alsa-lib.dev
          libxkbcommon
          udev
          vulkan-loader
          wayland
          xorg.libX11
          xorg.libXcursor
          xorg.libXi
          xorg.libXrandr
        ]);

        native_build_inputs = with pkgs; [
          cargo-auditable
          clang
          cmake
          pkg-config
          protobuf
          wasm-bindgen-cli
          wasm-tools

          # Leptos
          binaryen
          # cargo-leptos (broken right now?)
          openssl
        ];

        code = pkgs.callPackage ./. {
          inherit pkgs system build_inputs native_build_inputs;
        };
      in rec {
        packages = code // {
          all = pkgs.symlinkJoin {
            name = "all";
            paths = with code; [
              unavi_native
              unavi_server
              unavi_ui
              # unavi_web
              wired_protocol_proto
            ];
          };

          unavi_native = code.unavi_native;
          unavi_server = code.unavi_server;
          unavi_ui = code.unavi_ui;
          unavi_web = code.unavi_web;
          wired_protocol_proto = code.wired_protocol_proto;

          default = packages.all;
          override = packages.all;
          overrideDerivation = packages.all;
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs;
            [ cargo-watch clang rust-analyzer rustBin ] ++ build_inputs;
          nativeBuildInputs = native_build_inputs;

          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath build_inputs;
          LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
        };
      });
}
