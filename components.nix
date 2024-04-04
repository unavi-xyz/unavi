{ craneLib, flake-utils, localSystem, pkgs, self, ... }:
let
  lib = pkgs.lib;

  buildComponent = pname:
    craneLib.buildPackage ({
      inherit pname;

      src = lib.cleanSourceWith {
        src = ./.;
        filter = path: type:
          (lib.hasSuffix ".wit" path)
          || (craneLib.filterCargoSources path type);
      };

      nativeBuildInputs = with pkgs; [ cargo-component ];

      cargoBuildCommand = "cargo component build --profile wasm-release";
      cargoExtraArgs = "--locked -p ${pname}";
      doCheck = false;
      strictDeps = true;
    });

  componentNames =
    lib.mapAttrsToList (name: _: name) (builtins.readDir ./components);

  components = (map (name: buildComponent name) componentNames);

  generateAssetsScript = ''
    rm -rf assets/components
    mkdir -p assets/components
    cp -r --no-preserve=mode ${
      self.packages.${localSystem}.components
    }/lib/* assets/components
  '';
in {
  inherit generateAssetsScript;

  apps = {
    check-components = flake-utils.lib.mkApp {
      drv = pkgs.writeShellScriptBin "check-components"
        (lib.concatStringsSep " -p "
          ([ "cargo component check --locked" ] ++ componentNames));
    };

    generate-assets = flake-utils.lib.mkApp {
      drv = pkgs.writeShellScriptBin "generate-assets" generateAssetsScript;
    };
  };

  packages = {
    components = pkgs.symlinkJoin {
      name = "components";
      paths = [ components ];
    };
  };
}
