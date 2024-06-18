{
  craneLib,
  flake-utils,
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

    buildInputs = [
      pkgs.openssl
    ] ++ lib.optionals pkgs.stdenv.isDarwin [ pkgs.darwin.apple_sdk.frameworks.SystemConfiguration ];
  };

  crate = craneLib.crateNameFromCargoToml { cargoToml = ./Cargo.toml; };

  buildComponent =
    pname:
    craneLib.buildPackage {
      inherit pname;

      src = lib.cleanSourceWith {
        src = ./.;
        filter = path: type: (lib.hasSuffix ".wit" path) || (craneLib.filterCargoSources path type);
      };

      nativeBuildInputs = with pkgs; [
        cargo-component
        clang
        mold
      ];

      cargoBuildCommand = "cargo component build --profile release-wasm";
      cargoExtraArgs = "--locked -p ${pname}";
      doCheck = false;
      strictDeps = true;
    };

  componentNames = lib.mapAttrsToList (name: _: name) (builtins.readDir ./wasm);

  components = pkgs.symlinkJoin {
    name = "components";
    paths = map buildComponent componentNames;

    nativeBuildInputs = [
      wac-cli
      pkgs.wasm-tools
    ];

    postBuild =
      let
        processName =
          name:
          let
            parts = lib.strings.splitString "-" name;
            namespace = builtins.elemAt parts 0;
            packageParts = builtins.tail parts;
            package = lib.concatStringsSep "-" packageParts;

            inName = lib.replaceStrings [ "-" ] [ "_" ] name;

            inFile = "$out/lib/${inName}.wasm";
            outDir = "$out/lib/${crate.version}/${namespace}";
            outFile = "${outDir}/${package}.wasm";
          in
          {
            inherit inFile outDir outFile;
          };
      in
      lib.concatStrings (
        map (
          name:
          let
            processed = processName name;
          in
          ''
            mkdir -p ${processed.outDir}
            mv ${processed.inFile} ${processed.outFile}
          ''
        ) componentNames
        ++ map (
          name:
          let
            processed = processName name;
            tmpFile = "${processed.outFile}.tmp";
          in
          lib.concatStrings (
            map (
              targetName:
              let
                targetProcessed = processName targetName;
              in
              ''
                (wac plug --plug ${targetProcessed.outFile} -o ${tmpFile} ${processed.outFile} && mv ${tmpFile} ${processed.outFile}) || true
              ''
            ) componentNames
          )
        ) componentNames
      );
  };

  assetOut = "crates/unavi-app/assets/components";
  generateAssetsScript = ''
    rm -rf ${assetOut}
    mkdir -p ${assetOut}
    cp -r --no-preserve=mode ${components}/lib/* ${assetOut}
  '';
in
{
  inherit generateAssetsScript wac-cli;

  apps = {
    check-components = flake-utils.lib.mkApp {
      drv = pkgs.writeShellApplication {
        name = "check-components";
        runtimeInputs = with pkgs; [
          cargo-component
          clang
          mold
          rust-bin.stable.latest.default
        ];
        text = lib.concatStringsSep " -p " ([ "cargo component check --locked" ] ++ componentNames);
      };
    };

    generate-assets = flake-utils.lib.mkApp {
      drv = pkgs.writeShellApplication {
        name = "generate-assets";
        text = generateAssetsScript;
      };
    };
  };

  packages = {
    inherit components;
  };
}
