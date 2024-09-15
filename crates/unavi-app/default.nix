{
  craneLib,
  pkgs,
  src,
}:
let
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

  nativeConfig = rec {
    pname = "unavi-app";
    strictDeps = true;
    cargoExtraArgs = "--locked -p unavi-app";

    inherit src;

    buildInputs =
      pkgs.lib.optionals pkgs.stdenv.isLinux (
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
      ++ pkgs.lib.optionals pkgs.stdenv.isDarwin (
        with pkgs;
        [
          darwin.apple_sdk.frameworks.AudioUnit
          darwin.apple_sdk.frameworks.Cocoa
        ]
      );

    nativeBuildInputs =
      pkgs.lib.optionals pkgs.stdenv.isLinux (
        with pkgs;
        [
          alsa-lib.dev
          clang
          cmake
          libxkbcommon.dev
          openssl.dev
          pkg-config
          udev.dev
          wayland.dev
        ]
      )
      ++ (with pkgs; [
        capnproto
        cargo-component
        rustPlatform.bindgenHook
      ])
      ++ [
        wac-cli
      ];

    postInstall = ''
      mv $out/bin/* $out
      rm -r $out/bin
      cp -r crates/unavi-app/assets $out
      cp crates/unavi-app/README.md $out
      cp LICENSE $out
    '';

    LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
  };

  webConfig = rec {
    pname = "unavi-web";
    strictDeps = true;
    cargoExtraArgs = "--locked -p unavi-app";

    inherit src;

    trunkIndexPath = "crates/unavi-app/index.html";
    wasm-bindgen-cli = pkgs.wasm-bindgen-cli;

    buildInputs = nativeConfig.buildInputs;
    nativeBuildInputs =
      (with pkgs; [
        binaryen
        trunk
        wasm-tools
      ])
      ++ [ wasm-bindgen-cli ]
      ++ nativeConfig.nativeBuildInputs;

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

  worldHostDid = "did:web:localhost%3A3000";

  mkEnv =
    { worldHostDid }:
    {
      UNAVI_WORLD_HOST_DID = worldHostDid;
    };

  mkNative = input: craneLib.buildPackage (nativeConfig // mkEnv input);
  mkWeb = input: craneLib.buildTrunkPackage (webConfig // mkEnv input);
in
{
  inherit mkNative mkWeb;

  buildInputs = webConfig.buildInputs;
  nativeBuildInputs = webConfig.nativeBuildInputs;

  native = mkNative { inherit worldHostDid; };
  web = mkWeb { inherit worldHostDid; };
}
