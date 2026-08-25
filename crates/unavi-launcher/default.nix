_: {
  perSystem =
    {
      mkAppImage,
      pkgs,
      lib,
      ...
    }:
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

        doCheck = false;

        cargoExtraArgs = "-p ${pname}";
        strictDeps = true;

        nativeBuildInputs = lib.optionals pkgs.stdenv.hostPlatform.isLinux (
          with pkgs;
          [
            clang
            lld
            pkg-config
            python3
          ]
        );

        linkedInputs = lib.optionals pkgs.stdenv.hostPlatform.isLinux (
          with pkgs;
          [
            fontconfig
            libGL
            libX11
            libXcursor
            libXi
            libXrandr
            libxcb
            libxkbcommon
            openssl
            vulkan-loader
            wayland
          ]
        );

        buildInputs = linkedInputs;
      };

      libraryPath = lib.makeLibraryPath cargoArgs.linkedInputs;

      # nix-appimage reads this out of the package to give the AppImage a name
      # and a .DirIcon, which is all a desktop has to identify it by.
      desktopItem = pkgs.makeDesktopItem {
        name = pname;
        desktopName = "UNAVI";
        comment = "Launcher for the UNAVI client";
        exec = pname;
        icon = pname;
        categories = [ "Network" ];
      };

      iconSizes = [
        256
        128
        64
      ];

      cargoArtifacts = pkgs.crane.buildDepsOnly cargoArgs;

      packageDrv = pkgs.crane.buildPackage (
        cargoArgs
        // {
          inherit cargoArtifacts;

          nativeBuildInputs = cargoArgs.nativeBuildInputs ++ [
            pkgs.imagemagick
            pkgs.makeWrapper
            pkgs.patchelf
          ];

          postInstall = ''
            cp LICENSE $out
            patchelf --set-rpath "${libraryPath}" $out/bin/${pname}
            wrapProgram $out/bin/${pname} \
              --prefix LD_LIBRARY_PATH : "${libraryPath}"

            install -Dm444 -t $out/share/applications \
              ${desktopItem}/share/applications/${pname}.desktop

            ${lib.concatMapStringsSep "\n" (size: ''
              install -dm755 $out/share/icons/hicolor/${toString size}x${toString size}/apps
              magick assets/icon-nobg.png -resize ${toString size}x${toString size} \
                $out/share/icons/hicolor/${toString size}x${toString size}/apps/${pname}.png
            '') iconSizes}
          '';
        }
      );
    in
    {
      packages = {
        "${pname}" = packageDrv;
      }
      // lib.optionalAttrs pkgs.stdenv.hostPlatform.isLinux {
        "${pname}-appimage" = mkAppImage { program = "${packageDrv}/bin/${pname}"; };
      };
    };
}
