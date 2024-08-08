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

        wac-cli = pkgs.rustPlatform.buildRustPackage rec {
          pname = "wac-cli";
          version = "0.3.0";

          src = pkgs.fetchFromGitHub {
            owner = "bytecodealliance";
            repo = "wac";
            rev = "v${version}";
            sha256 = "sha256-xv+lSsJ+SSRovJ0mt8/AbEjEdyaRvO3qzY44ih9oSF0=";
          };

          cargoHash = "sha256-+hmTsTfcxygdU/pDTkmkuQgujEOR1+H8YZG4ScVBKcc=";

          nativeBuildInputs = [ pkgs.pkg-config ];

          buildInputs =
            [ pkgs.openssl ]
            ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [ pkgs.darwin.apple_sdk.frameworks.SystemConfiguration ];
        };

        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        crates = import ./crates.nix (inputs // { inherit craneLib localSystem pkgs; });

        terraform = import ./deployments/terraform.nix (inputs // { inherit localSystem pkgs; });

        deployments = import ./deployments (inputs // { inherit localSystem; });
      in
      {
        inherit crates;

        apps =
          crates.apps
          // terraform.apps
          // deployments.apps
          // {
            generate-readme = flake-utils.lib.mkApp {
              drv = pkgs.writeShellApplication {
                name = "generate-readme";
                runtimeInputs = with pkgs; [
                  rust-bin.stable.latest.default
                  cargo-rdme
                ];
                text = ''
                  cd crates
                  for folder in */; do
                    (cd "$folder" && cargo rdme) || true
                  done'';
              };
            };
          };

        checks = crates.checks;
        packages = crates.packages // deployments.packages;

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
              rust-analyzer
            ])
            ++ [ wac-cli ]
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
      }
    );
}
