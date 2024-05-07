{
  craneLib,
  flake-utils,
  localSystem,
  pkgs,
  self,
  ...
}:
let
  lib = pkgs.lib;

  crate = craneLib.crateNameFromCargoToml { cargoToml = ./Cargo.toml; };

  buildComponent =
    pname:
    craneLib.buildPackage {
      inherit pname;

      src = lib.cleanSourceWith {
        src = ./.;
        filter = path: type: (lib.hasSuffix ".wit" path) || (craneLib.filterCargoSources path type);
      };

      nativeBuildInputs = with pkgs; [ cargo-component ];

      cargoBuildCommand = "cargo component build --profile wasm-release --target wasm32-unknown-unknown";
      cargoExtraArgs = "--locked -p ${pname}";
      doCheck = false;
      strictDeps = true;
    };

  componentNames = lib.mapAttrsToList (name: _: name) (builtins.readDir ./wasm);

  components = pkgs.symlinkJoin {
    name = "components";
    paths = map buildComponent componentNames;

    nativeBuildInputs = with pkgs; [ wasm-tools ];

    postBuild =
      let
        # We must manually keep this updated for WASM component dependencies.
        # `wasm-tools compose` says it can search directories automatically for dependencies,
        # but I've never gotten it to work.
        #
        # This could maybe be automated if you extract the interface from each component wit definition.
        #
        # Example:
        # dependencies:
        #   unavi:ecs/api: ${buildComponent "unavi-ecs"}/lib/unavi_ecs.wasm
        config = pkgs.writeText "config.yml" '''';
      in
      lib.concatStrings (
        map (
          name:
          let
            wasm_name = lib.replaceStrings [ "-" ] [ "_" ] name;
            out_name = "${wasm_name}_${crate.version}";
          in
          ''
            (wasm-tools compose --config ${config} -o $out/lib/${out_name}.wasm $out/lib/${wasm_name}.wasm && rm $out/lib/${wasm_name}.wasm) || \
            mv $out/lib/${wasm_name}.wasm $out/lib/${out_name}.wasm
          ''
        ) componentNames
      );
  };

  assetOut = "apps/unavi-app/assets/components";
  generateAssetsScript = ''
    rm -rf ${assetOut}
    mkdir -p ${assetOut}
    cp -r --no-preserve=mode ${self.packages.${localSystem}.components}/lib/* ${assetOut}
  '';
in
{
  inherit generateAssetsScript;

  apps = {
    check-components = flake-utils.lib.mkApp {
      drv = pkgs.writeShellScriptBin "check-components" (
        lib.concatStringsSep " -p " ([ "cargo component check --locked" ] ++ componentNames)
      );
    };

    generate-assets = flake-utils.lib.mkApp {
      drv = pkgs.writeShellScriptBin "generate-assets" generateAssetsScript;
    };
  };

  packages = {
    inherit components;
  };
}
