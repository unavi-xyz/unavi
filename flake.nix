{
  inputs = {
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
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

        crates = import ./crates.nix (inputs // { inherit craneLib localSystem pkgs; });
        deployments = import ./deployments (inputs // { inherit localSystem; });

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
        inherit crates;

        apps = crates.apps // deployments.apps;

        checks = crates.checks;
        packages =
          crates.packages
          // deployments.packages
          // {
            githubMatrix = githubMatrix // {
              matrix.include = map (
                entry:
                let
                  split = pkgs.lib.strings.splitString "." entry.attr;
                  package = pkgs.lib.strings.removeSuffix "\"" (
                    pkgs.lib.strings.removePrefix "\"" (builtins.elemAt split 1)
                  );
                  platform = builtins.elemAt split 0;
                in
                entry // { name = "${package}.${platform}"; }
              ) githubMatrix.matrix.include;
            };
          };

        devShells.default = craneLib.devShell {
          packages =
            (with pkgs; [
              cargo-component
              cargo-deny
              cargo-machete
              cargo-rdme
              cargo-release
              cargo-watch
              nodePackages.prettier
              openssh
              rust-analyzer
              terraform
            ])
            ++ [ cargo-wix ]
            ++ crates.buildInputs
            ++ crates.nativeBuildInputs;

          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath crates.buildInputs;
        };

        formatter = pkgs.nixfmt-rfc-style;

      }
    )
    // (
      let
        deployments = import ./deployments (inputs // { localSystem = "x86_64-linux"; });
      in
      {
        deploy = deployments.deploy;
      }
    );
}
