{ inputs, deployInfo, ... }:
{
  perSystem =
    { pkgs, lib, ... }:
    let
      pname = "unavi-client";

      remoteWds = channel: "did:web:${deployInfo.${channel}.services.unavi_server.domain}";

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
          ./index.html
          ./public
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
            preBuild = "${pkgs.nushell}/bin/nu scripts/build-wasm.nu";
          }
        );
      };

      packages =
        let
          webArgs = {
            wasm-bindgen-cli = pkgs.wasm-bindgen-cli_0_2_105;
            trunkIndexPath = "./crates/unavi-client/index.html";

            inherit cargoArtifacts;
            doCheck = false;

            preBuild = ''
              ${pkgs.nushell}/bin/nu scripts/build-wasm.nu
            '';

            postInstall = ''
              cp LICENSE $out
            '';

            CC_wasm32_unknown_unknown = "${pkgs.llvmPackages_21.clang-unwrapped}/bin/clang";
            AR_wasm32_unknown_unknown = "${pkgs.llvmPackages_21.llvm}/bin/llvm-ar";
            CFLAGS_wasm32_unknown_unknown = "--target=wasm32 -O3 -isystem ${pkgs.llvmPackages_21.libclang.lib}/lib/clang/21/include";
          };
        in
        {
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

          "${pname}-web" = pkgs.crane.buildTrunkPackage (cargoArgs // webArgs // { pname = "${pname}-web"; });
          "${pname}-web-beta" = pkgs.crane.buildTrunkPackage (
            cargoArgs
            // webArgs
            // {
              pname = "${pname}-web-beta";
              REMOTE_WDS = remoteWds "beta";
            }
          );
          "${pname}-web-stable" = pkgs.crane.buildTrunkPackage (
            cargoArgs
            // webArgs
            // {
              pname = "${pname}-web-stable";
              REMOTE_WDS = remoteWds "stable";
            }
          );
        };
    };
}
