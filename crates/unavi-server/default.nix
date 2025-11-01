_: {
  perSystem =
    { pkgs, lib, ... }:
    let
      pname = "unavi-server";

      src = lib.fileset.toSource rec {
        root = ../..;
        fileset = lib.fileset.unions [
          (pkgs.crane.fileset.commonCargoSources root)
          (lib.fileset.fileFilter (file: lib.any file.hasExt [ "json" ]) root)
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
          }
        );
      };

      packages = {
        "${pname}" = packageDrv;

        "${pname}-bundle" = pkgs.stdenv.mkDerivation {
          inherit pname;
          version = cargoArgs.pname;

          src = packageDrv;

          nativeBuildInputs = [
            pkgs.gnutar
            pkgs.gzip
          ];
          buildInputs = [ ];
          runtimeDependencies = [ ];

          installPhase = ''
            mkdir -p $out
            tar -czf $out/${pname}-${pkgs.system}.tar.gz -C $src .
          '';
        };
      };
    };
}
