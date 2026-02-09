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
          ../../Trunk.toml
          ../../scripts
          ../wds/migrations
          ./assets
          ./index.html
          ./loader.html
          ./public
        ];
      };

      cargoArgs = rec {
        inherit pname;
        inherit src;

        doCheck = false;

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
            openxr-loader
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
      cargoArtifactsWeb = pkgs.crane.buildDepsOnly cargoArgs // {
        CARGO_BUILD_TARGET = "wasm32-unknown-unknown";
        doCheck = false;
      };

      nativeArgs = cargoArgs // {
        inherit cargoArtifacts;

        preBuild = ''
          ${pkgs.nushell}/bin/nu scripts/build-wasm.nu
        '';

        postInstall = ''
          mkdir -p $out/bin/assets
          cp -r crates/${pname}/assets/* $out/bin/assets/
          rm -rf $out/bin/assets/wasm/test $out/bin/assets/wasm/example
          cp LICENSE $out
        '';
      };

      webArgs = cargoArgs // {
        cargoArtifacts = cargoArtifactsWeb;

        nativeBuildInputs = cargoArgs.nativeBuildInputs ++ [ pkgs.trunk ];
        wasm-bindgen-cli = pkgs.wasm-bindgen-cli_0_2_106;

        preBuild = ''
          ls -l
          ${pkgs.nushell}/bin/nu scripts/build-wasm.nu
        '';

        buildPhaseCargoCommand = ''
          ${pkgs.nushell}/bin/nu scripts/build-web.nu --release
        '';

        installPhaseCommand = ''
          cp -r dist $out
        '';

        postInstall = ''
          cp LICENSE $out
        '';

        CC_wasm32_unknown_unknown = "${pkgs.llvmPackages_21.clang-unwrapped}/bin/clang";
        AR_wasm32_unknown_unknown = "${pkgs.llvmPackages_21.llvm}/bin/llvm-ar";
        CFLAGS_wasm32_unknown_unknown = "--target=wasm32 -O3 -isystem ${pkgs.llvmPackages_21.libclang.lib}/lib/clang/21/include";
      };

      channels = lib.filter (c: deployInfo.${c} ? services) (builtins.attrNames deployInfo);

      mkNativePackage =
        channel:
        pkgs.crane.buildPackage (
          nativeArgs
          // {
            pname = "${pname}-${channel}";
            UNAVI_REMOTE_WDS = remoteWds channel;
          }
        );

      mkWebPackage =
        channel:
        pkgs.crane.buildTrunkPackage (
          webArgs
          // {
            pname = "${pname}-web-${channel}";
            UNAVI_REMOTE_WDS = remoteWds channel;
          }
        );

      channelPackages = lib.listToAttrs (
        lib.concatMap (c: [
          {
            name = "${pname}-${c}";
            value = mkNativePackage c;
          }
          {
            name = "${pname}-web-${c}";
            value = mkWebPackage c;
          }
        ]) channels
      );
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

      packages = {
        "${pname}" = pkgs.crane.buildPackage nativeArgs;
        "${pname}-web" = pkgs.crane.buildTrunkPackage (webArgs // { pname = "${pname}-web"; });
      }
      // channelPackages;
    };
}
