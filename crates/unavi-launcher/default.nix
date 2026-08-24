_: {
  perSystem =
    { pkgs, lib, ... }:
    let
      pname = "unavi-launcher";

      libCheck = pkgs.writeShellScript "unavi-launcher-libcheck" ''
        missing=$(${pkgs.glibc.bin}/bin/ldd "$1" 2>&1 | ${pkgs.gnugrep}/bin/grep "not found") || true
        if [ -n "$missing" ]; then
          echo "unavi-launcher: missing system libraries, this host needs WebKitGTK 4.1 + GTK3 installed:" >&2
          echo "$missing" >&2
          exit 1
        fi
      '';

      # webkitgtk/gtk3/glib/cairo/... resolve against the host's own install
      # (bundling webkitgtk's own closure balloons this from ~50MB to ~850MB),
      # but these are unavi-launcher's own small direct deps, cheap to bundle
      # and not something every host is guaranteed to already have.
      rpath = lib.makeLibraryPath [
        pkgs.openssl
        pkgs.xdotool
        pkgs.xz
      ];

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
            patchelf --set-rpath "${rpath}" $out/bin/${pname}
            wrapProgram $out/bin/${pname} \
              --set WEBKIT_DISABLE_DMABUF_RENDERER 1 \
              --run "${libCheck} $out/bin/.${pname}-wrapped"
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
