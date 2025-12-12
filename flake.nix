{
  inputs = {
    # Nix
    flake-parts = {
      inputs.nixpkgs-lib.follows = "nixpkgs";
      url = "github:hercules-ci/flake-parts";
    };
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    systems.url = "github:nix-systems/default";
    treefmt-nix.url = "github:numtide/treefmt-nix";

    # Rust
    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };
    crane.url = "github:ipetkov/crane";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    wit-deps.url = "github:bytecodealliance/wit-deps";

    # Deployment
    deploy-rs.url = "github:serokell/deploy-rs";
    sops-nix = {
      url = "github:Mic92/sops-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    inputs@{
      self,
      flake-parts,
      systems,
      nixpkgs,
      deploy-rs,
      sops-nix,
      ...
    }:
    flake-parts.lib.mkFlake { inherit inputs; } (
      { ... }:
      {
        systems = import systems;

        imports = [
          inputs.treefmt-nix.flakeModule
          ./crates/unavi-client
          ./crates/unavi-launcher
          ./crates/unavi-server
        ];

        flake =
          let
            deployInfo = builtins.fromJSON (builtins.readFile ./infra/terraform/deploy.json);
          in
          {
            nixosConfigurations = {
              beta = nixpkgs.lib.nixosSystem {
                system = "x86_64-linux";
                specialArgs = { inherit inputs self deployInfo; };
                modules = [
                  ./infra/nixos/beta.nix
                  sops-nix.nixosModules.sops
                ];
              };

              stable = nixpkgs.lib.nixosSystem {
                system = "x86_64-linux";
                specialArgs = { inherit inputs self deployInfo; };
                modules = [
                  ./infra/nixos/stable.nix
                  sops-nix.nixosModules.sops
                ];
              };
            };

            deploy.nodes = {
              unavi-beta = {
                hostname = deployInfo.beta.server_ipv4;
                sshUser = "root";
                profiles.system = {
                  user = "root";
                  path = deploy-rs.lib.x86_64-linux.activate.nixos self.nixosConfigurations.beta;
                };
              };

              unavi-stable = {
                hostname = deployInfo.stable.server_ipv4;
                sshUser = "root";
                profiles.system = {
                  user = "root";
                  path = deploy-rs.lib.x86_64-linux.activate.nixos self.nixosConfigurations.stable;
                };
              };
            };

            checks = builtins.mapAttrs (_system: deployLib: deployLib.deployChecks self.deploy) deploy-rs.lib;
          };

        perSystem =
          {
            config,
            lib,
            pkgs,
            system,
            ...
          }:
          {
            _module.args.pkgs = import inputs.nixpkgs {
              system = "x86_64-linux";
              config.allowUnfree = true;
              overlays = [
                inputs.fenix.overlays.default

                (
                  self: _:
                  let
                    toolchain = (
                      with self.fenix;
                      combine [
                        complete.toolchain
                        targets.wasm32-wasip2.latest.rust-std
                      ]
                    );
                  in
                  {
                    crane = (inputs.crane.mkLib self).overrideToolchain toolchain;
                  }
                )
              ];
            };

            checks = {
              # audit = pkgs.crane.cargoAudit {
              #   inherit (inputs) advisory-db;
              #   src = ./.;
              #   pname = "unavi";
              # };
              deny = pkgs.crane.cargoDeny {
                src = ./.;
                pname = "unavi";
              };
            };

            packages.default = config.packages.unavi-client;

            treefmt.programs = {
              actionlint.enable = true;
              deadnix.enable = true;
              jsonfmt.enable = true;
              mdformat.enable = true;
              nixfmt = {
                enable = true;
                strict = true;
              };
              oxipng.enable = true;
              rustfmt.enable = true;
              statix.enable = true;
              taplo.enable = true;
              terraform.enable = true;
              yamlfmt.enable = true;
            };

            devShells =
              let
                packages =
                  config.packages
                  |> lib.attrValues
                  |> lib.flip pkgs.lib.forEach (x: x.buildInputs ++ x.nativeBuildInputs)
                  |> lib.concatLists;

                LD_LIBRARY_PATH =
                  config.packages
                  |> lib.attrValues
                  |> lib.filter (x: x ? linkedInputs)
                  |> lib.flip pkgs.lib.forEach (x: x.linkedInputs)
                  |> lib.concatLists
                  |> lib.makeLibraryPath;
              in
              {
                minimal = pkgs.crane.devShell { inherit packages LD_LIBRARY_PATH; };

                default = pkgs.crane.devShell {
                  packages =
                    (with pkgs; [
                      age
                      bacon
                      cargo-deny
                      cargo-edit
                      cargo-machete
                      cargo-nextest
                      cargo-release
                      cargo-watch
                      cargo-workspaces
                      deploy-rs.packages.${system}.default
                      doctl
                      rustup
                      sops
                      sqlx-cli
                      terraform
                      tokio-console
                    ])
                    ++ packages;

                  inherit LD_LIBRARY_PATH;

                  WEBKIT_DISABLE_DMABUF_RENDERER = 1; # Nvida + Wayland launcher bug
                };
              };
          };
      }
    );
}
