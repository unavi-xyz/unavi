{
  craneLib,
  pkgs,
  src,
}:
let
  buildInputs = pkgs.lib.optionals pkgs.stdenv.isLinux (
    with pkgs;
    [
      openssl
      udev
    ]
  );

  nativeBuildInputs =
    (with pkgs; [
      capnproto
      clang
      cmake
      openssl.dev
      pkg-config
      rustPlatform.bindgenHook
      udev.dev
    ])
    ++ pkgs.lib.optionals pkgs.stdenv.isDarwin (with pkgs; [ darwin.apple_sdk.frameworks.Cocoa ]);
in
{
  inherit buildInputs nativeBuildInputs;

  server = craneLib.buildPackage rec {
    pname = "unavi-server";
    strictDeps = true;
    cargoExtraArgs = "--locked -p unavi-server";

    inherit
      src
      buildInputs
      nativeBuildInputs
      ;

    postInstall = ''
      mv $out/bin/* $out
      rm -r $out/bin
      cp crates/unavi-server/README.md $out
      cp LICENSE $out
    '';

    LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
  };
}
