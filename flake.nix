{
  inputs = {
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
  };

  outputs = { self, nixpkgs, crane, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };
        inherit (pkgs) lib;

        rustToolchain =
          pkgs.pkgsBuildHost.rust-bin.stable.latest.default.override {
            targets = [ "wasm32-unknown-unknown" ];
          };

        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        commonArgs = {
          src = craneLib.cleanCargoSource (craneLib.path ./.);
          strictDeps = true;

          buildInputs = pkgs.lib.optionals pkgs.stdenv.isLinux (with pkgs; [
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

          nativeBuildInputs = with pkgs; [
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
        };

        cargoArtifacts =
          craneLib.buildDepsOnly (commonArgs // { pname = "deps"; });

        cargoClippy = craneLib.cargoClippy (commonArgs // {
          inherit cargoArtifacts;
          pname = "clippy";
        });

        cargoDoc = craneLib.cargoDoc (commonArgs // {
          inherit cargoArtifacts;
          pname = "doc";
        });

        unavi-app = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
          pname = "unavi-app";
          cargoExtraArgs = "-p unavi-app";
          postInstall = ''
            cp -r assets $out/bin
          '';
        });

        unavi-web = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
          pname = "unavi-web";
          cargoExtraArgs = "-p unavi-app --target wasm32-unknown-unknown";
          src = lib.cleanSourceWith {
            src = ./.;
            filter = path: type:
              (lib.hasSuffix ".html" path)
              || (lib.hasInfix "/crates/unavi-app/public/" path)
              || (craneLib.filterCargoSources path type);
          };
        });

        unavi-server = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
          pname = "unavi-server";
          cargoExtraArgs = "-p unavi-server";
        });

        linux = pkgs.callPackage ./derivations/linux { };
      in {
        apps = rec {
          app = flake-utils.lib.mkApp {
            drv = pkgs.writeScriptBin "app" ''
              ${unavi-app}/bin/unavi-app
            '';
          };

          server = flake-utils.lib.mkApp {
            drv = pkgs.writeScriptBin "server" ''
              ${unavi-server}/bin/unavi-server
            '';
          };

          web = flake-utils.lib.mkApp {
            drv = pkgs.writeShellScriptBin "web" ''
              ${pkgs.python3Minimal}/bin/python3 -m http.server --directory ${unavi-web} 3000
            '';
          };

          default = app;
        };

        packages = rec {
          linux-app = pkgs.symlinkJoin {
            name = "linux-app";
            paths = [ linux.unavi-app ];
          };
          linux-server = pkgs.symlinkJoin {
            name = "linux-server";
            paths = [ linux.unavi-server ];
          };
          linux = pkgs.symlinkJoin {
            name = "linux";
            paths = [ linux-app linux-server ];
          };

          app = unavi-app;
          server = unavi-server;
          web = unavi-web;
          default = pkgs.symlinkJoin {
            name = "all";
            paths = [ app server web ];
          };
        };

        checks = { inherit cargoClippy cargoDoc unavi-app unavi-server; };

        devShells.default = craneLib.devShell {
          inputsFrom = [ unavi-app ];

          packages = with pkgs; [
            cargo-watch
            clang
            rust-analyzer
            rustToolchain
            zip
          ];

          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [ pkgs.vulkan-loader ];
          LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
        };
      });
}
