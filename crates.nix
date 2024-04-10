{
  flake-utils,
  components,
  craneLib,
  localSystem,
  pkgs,
  self,
  ...
}:
let
  lib = pkgs.lib;

  src = lib.cleanSourceWith {
    src = ./.;
    filter =
      path: type:
      (lib.hasInfix "/assets/" path)
      || (lib.hasInfix "/crates/unavi-app/public/" path)
      || (lib.hasSuffix ".html" path)
      || (lib.hasSuffix ".json" path)
      || (lib.hasSuffix ".wit" path)
      || (craneLib.filterCargoSources path type);
  };

  clibs =
    with pkgs;
    [
      clang
      cmake
      pkg-config
      rustPlatform.bindgenHook
    ]
    ++ lib.optionals (pkgs.stdenv.isLinux && pkgs.stdenv.hostPlatform.system != "aarch64-linux") (
      with pkgs;
      [
        glibc_multi
        glibc_multi.dev
      ]
    )
    ++ lib.optionals pkgs.stdenv.isDarwin (with pkgs; [ libiconv ]);

  unaviAppConfig = {
    inherit src;

    buildInputs =
      lib.optionals pkgs.stdenv.isLinux (
        with pkgs;
        [
          alsa-lib
          libxkbcommon
          openssl
          udev
          vulkan-loader
          wayland
          xorg.libX11
          xorg.libXcursor
          xorg.libXi
          xorg.libXrandr
        ]
      )
      ++ lib.optionals pkgs.stdenv.isDarwin (
        with pkgs;
        [
          darwin.apple_sdk.frameworks.AudioUnit
          darwin.apple_sdk.frameworks.Cocoa
        ]
      );
    nativeBuildInputs =
      lib.optionals pkgs.stdenv.isLinux (
        with pkgs;
        [
          alsa-lib.dev
          openssl.dev
        ]
      )
      ++ clibs;

    cargoExtraArgs = "--locked -p unavi-app";
    pname = "unavi-app";
    strictDeps = true;

    preBuild = components.generateAssetsScript;
    postInstall = ''
      cp -r assets $out/bin
    '';
  };

  unaviServerConfig = {
    inherit src;

    buildInputs = with pkgs; [ openssl ];
    nativeBuildInputs = with pkgs; [ openssl.dev ] ++ clibs;

    cargoExtraArgs = "--locked -p unavi-server";
    pname = "unavi-server";
    strictDeps = true;
  };

  unaviWebConfig = {
    inherit src;

    buildInputs = unaviAppConfig.buildInputs;
    nativeBuildInputs =
      with pkgs;
      [
        binaryen
        trunk
        wasm-bindgen-cli
        wasm-tools
      ]
      ++ unaviAppConfig.nativeBuildInputs;

    cargoExtraArgs = "--locked -p unavi-app";
    pname = "web";
    strictDeps = true;
    trunkIndexPath = "./crates/unavi-app/index.html";
    wasm-bindgen-cli = pkgs.wasm-bindgen-cli;

    preBuild = components.generateAssetsScript;
  };

  commonArgs = {
    inherit src;

    buildInputs = unaviAppConfig.buildInputs ++ unaviServerConfig.buildInputs;
    nativeBuildInputs = unaviServerConfig.nativeBuildInputs ++ unaviWebConfig.nativeBuildInputs;

    pname = "unavi";
    strictDeps = true;
  };

  cargoArtifacts = craneLib.buildDepsOnly commonArgs;
  cargoClippy = craneLib.cargoClippy (commonArgs // { inherit cargoArtifacts; });
  cargoDoc = craneLib.cargoDoc (commonArgs // { inherit cargoArtifacts; });
  cargoFmt = craneLib.cargoFmt commonArgs;

  registry = "did:web:localhost%3A3000";

  mkAppEnv =
    { registry }:
    {
      UNAVI_REGISTRY_DID = registry;
    };

  unaviAppArtifacts = craneLib.buildDepsOnly unaviAppConfig;
  mkUnaviApp =
    input:
    craneLib.buildPackage (unaviAppConfig // { cargoArtifacts = unaviAppArtifacts; } // mkAppEnv input);
  unavi-app = mkUnaviApp { inherit registry; };

  unaviWebArtifacts = craneLib.buildDepsOnly unaviWebConfig;
  mkUnaviWeb =
    input:
    craneLib.buildTrunkPackage (
      unaviWebConfig // { cargoArtifacts = unaviWebArtifacts; } // mkAppEnv input
    );
  unavi-web = mkUnaviWeb { inherit registry; };

  unaviServerArtifacts = craneLib.buildDepsOnly unaviServerConfig;
  mkUnaviServer =
    input:
    craneLib.buildPackage (
      unaviServerConfig
      // {
        cargoArtifacts = unaviServerArtifacts;
        postInstall = ''
          mkdir -p $out/bin
          ln -s ${mkUnaviWeb input} $out/bin/web
        '';
      }
    );
  unavi-server = mkUnaviServer { inherit registry; };
in
{
  inherit mkUnaviApp mkUnaviServer;

  buildInputs = commonArgs.buildInputs;
  nativeBuildInputs = commonArgs.nativeBuildInputs;

  apps = {
    unavi-app = flake-utils.lib.mkApp { drv = unavi-app; };
    unavi-server = flake-utils.lib.mkApp { drv = unavi-server; };
    unavi-web = flake-utils.lib.mkApp {
      drv = pkgs.writeShellScriptBin "unavi-web" ''
        ${pkgs.python3Minimal}/bin/python3 -m http.server --directory ${unavi-web} 8080
      '';
    };
  };
  checks = {
    inherit cargoClippy cargoDoc cargoFmt;
  };
  packages = {
    inherit unavi-app unavi-server unavi-web;
  };
}
