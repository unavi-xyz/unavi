_: {
  perSystem =
    { pkgs, lib, ... }:
    let
      pname = "unavi-server";

      buildInputs = [ ];

      nativeBuildInputs = pkgs.lib.optionals pkgs.stdenv.isLinux (
        with pkgs;
        [
          clang
          lld
          mold
          pkg-config
        ]
      );

      src = lib.fileset.toSource rec {
        root = ../..;
        fileset = lib.fileset.unions [
          (pkgs.crane.fileset.commonCargoSources root)
          ../../LICENSE
        ];
      };

      cargoArgs = {
        inherit buildInputs;
        inherit nativeBuildInputs;
        inherit pname;
        inherit src;

        runtimeDependencies = buildInputs;

        cargoExtraArgs = "-p ${pname}";
        strictDeps = true;
      };

      cargoArtifacts = pkgs.crane.buildDepsOnly cargoArgs;
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

      packages."${pname}" = pkgs.crane.buildPackage (
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
    };
}
