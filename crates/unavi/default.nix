{ inputs, ... }:
{
  perSystem =
    { pkgs, lib, ... }:
    let
      pname = "unavi";

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

        buildInputs = pkgs.lib.optionals pkgs.stdenv.isLinux (with pkgs; [ openssl ]);
        nativeBuildInputs = pkgs.lib.optionals pkgs.stdenv.isLinux (with pkgs; [ pkg-config ]);
      };

      src = lib.fileset.toSource rec {
        root = ../..;
        fileset = lib.fileset.unions [
          (pkgs.crane.fileset.commonCargoSources root)
          (lib.fileset.fileFilter (
            file:
            lib.any file.hasExt [
              "wit"
              "md"
            ]
          ) root)
          ../../LICENSE
          ../../scripts
          ./assets
        ];
      };

      cargoArgs = rec {
        inherit pname;
        inherit src;

        cargoExtraArgs = "-p ${pname}";
        strictDeps = true;

        runtimeDependencies = pkgs.lib.optionals pkgs.stdenv.isLinux (
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
              wasm-tools
            ]
          )
          ++ [
            wac-cli
            inputs.wit-deps.packages.${pkgs.system}.wit-deps
          ];

        buildInputs = runtimeDependencies;
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
            preBuild = ''
              ${pkgs.nushell}/bin/nu scripts/build-wasm.nu
            '';
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
