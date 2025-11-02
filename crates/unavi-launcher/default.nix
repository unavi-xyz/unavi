_:
{
  perSystem =
    { pkgs, lib, ... }:
    let
      pname = "unavi-launcher";

      src = lib.fileset.toSource rec {
        root = ../..;
        fileset = lib.fileset.unions [
          (pkgs.crane.fileset.commonCargoSources root)
          (lib.fileset.fileFilter (file: lib.any file.hasExt [ "md" ]) root)
          ../../LICENSE
        ];
      };

      cargoArgs = rec {
        inherit pname;
        inherit src;

        cargoExtraArgs = "-p ${pname}";
        strictDeps = true;

        runtimeDependencies = [ ];

        nativeBuildInputs = pkgs.lib.optionals pkgs.stdenv.isLinux (
          with pkgs;
          [
            clang
            lld
            mold
            pkg-config
          ]
        );

        buildInputs = runtimeDependencies;
      };

      cargoArtifacts = pkgs.crane.buildDepsOnly cargoArgs;

      packageDrv = pkgs.crane.buildPackage (
        cargoArgs
        // {
          inherit cargoArtifacts;
          doCheck = false;

          preBuild = ''
            ${pkgs.nushell}/bin/nu scripts/build-wasm.nu
          '';

          postInstall = ''
            mv $out/bin/* $out
            rm -r $out/bin
            cp LICENSE $out
          '';
        }
      );
    in
    {
      checks = {
        "${pname}-doc" = pkgs.crane.cargoDoc (cargoArgs // { inherit cargoArtifacts; });
        "${pname}-doctest" = pkgs.crane.cargoDocTest (cargoArgs // { inherit cargoArtifacts; });
        "${pname}-nextest" = pkgs.crane.cargoNextest (
          cargoArgs
          // {
            inherit cargoArtifacts;
            cargoExtraArgs = cargoArgs.cargoExtraArgs + " --no-tests pass";
            preBuild = ''
              ${pkgs.nushell}/bin/nu scripts/build-wasm.nu
            '';
          }
        );
      };

      packages = {
        "${pname}" = packageDrv;
      };
    };
}
