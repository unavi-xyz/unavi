{ inputs, ... }: {
  perSystem =
    {
      mkAppImage,
      pkgs,
      lib,
      system,
      ...
    }:
    let
      pname = "unavi-client";

      wasmBindgenVersion =
        (fromTOML (builtins.readFile ../../Cargo.lock)).package
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
          ../../assets
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
          pkgs.lib.optionals pkgs.stdenv.hostPlatform.isLinux (
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
          ++ [ inputs.wit-deps.packages.${system}.wit-deps ];

        linkedInputs = pkgs.lib.optionals pkgs.stdenv.hostPlatform.isLinux (
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

      wasmCC = {
        CC_wasm32_unknown_unknown = "${pkgs.llvmPackages_21.clang-unwrapped}/bin/clang";
        AR_wasm32_unknown_unknown = "${pkgs.llvmPackages_21.llvm}/bin/llvm-ar";
        CFLAGS_wasm32_unknown_unknown = "--target=wasm32 -O3 -isystem ${pkgs.llvmPackages_21.libclang.lib}/lib/clang/21/include";
      };

      cargoArtifacts = pkgs.crane.buildDepsOnly cargoArgs;
      cargoArtifactsWeb = pkgs.crane.buildDepsOnly (
        cargoArgs
        // wasmCC
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
          pkgs.patchelf
        ];

        preBuild = ''
          nu scripts/update-wasm.nu --locked
          nu scripts/build-wasm.nu
        '';

        postInstall = ''
          mkdir -p $out/bin/assets
          cp -r crates/${pname}/assets/* $out/bin/assets/
          rm -f $out/bin/assets/hsd/example_*.hsdz
          cp LICENSE $out
          patchelf --set-rpath "${lib.makeLibraryPath cargoArgs.linkedInputs}" $out/bin/${pname}
          wrapProgram $out/bin/${pname} \
            --prefix LD_LIBRARY_PATH : "${lib.makeLibraryPath cargoArgs.linkedInputs}"
        '';
      };

      webArgs =
        cargoArgs
        // wasmCC
        // {
          cargoArtifacts = cargoArtifactsWeb;

          npmRoot = "crates/unavi-script";
          inherit npmDeps;

          nativeBuildInputs = cargoArgs.nativeBuildInputs ++ [
            pkgs.nodejs
            pkgs.npmHooks.npmConfigHook
            pkgs.nushell
          ];
          inherit wasm-bindgen-cli;

          preBuild = ''
            nu scripts/update-wasm.nu --locked
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
        };

      packageDrv = pkgs.crane.buildPackage nativeArgs;
    in
    {
      packages = {
        "${pname}" = packageDrv;
        "${pname}-web" = pkgs.crane.buildTrunkPackage (webArgs // { pname = "${pname}-web"; });
      }
      // lib.optionalAttrs pkgs.stdenv.hostPlatform.isLinux {
        "${pname}-appimage" = mkAppImage { program = "${packageDrv}/bin/${pname}"; };
      };
    };
}
