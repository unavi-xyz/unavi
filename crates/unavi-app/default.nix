{ inputs, ... }:
{
  perSystem =
    { pkgs, lib, ... }:
    let
      pname = "unavi-app";

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

        nativeBuildInputs = with pkgs; [ pkg-config ];

        buildInputs =
          (with pkgs; [ openssl ])
          ++ lib.optionals pkgs.stdenv.isDarwin [ pkgs.darwin.apple_sdk.frameworks.SystemConfiguration ];
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
            binaryen
            clang
            lld
            mold
            pkg-config
          ]
        )
        ++ [
          wac-cli
          inputs.wit-deps.packages.${pkgs.system}.wit-deps
        ];

      src = lib.fileset.toSource rec {
        root = ../..;
        fileset = lib.fileset.unions [
          (pkgs.crane.fileset.commonCargoSources root)
          ../../LICENSE
          ../../scripts
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

          preBuild = ''
            ${pkgs.nushell}/bin/nu scripts/build-wasm.nu
            ${pkgs.nushell}/bin/nu scripts/optimize-wasm.nu
          '';

          postInstall = ''
            mv $out/bin/* $out
            rm -r $out/bin
            cp -r crates/${pname}/assets $out
            cp LICENSE $out
          '';
        }
      );
    };
}
