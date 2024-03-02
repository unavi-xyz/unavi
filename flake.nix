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
    flake-utils.lib.eachSystem [
      flake-utils.lib.system.aarch64-darwin
      flake-utils.lib.system.aarch64-linux
      flake-utils.lib.system.x86_64-darwin
      flake-utils.lib.system.x86_64-linux
      # flake-utils.lib.system.x86_64-windows
    ] (localSystem:
      let
        pkgs = import nixpkgs {
          inherit localSystem;
          overlays = [ (import rust-overlay) ];
        };

        inherit (pkgs) lib;

        rustToolchain =
          pkgs.pkgsBuildHost.rust-bin.stable.latest.default.override {
            targets = [ "wasm32-unknown-unknown" "wasm32-wasi" ];
          };

        # TODO: Remove manual cargo-component build once nixpkgs-unstable has it
        cargo-component = pkgs.rustPlatform.buildRustPackage rec {
          pname = "cargo-component";
          version = "0.9.0";

          src = pkgs.fetchFromGitHub {
            owner = "bytecodealliance";
            repo = "cargo-component";
            rev = "v${version}";
            hash = "sha256-zJ3fV6GOYcbLvOjZKrSOxGPc8GSQGridInvOZFruXks=";
          };

          cargoHash = "sha256-ixk9ui/vS6DynCTF086JBFEw/JC8jpixvUkwIi5Hr0A=";

          nativeBuildInputs = with pkgs; [ pkg-config ];
          buildInputs = with pkgs;
            [ openssl ] ++ lib.optionals stdenv.isDarwin
            [ darwin.apple_sdk.frameworks.SystemConfiguration ];

          doCheck = false;
        };

        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        commonArgs = {
          src = lib.cleanSourceWith {
            src = ./.;
            filter = path: type:
              (lib.hasSuffix ".proto" path) || (lib.hasSuffix ".wit" path)
              || (craneLib.filterCargoSources path type);
          };

          strictDeps = true;

          buildInputs = lib.optionals pkgs.stdenv.isLinux (with pkgs; [
            alsa-lib
            alsa-lib.dev
            libxkbcommon
            udev
            vulkan-loader
            wayland
            xorg.libX11
            xorg.libXcursor
            xorg.libXi
            xorg.libXrandr
          ]) ++ lib.optionals pkgs.stdenv.isDarwin
            (with pkgs; [ pkgs.darwin.apple_sdk.frameworks.Cocoa ]);

          nativeBuildInputs = with pkgs;
            [
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
            ] ++ lib.optionals (!pkgs.stdenv.isDarwin)
            (with pkgs; [ alsa-lib alsa-lib.dev ]);
        };

        wasmArgs = commonArgs // {
          CARGO_BUILD_TARGET = "wasm32-unknown-unknown";
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

        unavi-ui = craneLib.buildPackage (wasmArgs // {
          pname = "unavi-ui";
          cargoBuildCommand = "cargo component build --profile wasm-release";
          cargoExtraArgs = "--locked -p unavi-ui";
          doCheck = false;
        });

        unavi-system = craneLib.buildPackage (commonArgs // {
          # Manually build component deps until cargo-component has better support :(
          cargoArtifacts = unavi-ui;
          preBuild = ''
            cargo component build --profile wasm-release --locked -p unavi-ui
          '';

          pname = "unavi-system";
          cargoBuildCommand = "cargo component build --profile wasm-release";
          cargoExtraArgs = "--locked -p unavi-system";
          doCheck = false;
        });

        unavi-app = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
          pname = "unavi-app";
          cargoExtraArgs = "--locked -p unavi-app";

          src = lib.cleanSourceWith {
            src = ./.;
            filter = path: type:
              (lib.hasSuffix ".proto" path) || (lib.hasSuffix ".wit" path)
              || (lib.hasInfix "/assets/" path)
              || (craneLib.filterCargoSources path type);
          };

          postInstall = ''
            cp -r assets $out/bin
          '';
        });

        unavi-server = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
          pname = "unavi-server";
          cargoExtraArgs = "--locked -p unavi-server";
        });

        cargoArtifactsWeb = craneLib.buildDepsOnly (wasmArgs // {
          pname = "deps-web";
          cargoExtraArgs = "--locked -p unavi-app";
          doCheck = false;
        });

        unavi-web = craneLib.buildTrunkPackage (wasmArgs // {
          cargoArtifacts = cargoArtifactsWeb;

          pname = "unavi-web";
          cargoExtraArgs = "--locked -p unavi-app";
          trunkIndexPath = "./crates/unavi-app/index.html";

          src = lib.cleanSourceWith {
            src = ./.;
            filter = path: type:
              (lib.hasSuffix ".html" path) || (lib.hasSuffix ".proto" path)
              || (lib.hasSuffix ".wit" path) || (lib.hasInfix "/assets/" path)
              || (lib.hasInfix "/crates/unavi-app/public/" path)
              || (craneLib.filterCargoSources path type);
          };

          wasm-bindgen-cli = pkgs.wasm-bindgen-cli;
        });
      in {
        checks = {
          inherit unavi-app unavi-server unavi-web cargoClippy cargoDoc;
        };

        apps = rec {
          app = flake-utils.lib.mkApp {
            drv = pkgs.writeScriptBin "app" ''
              ${self.packages.${localSystem}.app}/bin/unavi-app
            '';
          };
          server = flake-utils.lib.mkApp {
            drv = pkgs.writeScriptBin "server" ''
              ${self.packages.${localSystem}.server}/bin/unavi-server
            '';
          };
          web = flake-utils.lib.mkApp {
            drv = pkgs.writeShellScriptBin "web" ''
              ${pkgs.python3Minimal}/bin/python3 -m http.server --directory ${
                self.packages.${localSystem}.web
              } 3000
            '';
          };

          check-components = flake-utils.lib.mkApp {
            drv = pkgs.writeScriptBin "check-components" ''
              mkdir -p ./target/wasm32-wasi/wasm-release
              cp ${
                self.packages.${localSystem}.components
              }/lib/* ./target/wasm32-wasi/wasm-release

              cargo component check -p unavi-ui -p unavi-system
            '';
          };

          default = app;
        };

        packages = {
          app = unavi-app;
          server = unavi-server;
          web = unavi-web;

          unavi-ui = unavi-ui;
          unavi-system = unavi-system;

          components = pkgs.symlinkJoin {
            name = "components";
            paths = [ unavi-ui unavi-system ];
          };

          default = pkgs.symlinkJoin {
            name = "all";
            paths = [ unavi-app unavi-server unavi-web unavi-ui unavi-system ];
          };
        };

        devShells.default = craneLib.devShell {
          checks = self.checks.${localSystem};

          packages = with pkgs; [ cargo-watch clang rust-analyzer zip ];

          LD_LIBRARY_PATH = lib.makeLibraryPath [ pkgs.vulkan-loader ];
          LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
        };
      }) // (let
        gh_packages = [ "app" "server" ];
        gh_systems = [
          # flake-utils.lib.system.aarch64-darwin
          flake-utils.lib.system.x86_64-linux
          # flake-utils.lib.system.x86_64-windows
        ];
      in {
        githubActions = nix-github-actions.lib.mkGithubMatrix {
          attrPrefix = "";
          checks = nixpkgs.lib.mapAttrs (_: v:
            (nixpkgs.lib.filterAttrs
              (n: _: !(nixpkgs.lib.mutuallyExclusive [ n ] gh_packages)) v))
            (nixpkgs.lib.getAttrs gh_systems self.packages);
        };
      });
}
