_: {
  perSystem =
    { pkgs, lib, ... }:
    let
      pname = "unavi-server";

      src = lib.fileset.toSource rec {
        root = ../..;
        fileset = lib.fileset.unions [
          (pkgs.crane.fileset.commonCargoSources root)
          (lib.fileset.fileFilter (file: lib.any file.hasExt [ "ron" ]) root)
          ../../LICENSE
          ./secretspec.toml
        ];
      };

      cargoArgs = rec {
        inherit pname;
        inherit src;

        cargoExtraArgs = "-p ${pname}";
        strictDeps = true;

        nativeBuildInputs = pkgs.lib.optionals pkgs.stdenv.hostPlatform.isLinux (
          with pkgs;
          [
            clang
            lld
            pkg-config
          ]
        );

        linkedInputs = [ ];

        buildInputs = linkedInputs;
      };

      cargoArtifacts = pkgs.crane.buildDepsOnly cargoArgs;

      packageDrv = pkgs.crane.buildPackage (
        cargoArgs
        // {
          inherit cargoArtifacts;
          doCheck = false;

          postInstall = ''
            cp LICENSE $out
          '';
        }
      );
    in
    {
      packages = {
        "${pname}" = packageDrv;
      };
    };
}
