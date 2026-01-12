{ inputs, ... }:
{
  perSystem =
    { pkgs, lib, ... }:
    let
      pname = "unavi-client";

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
              "ron"
              "wgsl"
              "wit"
            ]
          ) root)
          ../../LICENSE
          ../../scripts
          ../wds/migrations
          ./assets
        ];
      };

      cargoArgs = rec {
        inherit pname;
        inherit src;

        cargoExtraArgs = "-p ${pname}";
        strictDeps = true;

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

        linkedInputs = pkgs.lib.optionals pkgs.stdenv.isLinux (
          with pkgs;
          [
            alsa-lib
            libxkbcommon
            udev
            vulkan-loader
            wayland
            xorg.libX11
            xorg.libXcursor
            xorg.libXi
            xorg.libXrandr
          ]
        );

        buildInputs = linkedInputs;
      };

      cargoArtifacts = pkgs.crane.buildDepsOnly cargoArgs;

    in
    {
      checks = {
        "${pname}-doc" = pkgs.crane.cargoDoc (cargoArgs // { inherit cargoArtifacts; });
        "${pname}-nextest" = pkgs.crane.cargoNextest (
          cargoArgs
          // {
            inherit cargoArtifacts;
            cargoExtraArgs = cargoArgs.cargoExtraArgs + " --no-tests pass";
            preBuild = lib.concatLines [
              cargoArgs.preBuild
              ''
                ${pkgs.nushell}/bin/nu scripts/build-wasm.nu
              ''
            ];
          }
        );
      };

      packages = {
        "${pname}" = pkgs.crane.buildPackage (
          cargoArgs
          // {
            inherit cargoArtifacts;
            doCheck = false;

            preBuild = ''
              ${pkgs.nushell}/bin/nu scripts/build-wasm.nu
            '';

            postInstall = ''
              mkdir -p $out/bin/assets
              cp -r crates/${pname}/assets/* $out/bin/assets/
              rm -rf $out/bin/assets/wasm/test $out/bin/assets/wasm/example

              cp LICENSE $out
            '';
          }
        );
        "${pname}-web" = pkgs.crane.buildTrunkPackage (
          cargoArgs
          // {
            pname = "${pname}-web";
            wasm-bindgen-cli = pkgs.wasm-bindgen-cli_0_2_105;

            inherit cargoArtifacts;
            doCheck = false;

            preBuild = ''
              ${pkgs.nushell}/bin/nu scripts/build-wasm.nu
            '';

            postInstall = ''
              mkdir -p $out/bin/assets
              cp -r crates/${pname}/assets/* $out/bin/assets/
              rm -rf $out/bin/assets/wasm/test $out/bin/assets/wasm/example

              cp LICENSE $out
            '';
          }
        );
      };
    };
}
