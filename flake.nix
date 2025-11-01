{
  inputs = {
    flake-parts = {
      inputs.nixpkgs-lib.follows = "nixpkgs";
      url = "github:hercules-ci/flake-parts";
    };
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    systems.url = "github:nix-systems/default";

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

    # Other
    treefmt-nix.url = "github:numtide/treefmt-nix";
  };

  outputs =
    inputs@{ flake-parts, systems, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } (
      { ... }:
      {
        systems = import systems;

        imports = [
          inputs.treefmt-nix.flakeModule
          ./crates/unavi
          ./crates/unavi-server
        ];

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
              inherit system;
              overlays = [
                inputs.fenix.overlays.default

                (
                  self: _:
                  let
                    nightly = (
                      with self.fenix;
                      combine [
                        complete.toolchain
                        targets.wasm32-wasip2.latest.rust-std
                      ]
                    );
                  in
                  {
                    crane = (inputs.crane.mkLib self).overrideToolchain nightly;
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

            devShells.default = pkgs.crane.devShell {
              packages =
                (with pkgs; [
                  cargo-deny
                  cargo-edit
                  cargo-machete
                  cargo-nextest
                  cargo-watch
                  cargo-workspaces
                ])
                ++ (
                  config.packages
                  |> lib.attrValues
                  |> lib.flip pkgs.lib.forEach (x: x.buildInputs ++ x.nativeBuildInputs)
                  |> lib.concatLists
                );

              LD_LIBRARY_PATH =
                config.packages
                |> lib.attrValues
                |> lib.flip pkgs.lib.forEach (x: x.runtimeDependencies)
                |> lib.concatLists
                |> lib.makeLibraryPath;
            };
          };
      }
    );
}
