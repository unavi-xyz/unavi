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
    flake-utils.lib.eachDefaultSystem (localSystem:
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

        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        src = craneLib.cleanCargoSource (craneLib.path ./.);

        commonArgs = {
          pname = "unavi";

          src = lib.cleanSourceWith {
            src = ./.;
            filter = path: type:
              (lib.hasSuffix ".proto" path)
              || (craneLib.filterCargoSources path type);
          };

          strictDeps = true;

          buildInputs = with pkgs;
            [ rustPlatform.bindgenHook ] ++ lib.optionals pkgs.stdenv.isLinux
            (with pkgs; [
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
            (with pkgs; [ darwin.apple_sdk.frameworks.Cocoa libiconv ]);

          nativeBuildInputs = with pkgs;
            [
              binaryen
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

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        cargoClippy =
          craneLib.cargoClippy (commonArgs // { inherit cargoArtifacts; });

        cargoDoc =
          craneLib.cargoDoc (commonArgs // { inherit cargoArtifacts; });

        cargoFmt = craneLib.cargoFmt {
          inherit src;
          pname = "unavi";
        };

        cargoTest = craneLib.cargoTest (commonArgs // {
          inherit cargoArtifacts;
          cargoTestArgs = "--all-features --all-targets";
        });

        cargoTestDoc = craneLib.cargoTest (commonArgs // {
          inherit cargoArtifacts;
          pname = "doc";
          cargoTestArgs = "--all-features --doc";
        });

        generateAssetsScript = ''
          rm -rf assets/components
          mkdir -p assets/components
          cp -r --no-preserve=mode ${
            self.packages.${localSystem}.components
          }/lib/* assets/components
        '';

        # Crates
        unavi-app = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
          pname = "unavi-app";
          cargoExtraArgs = "--locked -p unavi-app";

          preBuild = generateAssetsScript;
          postInstall = ''
            cp -r assets $out/bin
          '';

          src = lib.cleanSourceWith {
            src = ./.;
            filter = path: type:
              (lib.hasSuffix ".wit" path) || (lib.hasInfix "/assets/" path)
              || (craneLib.filterCargoSources path type);
          };
        });

        unavi-server = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
          pname = "unavi-server";
          cargoExtraArgs = "--locked -p unavi-server";
        });

        web = craneLib.buildTrunkPackage (commonArgs // rec {
          src = lib.cleanSourceWith {
            src = ./.;
            filter = path: type:
              (lib.hasSuffix ".wit" path) || (lib.hasSuffix ".html" path)
              || (lib.hasInfix "/assets/" path)
              || (lib.hasInfix "/crates/unavi-app/public/" path)
              || (craneLib.filterCargoSources path type);
          };

          pname = "web";
          cargoExtraArgs = "--locked -p unavi-app";
          trunkIndexPath = "./crates/unavi-app/index.html";
          wasm-bindgen-cli = pkgs.wasm-bindgen-cli;

          preBuild = generateAssetsScript;
        });

        # Components
        componentArgs = commonArgs // {
          cargoBuildCommand = "cargo component build --profile wasm-release";
          doCheck = false;

          src = lib.cleanSourceWith {
            src = ./.;
            filter = path: type:
              (lib.hasSuffix ".wit" path)
              || (craneLib.filterCargoSources path type);
          };
        };

        buildComponent = pname:
          craneLib.buildPackage (componentArgs // {
            inherit pname;
            cargoExtraArgs = "--locked -p ${pname}";
          });

        component-names =
          lib.mapAttrsToList (name: _: name) (builtins.readDir ./components);

        components = (map (name: buildComponent name) component-names);

      in {
        checks = {
          inherit cargoClippy cargoDoc cargoFmt cargoTest cargoTestDoc;
        };

        apps = rec {
          app = flake-utils.lib.mkApp { drv = unavi-app; };
          server = flake-utils.lib.mkApp { drv = unavi-server; };

          web = flake-utils.lib.mkApp {
            drv = pkgs.writeShellScriptBin "web" ''
              ${pkgs.python3Minimal}/bin/python3 -m http.server --directory ${
                self.packages.${localSystem}.web
              } 3000
            '';
          };

          check-components = flake-utils.lib.mkApp {
            drv = pkgs.writeShellScriptBin "check-components"
              (lib.concatStringsSep " -p "
                ([ "cargo component check --locked" ] ++ component-names));
          };

          generate-assets = flake-utils.lib.mkApp {
            drv =
              pkgs.writeShellScriptBin "generate-assets" generateAssetsScript;
          };

          default = app;
        };

        packages = {
          inherit unavi-app unavi-server web;

          components = pkgs.symlinkJoin {
            name = "components";
            paths = [ components ];
          };

          default = pkgs.symlinkJoin {
            name = "all";
            paths = [ components unavi-app unavi-server web ];
          };
        };

        devShells.default = craneLib.devShell {
          checks = self.checks.${localSystem};

          packages = with pkgs; [ cargo-deny cargo-watch clang rust-analyzer ];

          LD_LIBRARY_PATH = lib.makeLibraryPath (commonArgs.buildInputs);
          LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
        };
      }) // (let
        ghPackages = [ "unavi-app" "unavi-server" ];
        ghSystems = [
          flake-utils.lib.system.x86_64-darwin
          flake-utils.lib.system.x86_64-linux
        ];
        githubActions = nix-github-actions.lib.mkGithubMatrix {
          attrPrefix = "";
          checks = nixpkgs.lib.mapAttrs (_: v:
            (nixpkgs.lib.filterAttrs
              (n: _: !(nixpkgs.lib.mutuallyExclusive [ n ] ghPackages)) v))
            (nixpkgs.lib.getAttrs ghSystems self.packages);
        };
      in {
        githubActions = {
          checks = githubActions.checks;
          matrix = {
            include = githubActions.matrix.include ++ (map (package: {
              inherit package;
              attr =
                nixpkgs.lib.concatStringsSep "." [ "x86_64-windows" package ];
              os = [ "windows-latest" ];
            }) ghPackages);
          };
        };
      });
}
