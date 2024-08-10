{
  flake-utils,
  craneLib,
  pkgs,
  ...
}:
let
  lib = pkgs.lib;

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

  src = lib.cleanSourceWith {
    src = ./.;
    filter =
      path: type:
      (lib.hasInfix "crates/unavi-app/assets/" path)
      || (lib.hasInfix "crates/unavi-app/public/" path)
      || (lib.hasInfix "wired-protocol" path)
      || (lib.hasSuffix ".capnp" path)
      || (lib.hasSuffix ".html" path)
      || (lib.hasSuffix ".json" path)
      || (lib.hasSuffix ".wit" path)
      || (lib.hasSuffix "LICENSE" path)
      || (lib.hasSuffix "README.md" path)
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
    pname = "unavi-app";
    strictDeps = true;
    cargoExtraArgs = "--locked -p unavi-app";

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
      (with pkgs; [
        capnproto
        cargo-component
      ])
      ++ [ wac-cli ]
      ++ commonNativeBuildInputs;

    postInstall = ''
      mv $out/bin/* $out
      rm -r $out/bin
      cp -r crates/unavi-app/assets $out
      cp crates/unavi-app/README.md $out
      cp LICENSE $out
    '';

    LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
  };

  unaviWebConfig = rec {
    pname = "unavi-web";
    strictDeps = true;
    cargoExtraArgs = "--locked -p unavi-app";

    inherit src;

    trunkIndexPath = "crates/unavi-app/index.html";
    wasm-bindgen-cli = pkgs.wasm-bindgen-cli;

    buildInputs = unaviAppConfig.buildInputs;
    nativeBuildInputs =
      (with pkgs; [
        binaryen
        trunk
        wasm-tools
      ])
      ++ [ wasm-bindgen-cli ]
      ++ unaviAppConfig.nativeBuildInputs;

    postInstall = ''
      cp -r crates/unavi-app/assets $out
      cp LICENSE $out
    '';

    CARGO_PROFILE = "release-wasm";

    cargoArtifacts = craneLib.buildDepsOnly {
      inherit pname;
      inherit strictDeps;
      inherit src;
      inherit buildInputs;
      inherit nativeBuildInputs;
      inherit CARGO_PROFILE;
    };
  };

  unaviServerConfig = rec {
    pname = "unavi-server";
    strictDeps = true;
    cargoExtraArgs = "--locked -p unavi-server";

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

    postInstall = ''
      mv $out/bin/* $out
      rm -r $out/bin
      cp crates/unavi-server/README.md $out
      cp LICENSE $out
    '';

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
  cargoFmt = craneLib.cargoFmt (commonArgs // { inherit cargoArtifacts; });
  cargoTarpaulin = craneLib.cargoTarpaulin (commonArgs // { inherit cargoArtifacts; });

  worldHostDid = "did:web:localhost%3A3000";

  mkAppEnv =
    { worldHostDid }:
    {
      UNAVI_WORLD_HOST_DID = worldHostDid;
    };

  mkUnaviApp =
    input: craneLib.buildPackage (unaviAppConfig // { inherit cargoArtifacts; } // mkAppEnv input);
  unavi-app = mkUnaviApp { inherit worldHostDid; };

  mkUnaviWeb = input: craneLib.buildTrunkPackage (unaviWebConfig // mkAppEnv input);
  unavi-web = mkUnaviWeb { inherit worldHostDid; };

  unavi-server = craneLib.buildPackage (unaviServerConfig // { inherit cargoArtifacts; });
in
{
  inherit mkUnaviApp mkUnaviWeb;

  buildInputs = commonArgs.buildInputs;
  nativeBuildInputs = commonArgs.nativeBuildInputs;

  checks = {
    inherit
      cargoClippy
      cargoDoc
      cargoFmt
      cargoTarpaulin
      ;
  };

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

  packages = {
    inherit unavi-app unavi-server unavi-web;
    default = unavi-app;
  };
}
