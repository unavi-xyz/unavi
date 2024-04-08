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

  clibs =
    lib.optionals (pkgs.stdenv.isLinux && pkgs.stdenv.hostPlatform.system != "aarch64-linux") (
      with pkgs;
      [
        glibc_multi
        glibc_multi.dev
      ]
    )
    ++ lib.optionals pkgs.stdenv.isLinux (
      with pkgs;
      [
        clang
        cmake
        libcxx
        pkg-config
        rustPlatform.bindgenHook
      ]
    );

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

    nativeBuildInputs = lib.optionals pkgs.stdenv.isLinux (with pkgs; [ alsa-lib.dev ]) ++ clibs;
  };

  unaviServerConfig = {
    buildInputs = with pkgs; [ openssl.dev ];
    nativeBuildInputs = clibs;
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
    buildInputs = unaviAppConfig.buildInputs ++ unaviServerConfig.buildInputs;
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

  unavi-server = craneLib.buildPackage (
    unaviServerConfig
    // {
      src = commonArgs.src;

      cargoExtraArgs = "--locked -p unavi-server";
      pname = "unavi-server";
      strictDeps = true;

      postInstall = ''
        ln -s ${web} $out/bin/web
      '';
    }
  );
in
{
  buildInputs = commonArgs.buildInputs;
  nativeBuildInputs = commonArgs.nativeBuildInputs;

  apps = {
    unavi-app = flake-utils.lib.mkApp { drv = unavi-app; };
    unavi-server = flake-utils.lib.mkApp { drv = unavi-server; };
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
