{ flake-utils, components, craneLib, localSystem, pkgs, self, ... }:
let
  lib = pkgs.lib;

  src = craneLib.cleanCargoSource (craneLib.path ./.);

  unaviAppConfig = {
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
      (with pkgs; [ darwin.apple_sdk.frameworks.Cocoa libiconv ]);

    nativeBuildInputs = with pkgs;
      [ clang cmake pkg-config rustPlatform.bindgenHook ]
      ++ lib.optionals (!pkgs.stdenv.isDarwin)
      (with pkgs; [ alsa-lib alsa-lib.dev ]);
  };

  unaviServerConfig = {
    buildInputs = (with pkgs; [ openssl ]);
    nativeBuildInputs = with pkgs; [
      clang
      cmake
      pkg-config
      rustPlatform.bindgenHook
    ];
  };

  unaviWebConfig = {
    buildInputs = unaviAppConfig.buildInputs;
    nativeBuildInputs = unaviAppConfig.nativeBuildInputs
      ++ (with pkgs; [ binaryen trunk wasm-bindgen-cli wasm-tools ]);
  };

  commonArgs = {
    pname = "unavi";

    src = lib.cleanSourceWith {
      src = ./.;
      filter = path: type:
        (lib.hasSuffix ".proto" path) || (lib.hasSuffix ".wit" path)
        || (craneLib.filterCargoSources path type);
    };

    strictDeps = true;

    buildInputs = unaviAppConfig.buildInputs ++ unaviServerConfig.buildInputs;
    nativeBuildInputs = unaviServerConfig.nativeBuildInputs
      ++ unaviWebConfig.nativeBuildInputs;
  };

  cargoArtifacts = craneLib.buildDepsOnly commonArgs;

  cargoClippy =
    craneLib.cargoClippy (commonArgs // { inherit cargoArtifacts; });

  cargoDoc = craneLib.cargoDoc (commonArgs // { inherit cargoArtifacts; });

  cargoFmt = craneLib.cargoFmt {
    inherit src;
    pname = "unavi";
  };

  unavi-app = craneLib.buildPackage (unaviAppConfig // {
    src = lib.cleanSourceWith {
      src = ./.;
      filter = path: type:
        (lib.hasSuffix ".wit" path) || (lib.hasInfix "/assets/" path)
        || (craneLib.filterCargoSources path type);
    };

    pname = "unavi-app";
    cargoExtraArgs = "--locked -p unavi-app";

    preBuild = components.generateAssetsScript;
    postInstall = ''
      cp -r assets $out/bin
    '';
  });

  unavi-server = craneLib.buildPackage (unaviServerConfig // {
    src = commonArgs.src;
    pname = "unavi-server";
    cargoExtraArgs = "--locked -p unavi-server";
  });

  web = craneLib.buildTrunkPackage (unaviWebConfig // {
    src = lib.cleanSourceWith {
      src = ./.;
      filter = path: type:
        (lib.hasSuffix ".wit" path) || (lib.hasSuffix ".html" path)
        || (lib.hasInfix "/assets/" path)
        || (lib.hasInfix "/crates/unavi-app/public/" path)
        || (craneLib.filterCargoSources path type);
    };

    cargoExtraArgs = "--locked -p unavi-app";
    pname = "web";
    trunkIndexPath = "./crates/unavi-app/index.html";
    wasm-bindgen-cli = pkgs.wasm-bindgen-cli;

    preBuild = components.generateAssetsScript;
  });
in {
  buildInputs = commonArgs.buildInputs;
  nativeBuildInputs = commonArgs.nativeBuildInputs;

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
