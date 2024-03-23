{ flake-utils, components, craneLib, localSystem, pkgs, self, ... }:
let
  lib = pkgs.lib;

  src = craneLib.cleanCargoSource (craneLib.path ./.);

  commonArgs = {
    pname = "unavi";

    src = lib.cleanSourceWith {
      src = ./.;
      filter = path: type:
        (lib.hasSuffix ".proto" path) || (lib.hasSuffix ".wit" path)
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
        clang
        cmake
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

  cargoDoc = craneLib.cargoDoc (commonArgs // { inherit cargoArtifacts; });

  cargoFmt = craneLib.cargoFmt {
    inherit src;
    pname = "unavi";
  };

  unavi-app = craneLib.buildPackage (commonArgs // {
    inherit cargoArtifacts;
    pname = "unavi-app";
    cargoExtraArgs = "--locked -p unavi-app";

    preBuild = components.generateAssetsScript;
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

    preBuild = components.generateAssetsScript;
  });
in {
  LD_LIBRARY_PATH = lib.makeLibraryPath (commonArgs.buildInputs);

  apps = {
    app = flake-utils.lib.mkApp { drv = unavi-app; };
    server = flake-utils.lib.mkApp { drv = unavi-server; };
    web = flake-utils.lib.mkApp {
      drv = pkgs.writeShellScriptBin "web" ''
        ${pkgs.python3Minimal}/bin/python3 -m http.server --directory ${web} 3000
      '';
    };
  };
  checks = { inherit cargoClippy cargoDoc cargoFmt; };
  packages = { inherit unavi-app unavi-server web; };
}
