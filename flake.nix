{
  inputs = {
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
    nix-github-actions = {
      url = "github:nix-community/nix-github-actions";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
  };

  outputs = { self, nix-github-actions, nixpkgs, crane, flake-utils
    , rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };
        inherit (pkgs) lib;

        rustToolchain =
          pkgs.pkgsBuildHost.rust-bin.stable.latest.default.override {
            targets = [ "wasm32-unknown-unknown" "wasm32-wasi" ];
          };

        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        commonArgs = {
          src = lib.cleanSourceWith {
            src = ./.;
            filter = path: type:
              (lib.hasInfix "/wired-protocol/" path)
              || (craneLib.filterCargoSources path type);
          };

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
            cargo-component
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

        cargoArtifactsWasm = craneLib.buildDepsOnly (commonArgs // {
          pname = "deps-wasm";
          doCheck = false;
        });

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

          src = lib.cleanSourceWith {
            src = ./.;
            filter = path: type:
              (lib.hasInfix "/assets/" path)
              || (lib.hasInfix "/wired-protocol/" path)
              || (craneLib.filterCargoSources path type);
          };

          postInstall = ''
            cp -r assets $out/bin
          '';
        });

        unavi-server = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
          pname = "unavi-server";
          cargoExtraArgs = "-p unavi-server";
        });

        unavi-web = craneLib.buildTrunkPackage (commonArgs // {
          inherit cargoArtifactsWasm;
          pname = "unavi-web";
          cargoExtraArgs = "-p unavi-app --target wasm32-unknown-unknown";
          trunkIndexPath = "crates/unavi-app/index.html";

          src = lib.cleanSourceWith {
            src = ./.;
            filter = path: type:
              (lib.hasSuffix ".html" path) || (lib.hasInfix "/assets/" path)
              || (lib.hasInfix "/wired-protocol/" path)
              || (lib.hasInfix "/crates/unavi-app/public/" path)
              || (craneLib.filterCargoSources path type);
          };

          wasm-bindgen-cli = pkgs.wasm-bindgen-cli.override {
            version = "0.2.90";
            hash = "sha256-X8+DVX7dmKh7BgXqP7Fp0smhup5OO8eWEhn26ODYbkQ=";
            cargoHash = "sha256-ckJxAR20GuVGstzXzIj1M0WBFj5eJjrO2/DRMUK5dwM=";
          };
        });
      in {
        checks = {
          inherit unavi-app unavi-server unavi-web cargoClippy cargoDoc;
        };

        apps = rec {
          app = flake-utils.lib.mkApp {
            drv = pkgs.writeScriptBin "app" ''
              ${self.packages.${system}.app}/bin/unavi-app
            '';
          };
          server = flake-utils.lib.mkApp {
            drv = pkgs.writeScriptBin "server" ''
              ${self.packages.${system}.server}/bin/unavi-server
            '';
          };
          web = flake-utils.lib.mkApp {
            drv = pkgs.writeShellScriptBin "web" ''
              ${pkgs.python3Minimal}/bin/python3 -m http.server --directory ${
                self.packages.${system}.web
              } 3000
            '';
          };

          default = app;
        };

        packages = rec {
          app = unavi-app;
          server = unavi-server;
          web = unavi-web;

          default = pkgs.symlinkJoin {
            name = "all";
            paths = [ app server web ];
          };
        };

        devShells.default = craneLib.devShell {
          checks = self.checks.${system};

          packages = with pkgs; [ cargo-watch clang rust-analyzer zip ];

          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [ pkgs.vulkan-loader ];
          LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
        };
      }) // (let
        gh_packages = [ "app" "server" ];
        gh_systems = [
          flake-utils.lib.system.x86_64-darwin
          flake-utils.lib.system.x86_64-linux
        ];
      in {
        githubActions = nix-github-actions.lib.mkGithubMatrix {
          attrPrefix = "";
          checks = nixpkgs.lib.mapAttrs (_: v:
            nixpkgs.lib.filterAttrs
            (n: _: !nixpkgs.lib.mutuallyExclusive [ n ] gh_packages) v)
            (nixpkgs.lib.getAttrs gh_systems self.packages);
        };
      });
}
