{
  inputs = {
    crane.url = "github:ipetkov/crane";
    deploy-rs = {
      url = "github:serokell/deploy-rs";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        utils.follows = "flake-utils";
      };
    };
    flake-utils.url = "github:numtide/flake-utils";
    nix-github-actions = {
      url = "github:nix-community/nix-github-actions";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    nixpkgs-stable.url = "github:NixOS/nixpkgs/nixos-23.11";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
  };

  outputs =
    {
      self,
      deploy-rs,
      nix-github-actions,
      nixpkgs,
      crane,
      flake-utils,
      rust-overlay,
      ...
    }@inputs:

    flake-utils.lib.eachDefaultSystem (
      localSystem:
      let
        pkgs = import nixpkgs {
          inherit localSystem;

          overlays = [
            (import rust-overlay)
            deploy-rs.overlay
            (self: super: {
              deploy-rs = {
                inherit (pkgs) deploy-rs;
                lib = super.deploy-rs.lib;
              };
            })
          ];

          config.allowUnfree = true;
        };

        rustToolchain = pkgs.pkgsBuildHost.rust-bin.stable.latest.default.override {
          targets = [
            "wasm32-unknown-unknown"
            "wasm32-wasip1"
          ];
        };
        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        cargo-wix = pkgs.rustPlatform.buildRustPackage rec {
          pname = "cargo-wix";
          version = "0.3.8";

          src = pkgs.fetchFromGitHub {
            owner = "volks73";
            repo = "cargo-wix";
            rev = "${version}";
            sha256 = "sha256-KtCQ068VDL596Q5dF61Q43UwbraKep+fv/2Q04tOP+M=";
          };

          cargoHash = "sha256-hLDIqcNVv2EEDMmdGrs54YacH0qkd+fTg0rfjdCClGk=";
        };

        src = pkgs.lib.cleanSourceWith {
          src = ./.;
          filter =
            path: type:
            (pkgs.lib.hasInfix "crates/unavi-app/assets/" path)
            || (pkgs.lib.hasInfix "crates/unavi-app/public/" path)
            || (pkgs.lib.hasInfix "wired-protocol" path)
            || (pkgs.lib.hasSuffix ".capnp" path)
            || (pkgs.lib.hasSuffix ".html" path)
            || (pkgs.lib.hasSuffix ".json" path)
            || (pkgs.lib.hasSuffix ".ttf" path)
            || (pkgs.lib.hasSuffix ".wit" path)
            || (pkgs.lib.hasSuffix "LICENSE" path)
            || (pkgs.lib.hasSuffix "README.md" path)
            || (craneLib.filterCargoSources path type);
        };

        commonArgs = {
          inherit
            buildInputs
            nativeBuildInputs
            src
            ;

          pname = "unavi";
          strictDeps = true;
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;
        cargoClippy = craneLib.cargoClippy (commonArgs // { inherit cargoArtifacts; });
        cargoDoc = craneLib.cargoDoc (commonArgs // { inherit cargoArtifacts; });
        cargoFmt = craneLib.cargoFmt (commonArgs // { inherit cargoArtifacts; });
        cargoTarpaulin = craneLib.cargoTarpaulin (commonArgs // { inherit cargoArtifacts; });

        unavi-app = import ./crates/unavi-app { inherit craneLib pkgs src; };
        unavi-server = import ./crates/unavi-server { inherit craneLib pkgs src; };

        buildInputs = unavi-app.buildInputs ++ unavi-server.buildInputs;
        nativeBuildInputs = unavi-app.nativeBuildInputs ++ unavi-server.nativeBuildInputs;

        deploy = import ./deploy (inputs // { inherit localSystem; });
      in
      {
        inherit unavi-app unavi-server;

        apps = rec {
          unavi-app = flake-utils.lib.mkApp { drv = unavi-app.native; };
          unavi-server = flake-utils.lib.mkApp { drv = unavi-server.server; };
          unavi-web = flake-utils.lib.mkApp {
            drv = pkgs.writeShellApplication {
              name = "unavi-web";
              runtimeInputs = with pkgs; [ python3Minimal ];
              text = ''
                python3 -m http.server --directory ${unavi-app.web} 8080
              '';
            };
          };
          default = unavi-app;
        } // deploy.apps;

        checks = {
          inherit
            cargoClippy
            cargoDoc
            cargoFmt
            cargoTarpaulin
            ;
        };

        packages = {
          default = unavi-app.native;
          unavi-app = unavi-app.native;
          unavi-server = unavi-server.server;
          unavi-web = unavi-app.web;
        } // deploy.packages;

        devShells.default = craneLib.devShell {
          packages =
            (with pkgs; [
              cargo-component
              cargo-deny
              cargo-edit
              cargo-machete
              cargo-rdme
              cargo-release
              cargo-watch
              cargo-workspaces
              nodePackages.prettier
              openssh
              rust-analyzer
              terraform
            ])
            ++ [ cargo-wix ]
            ++ buildInputs
            ++ nativeBuildInputs;

          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
        };

        formatter = pkgs.nixfmt-rfc-style;
      }
    )
    // (
      let
        deploy = import ./deploy (inputs // { localSystem = "x86_64-linux"; });
        githubMatrix = nix-github-actions.lib.mkGithubMatrix {
          attrPrefix = "";
          checks =
            nixpkgs.lib.mapAttrs
              (
                _: v:
                (nixpkgs.lib.filterAttrs (
                  n: _:
                  !(nixpkgs.lib.mutuallyExclusive [ n ] [
                    "unavi-app"
                    "unavi-server"
                  ])
                ) v)
              )
              (
                nixpkgs.lib.getAttrs [
                  flake-utils.lib.system.aarch64-darwin
                  flake-utils.lib.system.x86_64-linux
                ] self.packages
              );
        };
      in
      {
        deploy = deploy.deploy;
        githubMatrix = githubMatrix // {
          matrix.include = map (
            entry:
            let
              split = nixpkgs.lib.strings.splitString "." entry.attr;
              package = nixpkgs.lib.strings.removeSuffix "\"" (
                nixpkgs.lib.strings.removePrefix "\"" (builtins.elemAt split 1)
              );
              platformSplit = nixpkgs.lib.strings.splitString "-" (builtins.elemAt split 0);
              platformRaw = builtins.elemAt platformSplit 1;
              platform = if platformRaw == "darwin" then "macos" else platformRaw;
            in
            entry // { name = "${package}-${platform}"; }
          ) githubMatrix.matrix.include;
        };
      }
    );
}
