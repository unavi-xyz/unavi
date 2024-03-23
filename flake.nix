{
  inputs = {
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    deploy-rs = {
      url = "github:serokell/deploy-rs";
      inputs.nixpkgs.follows = "nixpkgs";
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
        flake-utils.follows = "flake-utils";
      };
    };
  };

  outputs = { self, deploy-rs, nix-github-actions, nixpkgs, crane, flake-utils
    , rust-overlay, ... }@inputs:
    flake-utils.lib.eachDefaultSystem (localSystem:
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

        rustToolchain =
          pkgs.pkgsBuildHost.rust-bin.stable.latest.default.override {
            targets = [ "wasm32-unknown-unknown" "wasm32-wasi" ];
          };

        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        components = import ./components.nix
          (inputs // { inherit craneLib localSystem pkgs; });
        crates = import ./crates.nix
          (inputs // { inherit components craneLib localSystem pkgs; });
      in {
        apps = components.apps // crates.apps;
        checks = crates.checks;
        packages = components.packages // crates.packages;

        devShells.default = craneLib.devShell {
          packages = with pkgs; [
            cargo-component
            cargo-watch
            clang
            curl
            morph
            nodePackages.prettier
            rust-analyzer
            terraform
          ];

          # LD_LIBRARY_PATH = lib.makeLibraryPath (commonArgs.buildInputs);
          LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
        };
      }) // (import ./deployments { inherit self nixpkgs deploy-rs; }) // {
        githubMatrix = nix-github-actions.lib.mkGithubMatrix {
          attrPrefix = "";
          checks = nixpkgs.lib.mapAttrs (_: v:
            (nixpkgs.lib.filterAttrs (n: _:
              !(nixpkgs.lib.mutuallyExclusive [ n ] [
                "unavi-app"
                "unavi-server"
              ])) v)) (nixpkgs.lib.getAttrs [
                flake-utils.lib.system.x86_64-linux
                flake-utils.lib.system.x86_64-darwin
              ] self.packages);
        };
      };
}
