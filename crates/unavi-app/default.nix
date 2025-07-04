{
  craneLib,
  pkgs,
  src,
  wac-cli,
  ...
}:
let
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
      ++ [ wac-cli ];

    postInstall = ''
      mv $out/bin/* $out
      rm -r $out/bin
      cp -r crates/unavi-app/assets $out
      cp LICENSE $out
    '';
  };

in
{
  inherit (config) buildInputs;
  inherit (config) nativeBuildInputs;
  package = craneLib.buildPackage config;
}
