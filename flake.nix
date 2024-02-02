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

        rustBin = pkgs.rust-bin.stable.latest.default.override {
          targets = [ "wasm32-unknown-unknown" ];
        };

        build_inputs = pkgs.lib.optionals pkgs.stdenv.isLinux (with pkgs; [
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
          binaryen
          cargo-auditable
          clang
          cmake
          nodePackages.prettier
          pkg-config
          protobuf
          trunk
          wasm-bindgen-cli
          wasm-tools
        ];

        code = pkgs.callPackage ./. {
          inherit pkgs system build_inputs native_build_inputs;
        };
      in rec {
        packages = code // {
          app = pkgs.symlinkJoin {
            name = "app";
            paths = with code; [ unavi-app ];
          };

          server = pkgs.symlinkJoin {
            name = "server";
            paths = with code; [ unavi-server ];
          };

          wasm = pkgs.symlinkJoin {
            name = "wasm";
            paths = with code; [ unavi-system unavi-ui wired-script ];
          };

          web = pkgs.symlinkJoin {
            name = "web";
            paths = with code; [ web ];
          };

          all = pkgs.symlinkJoin {
            name = "all";
            paths = with code; [ unavi-app unavi-server web ];
          };

          default = packages.all;
          override = packages.all;
          overrideDerivation = packages.all;
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs;
            [ cargo-watch clang rust-analyzer rustBin zip ] ++ build_inputs;

          nativeBuildInputs = native_build_inputs;

          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath build_inputs;
          LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
        };
      });
}
