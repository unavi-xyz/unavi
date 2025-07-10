_: {
  perSystem =
    { pkgs, lib, ... }:
    let
      wac-cli = pkgs.rustPlatform.buildRustPackage rec {
        pname = "wac-cli";
        version = "0.6.1";

        src = pkgs.fetchFromGitHub {
          owner = "bytecodealliance";
          repo = "wac";
          rev = "v${version}";
          sha256 = "sha256-noBVAhoHXl3FI6ZlnmCwpnqu7pub6FCtuY+026vdlYo=";
        };

        cargoHash = "sha256-5oLt1wnadtEKCOAtpbzPQRuU76qLWRtcCv6Jcozon4E=";

        nativeBuildInputs = [ pkgs.pkg-config ];

        buildInputs =
          [ pkgs.openssl ]
          ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [ pkgs.darwin.apple_sdk.frameworks.SystemConfiguration ];
      };

      buildInputs =
        pkgs.lib.optionals pkgs.stdenv.isLinux (
          with pkgs;
          [
            alsa-lib
            libxkbcommon
            openssl
            udev
            vulkan-loader
            wayland
            xorg.libX11
            xorg.libXcursor
            xorg.libXi
            xorg.libXrandr
          ]
        )
        ++ pkgs.lib.optionals pkgs.stdenv.isDarwin (
          with pkgs;
          [
            darwin.apple_sdk.frameworks.AudioUnit
            darwin.apple_sdk.frameworks.Cocoa
          ]
        );

      nativeBuildInputs =
        pkgs.lib.optionals pkgs.stdenv.isLinux (
          with pkgs;
          [
            clang
            mold
            pkg-config
          ]
        )
        ++ [ wac-cli ];

      src = lib.fileset.toSource rec {
        root = ../..;
        fileset = lib.fileset.unions [
          (pkgs.crane.fileset.commonCargoSources root)
          ../../LICENSE
          ./assets
          (lib.fileset.fileFilter (
            file:
            lib.any file.hasExt [
              "wit"
              "md"
            ]
          ) root)
        ];
      };

      cargoArgs = {
        inherit src;
        inherit buildInputs;
        inherit nativeBuildInputs;

        pname = "unavi-app";
        cargoExtraArgs = "-p unavi-app";
        strictDeps = true;
      };

      cargoArtifacts = pkgs.crane.buildDepsOnly cargoArgs;
    in
    {
      checks = {
        unavi-app-doc = pkgs.crane.cargoDoc (cargoArgs // { inherit cargoArtifacts; });
        unavi-app-doctest = pkgs.crane.cargoDocTest (cargoArgs // { inherit cargoArtifacts; });
        unavi-app-nextest = pkgs.crane.cargoNextest (
          cargoArgs
          // {
            inherit cargoArtifacts;
            cargoExtraArgs = cargoArgs.cargoExtraArgs + " --no-tests pass";
          }
        );
      };

      packages.app = pkgs.crane.buildPackage (cargoArgs // {
        inherit cargoArtifacts;
        doCheck = false;
        postInstall = ''
          mv $out/bin/* $out
          rm -r $out/bin
          cp -r crates/unavi-app/assets $out
          cp LICENSE $out
        '';
      });
    };
}
