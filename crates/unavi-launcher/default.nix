_: {
  perSystem =
    { pkgs, lib, ... }:
    let
      pname = "unavi-launcher";

      src = lib.fileset.toSource rec {
        root = ../..;
        fileset = lib.fileset.unions [
          (pkgs.crane.fileset.commonCargoSources root)
          (lib.fileset.fileFilter (
            file:
            lib.any file.hasExt [
              "css"
              "ico"
              "png"
            ]
          ) root)
          ../../LICENSE
        ];
      };

      cargoArgs = rec {
        inherit pname;
        inherit src;

        cargoExtraArgs = "-p ${pname}";
        strictDeps = true;

        nativeBuildInputs = pkgs.lib.optionals pkgs.stdenv.isLinux (
          with pkgs;
          [
            clang
            dioxus-cli
            lld
            mold
            pkg-config
            python3
          ]
        );

        linkedInputs = with pkgs; [
          bzip2
          cairo
          gdk-pixbuf
          glib
          gtk3
          libsoup_3
          pango
          webkitgtk_4_1
          xdotool
        ];

        buildInputs =
          (with pkgs; [
            at-spi2-atk
            atkmm
            harfbuzz
            libiconv
            librsvg
          ])
          ++ linkedInputs;
      };

      cargoArtifacts = pkgs.crane.buildDepsOnly cargoArgs;

      packageDrv = pkgs.crane.buildPackage (
        cargoArgs
        // rec {
          inherit cargoArtifacts;
          doCheck = false;

          cargoBuildCommand = ''
            dx build -p ${pname} --release
          '';

          buildPhaseCargoCommand = ''
            ${cargoBuildCommand}
          '';

          doNotPostBuildInstallCargoBinaries = true;

          installPhaseCommand = ''
            mkdir -p $out
            cp -r target/dx/${pname}/release/**/app/* $out
          '';

          postInstall = ''
            strip $out/${pname}
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
      };
    };
}
