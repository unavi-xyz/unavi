_: {
  perSystem =
    { pkgs, lib, ... }:
    let
      pname = "unavi-launcher";

      # dx must match the `dioxus` crate version in Cargo.lock or codegen
      # diverges from the compiled app.
      dioxusCli =
        let
          expected =
            (fromTOML (builtins.readFile ../../Cargo.lock)).package
            |> lib.findFirst (p: p.name == "dioxus") (throw "Cargo.lock has no dioxus")
            |> (p: p.version);
          actual = pkgs.dioxus-cli.version;
        in
        if actual == expected then
          pkgs.dioxus-cli
        else
          throw "nixpkgs dioxus-cli ${actual} != Cargo.lock dioxus ${expected}";

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
            dioxusCli
            lld
            makeWrapper
            patchelf
            pkg-config
            python3
          ]
        );

        # Compile-time only. postInstall strips RPATH so the binary
        # resolves these against whatever the host system has installed,
        # instead of shipping WebKitGTK's own multi-GB runtime closure
        # in the release artifact.
        buildInputs = with pkgs; [
          at-spi2-atk
          atkmm
          bzip2
          cairo
          gdk-pixbuf
          glib
          gtk3
          harfbuzz
          libiconv
          librsvg
          libsoup_3
          openssl
          pango
          webkitgtk_4_1
          xdotool
          xz
        ];
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
            mkdir -p $out/bin
            cp -r target/dx/${pname}/release/*/app/* $out/bin
          '';

          postInstall = ''
            cp LICENSE $out
            patchelf --remove-rpath $out/bin/${pname}
            wrapProgram $out/bin/${pname} \
              --set WEBKIT_DISABLE_DMABUF_RENDERER 1
          '';
        }
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
      #     }
      #   );
      # };

      packages = {
        "${pname}" = packageDrv;
      };
    };
}
