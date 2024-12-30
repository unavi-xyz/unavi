{
  craneLib,
  pkgs,
  src,
  ...
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

  config = {
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
          clang
          mold
          pkg-config
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
      cp -r crates/unavi-app/assets $out
      cp LICENSE $out
    '';
  };

in
{
  buildInputs = config.buildInputs;
  nativeBuildInputs = config.nativeBuildInputs;
  package = craneLib.buildPackage config;
}
