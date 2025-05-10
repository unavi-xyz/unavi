{
  inputs = {
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:NixOS/nixpkgs/release-24.11";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
    treefmt-nix.url = "github:numtide/treefmt-nix";
  };

  outputs =
    {
      crane,
      flake-utils,
      nixpkgs,
      rust-overlay,
      treefmt-nix,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      localSystem:
      let
        pkgs = import nixpkgs {
          inherit localSystem;

          overlays = [
            (import rust-overlay)
          ];
        };

        wac-cli = pkgs.rustPlatform.buildRustPackage rec {
          pname = "wac-cli";
          version = "0.6.1";

          src = pkgs.fetchFromGitHub {
            owner = "bytecodealliance";
            repo = "wac";
            rev = "v${version}";
            sha256 = "sha256-noBVAhoHXl3FI6ZlnmCwpnqu7pub6FCtuY+026vdlYo=";
          };

          cargoHash = "sha256-sSV5zzDUaR0JUuoigZDbkvZE43R+f66sUl9ytRFj9u0";

          nativeBuildInputs = [ pkgs.pkg-config ];

          buildInputs =
            [ pkgs.openssl ]
            ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [ pkgs.darwin.apple_sdk.frameworks.SystemConfiguration ];
        };

        rustToolchain = pkgs.pkgsBuildHost.rust-bin.stable.latest.default.override {
          targets = [
            "wasm32-wasip2"
          ];
        };
        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        treefmtEval = treefmt-nix.lib.evalModule pkgs ./treefmt.nix;

        src = pkgs.lib.cleanSourceWith {
          src = ./.;
          filter =
            path: type:
            (pkgs.lib.hasInfix "crates/unavi-app/assets/" path)
            || (pkgs.lib.hasInfix "wired-protocol" path)
            || (pkgs.lib.hasSuffix ".html" path)
            || (pkgs.lib.hasSuffix ".json" path)
            || (pkgs.lib.hasSuffix ".wit" path)
            || (pkgs.lib.hasSuffix "LICENSE" path)
            || (pkgs.lib.hasSuffix "README.md" path)
            || (craneLib.filterCargoSources path type);
        };

        unavi-app = import ./crates/unavi-app {
          inherit
            craneLib
            pkgs
            src
            wac-cli
            ;
        };

        buildInputs = unavi-app.buildInputs;
        nativeBuildInputs = unavi-app.nativeBuildInputs;
      in
      {
        formatter = treefmtEval.config.build.wrapper;

        apps = rec {
          app = flake-utils.lib.mkApp {
            drv = unavi-app.package;
            exePath = "/unavi-app";
          };
          default = app;
        };

        packages = {
          default = unavi-app.package;
          unavi-app = unavi-app.package;
        };

        devShells.default = craneLib.devShell {
          packages =
            (with pkgs; [
              cargo-deny
              cargo-edit
              cargo-machete
              cargo-release
              cargo-watch
              cargo-workspaces
              wac-cli
            ])
            ++ buildInputs
            ++ nativeBuildInputs;

          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
        };
      }
    );
}
