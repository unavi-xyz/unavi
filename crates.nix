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

  clibs = with pkgs; [
    clang
    cmake
    glibc_multi
    glibc_multi.dev
    libcxx
    pkg-config
    rustPlatform.bindgenHook
  ];

  unaviAppConfig = {
    buildInputs =
      lib.optionals pkgs.stdenv.isLinux (
        with pkgs;
        [
          alsa-lib
          libxkbcommon
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
          darwin.apple_sdk.frameworks.Cocoa
          libiconv
        ]
      );

    nativeBuildInputs = with pkgs; [ alsa-lib.dev ] ++ clibs;
  };

  unaviServerConfig = {
    nativeBuildInputs = with pkgs; [ openssl.dev ] ++ clibs;
  };

  unaviWebConfig = {
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
  };

  commonArgs = {
    buildInputs = unaviAppConfig.buildInputs;
    nativeBuildInputs = unaviServerConfig.nativeBuildInputs ++ unaviWebConfig.nativeBuildInputs;
    pname = "unavi";
    strictDeps = true;

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
  };

  cargoArtifacts = craneLib.buildDepsOnly commonArgs;
  cargoClippy = craneLib.cargoClippy (commonArgs // { inherit cargoArtifacts; });
  cargoDoc = craneLib.cargoDoc (commonArgs // { inherit cargoArtifacts; });
  cargoFmt = craneLib.cargoFmt commonArgs;

  unavi-app = craneLib.buildPackage (
    unaviAppConfig
    // {
      src = commonArgs.src;

      cargoExtraArgs = "--locked -p unavi-app";
      pname = "unavi-app";
      strictDeps = true;

      preBuild = components.generateAssetsScript;
      postInstall = ''
        cp -r assets $out/bin
      '';
    }
  );

  unavi-server = craneLib.buildPackage (
    unaviServerConfig
    // {
      src = commonArgs.src;

      cargoExtraArgs = "--locked -p unavi-server";
      pname = "unavi-server";
      strictDeps = true;
    }
  );

  web = craneLib.buildTrunkPackage (
    unaviWebConfig
    // {
      src = commonArgs.src;

      cargoExtraArgs = "--locked -p unavi-app";
      pname = "web";
      strictDeps = true;
      trunkIndexPath = "./crates/unavi-app/index.html";
      wasm-bindgen-cli = pkgs.wasm-bindgen-cli;

      preBuild = components.generateAssetsScript;
    }
  );
in
{
  buildInputs = commonArgs.buildInputs;
  nativeBuildInputs = commonArgs.nativeBuildInputs;

  apps = {
    app = flake-utils.lib.mkApp { drv = unavi-app; };
    server = flake-utils.lib.mkApp { drv = unavi-server; };
    web = flake-utils.lib.mkApp {
      drv = pkgs.writeShellScriptBin "web" ''
        ${pkgs.python3Minimal}/bin/python3 -m http.server --directory ${web} 8080
      '';
    };
  };
  checks = {
    inherit cargoClippy cargoDoc cargoFmt;
  };
  packages = {
    inherit unavi-app unavi-server web;
  };
}
