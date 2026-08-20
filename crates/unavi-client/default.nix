{ inputs, deployInfo, ... }: {
  perSystem =
    { pkgs, lib, ... }:
    let
      pname = "unavi-client";

      # Overrides the default the manifest declares, which the build script
      # compiles in.
      channelSyncTargets = channel: "did:web:${deployInfo.${channel}.services.unavi_server.domain}";

      # `wasm-bindgen` refuses a module whose schema came from a different
      # version, and the sandbox cannot fetch the matching CLI the way Trunk
      # would, so the tool is picked from the lockfile rather than pinned by
      # hand where it would drift on the next `cargo update`.
      wasmBindgenVersion =
        (builtins.fromTOML (builtins.readFile ../../Cargo.lock)).package
        |> lib.findFirst (p: p.name == "wasm-bindgen") (throw "Cargo.lock has no wasm-bindgen")
        |> (p: p.version);

      wasmBindgenAttr = "wasm-bindgen-cli_${builtins.replaceStrings [ "." ] [ "_" ] wasmBindgenVersion}";

      wasm-bindgen-cli =
        pkgs.${wasmBindgenAttr}
          or (throw "nixpkgs has no `${wasmBindgenAttr}`, which Cargo.lock's wasm-bindgen ${wasmBindgenVersion} requires");

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
          ../../wasm
          ../unavi-script/package.json
          ../unavi-script/package-lock.json
          ../unavi-script/runtime.ts
          ../wds/migrations
          ./assets
          ./index.html
          ./loader.html
          ./public
          ./secretspec.toml
        ];
      };

      npmSrc = lib.fileset.toSource {
        root = ../unavi-script;
        fileset = lib.fileset.unions [
          ../unavi-script/package.json
          ../unavi-script/package-lock.json
        ];
      };

      npmDeps = pkgs.fetchNpmDeps {
        src = npmSrc;
        hash = "sha256-0Otjim245LzzRJFMf0mmK+1iZsuoJs8LsG4Y4FNevh0=";
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
              esbuild
              lld
              pkg-config
              wac-cli
              wasm-tools
            ]
          )
          ++ [ inputs.wit-deps.packages.${pkgs.system}.wit-deps ];

        linkedInputs = pkgs.lib.optionals pkgs.stdenv.isLinux (
          with pkgs;
          [
            alsa-lib
            libxkbcommon
            openxr-loader
            udev
            vulkan-loader
            wayland
            libX11
            libXcursor
            libXi
            libXrandr
          ]
        );

        buildInputs = linkedInputs;
      };

      cargoArtifacts = pkgs.crane.buildDepsOnly cargoArgs;
      cargoArtifactsWeb = pkgs.crane.buildDepsOnly (
        cargoArgs
        // {
          pname = "${pname}-web";
          CARGO_BUILD_TARGET = "wasm32-unknown-unknown";
        }
      );

      nativeArgs = cargoArgs // {
        inherit cargoArtifacts;

        npmRoot = "crates/unavi-script";
        inherit npmDeps;

        nativeBuildInputs = cargoArgs.nativeBuildInputs ++ [
          pkgs.makeWrapper
          pkgs.nodejs
          pkgs.npmHooks.npmConfigHook
          pkgs.nushell
        ];

        preBuild = ''
          nu scripts/update-wasm.nu
          nu scripts/build-wasm.nu
        '';

        postInstall = ''
          mkdir -p $out/bin/assets
          cp -r crates/${pname}/assets/* $out/bin/assets/
          rm -rf $out/bin/assets/wasm/test $out/bin/assets/wasm/example
          cp LICENSE $out
          wrapProgram $out/bin/${pname} \
            --prefix LD_LIBRARY_PATH : "${pkgs.lib.makeLibraryPath cargoArgs.linkedInputs}"
        '';
      };

      webArgs = cargoArgs // {
        cargoArtifacts = cargoArtifactsWeb;

        npmRoot = "crates/unavi-script";
        inherit npmDeps;

        # Trunk's own `post_build` hook shells out to `nu`, so nushell has to be
        # on PATH rather than only reachable by store path.
        nativeBuildInputs = cargoArgs.nativeBuildInputs ++ [
          pkgs.nodejs
          pkgs.npmHooks.npmConfigHook
          pkgs.nushell
        ];
        inherit wasm-bindgen-cli;

        preBuild = ''
          nu scripts/update-wasm.nu
          nu scripts/build-wasm.nu
        '';

        buildPhaseCargoCommand = ''
          nu scripts/build-web.nu --release
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
            UNAVI_SYNC_TARGETS = channelSyncTargets channel;
          }
        );

      mkWebPackage =
        channel:
        pkgs.crane.buildTrunkPackage (
          webArgs
          // {
            pname = "${pname}-web-${channel}";
            UNAVI_SYNC_TARGETS = channelSyncTargets channel;
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
      # checks = {
      #   "${pname}-doc" = pkgs.crane.cargoDoc (cargoArgs // { inherit cargoArtifacts; });
      #   "${pname}-nextest" = pkgs.crane.cargoNextest (
      #     cargoArgs
      #     // {
      #       inherit cargoArtifacts;
      #       cargoExtraArgs = cargoArgs.cargoExtraArgs + " --no-tests pass";
      #       preBuild = "${pkgs.nushell}/bin/nu scripts/build-wasm.nu";
      #     }
      #   );
      # };

      packages = {
        "${pname}" = pkgs.crane.buildPackage nativeArgs;
        "${pname}-web" = pkgs.crane.buildTrunkPackage (webArgs // { pname = "${pname}-web"; });
      }
      // channelPackages;
    };
}
