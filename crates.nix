{
  flake-utils,
  wasm,
  craneLib,
  pkgs,
  ...
}:
let
  lib = pkgs.lib;

  src = lib.cleanSourceWith {
    src = ./.;
    filter =
      path: type:
      (lib.hasInfix "/crates/unavi-app/assets/" path)
      || (lib.hasInfix "/crates/unavi-app/public/" path)
      || (lib.hasSuffix ".capnp" path)
      || (lib.hasSuffix ".html" path)
      || (lib.hasSuffix ".json" path)
      || (lib.hasSuffix ".wit" path)
      || (craneLib.filterCargoSources path type);
  };

  commonNativeBuildInputs =
    (with pkgs; [
      clang
      cmake
      pkg-config
      rustPlatform.bindgenHook
    ])
    ++ lib.optionals (pkgs.stdenv.isLinux && pkgs.stdenv.hostPlatform.system != "aarch64-linux") (
      with pkgs;
      [
        glibc_multi
        glibc_multi.dev
        mold
      ]
    )
    ++ lib.optionals pkgs.stdenv.isDarwin (with pkgs; [ libiconv ]);

  unaviAppConfig = rec {
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
    nativeBuildInputs = (with pkgs; [ capnproto ]) ++ commonNativeBuildInputs;

    cargoExtraArgs = "--locked -p unavi-app";
    pname = "unavi-app";
    strictDeps = true;

    preBuild = wasm.generateAssetsScript;
    postInstall = ''
      cp -r ./crates/unavi-app/assets $out/bin
    '';

    LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
  };

  unaviWebConfig = {
    inherit src;

    buildInputs = unaviAppConfig.buildInputs;
    nativeBuildInputs =
      (with pkgs; [
        binaryen
        trunk
        wasm-bindgen-cli
        wasm-tools
      ])
      ++ unaviAppConfig.nativeBuildInputs;

    cargoExtraArgs = "--locked -p unavi-app";
    pname = "unavi-web";
    strictDeps = true;
    trunkIndexPath = "crates/unavi-app/index.html";
    wasm-bindgen-cli = pkgs.wasm-bindgen-cli;

    preBuild = wasm.generateAssetsScript;

    CARGO_PROFILE = "release-web";
  };

  unaviServerConfig = rec {
    inherit src;

    buildInputs = lib.optionals pkgs.stdenv.isLinux (
      with pkgs;
      [
        alsa-lib
        openssl
        udev
      ]
    );

    nativeBuildInputs =
      (with pkgs; [ capnproto ])
      ++ commonNativeBuildInputs
      ++ lib.optionals pkgs.stdenv.isDarwin (with pkgs; [ darwin.apple_sdk.frameworks.Cocoa ]);

    cargoExtraArgs = "--locked -p unavi-server";
    pname = "unavi-server";
    strictDeps = true;

    LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
  };

  commonArgs = {
    inherit src;

    buildInputs = unaviServerConfig.buildInputs ++ unaviWebConfig.buildInputs;
    nativeBuildInputs = unaviServerConfig.nativeBuildInputs ++ unaviWebConfig.nativeBuildInputs;

    pname = "unavi";
    strictDeps = true;
  };

  cargoArtifacts = craneLib.buildDepsOnly commonArgs;
  cargoClippy = craneLib.cargoClippy (commonArgs // { inherit cargoArtifacts; });
  cargoDoc = craneLib.cargoDoc (commonArgs // { inherit cargoArtifacts; });
  cargoFmt = craneLib.cargoFmt commonArgs;

  worldHostDid = "did:web:localhost%3A3000";

  mkAppEnv =
    { worldHostDid }:
    {
      UNAVI_WORLD_HOST_DID = worldHostDid;
    };

  unaviAppArtifacts = craneLib.buildDepsOnly unaviAppConfig;
  mkUnaviApp =
    input:
    craneLib.buildPackage (unaviAppConfig // { cargoArtifacts = unaviAppArtifacts; } // mkAppEnv input);
  unavi-app = mkUnaviApp { inherit worldHostDid; };

  mkUnaviWeb = input: craneLib.buildTrunkPackage (unaviWebConfig // mkAppEnv input);
  unavi-web = mkUnaviWeb { inherit worldHostDid; };

  unavi-server = craneLib.buildPackage unaviServerConfig;
in
{
  inherit mkUnaviApp mkUnaviWeb;

  buildInputs = commonArgs.buildInputs;
  nativeBuildInputs = commonArgs.nativeBuildInputs;

  apps = rec {
    app = flake-utils.lib.mkApp { drv = unavi-app; };
    server = flake-utils.lib.mkApp { drv = unavi-server; };
    web = flake-utils.lib.mkApp {
      drv = pkgs.writeShellApplication {
        name = "unavi-web";
        runtimeInputs = with pkgs; [ python3Minimal ];
        text = ''
          python3 -m http.server --directory ${unavi-web} 8080
        '';
      };
    };

    default = app;
  };
  checks = {
    inherit cargoClippy cargoDoc cargoFmt;
  };
  packages = {
    inherit unavi-app unavi-server unavi-web;
    default = unavi-app;
  };
}
