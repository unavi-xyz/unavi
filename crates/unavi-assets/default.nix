{
  perSystem =
    { pkgs, lib, ... }:
    let
      # The bytes every unavi-node must host, pinned so a source that moves
      # fails the build. Names are the manifest's `rel_path`s, which an operator
      # reads in files.json; the node serves by hash.
      sources = {
        "model/default.vrm" = pkgs.fetchurl {
          url = "https://unavi.nyc3.digitaloceanspaces.com/assets/model/default.vrm";
          hash = "sha256-UR/CQsv1sgKpeWHe7euzyuso3oHBzSPrWuMHvp88138=";
        };
        "model/animations.glb" = pkgs.fetchurl {
          url = "https://unavi.nyc3.digitaloceanspaces.com/assets/model/animations.glb";
          hash = "sha256-wP9wd3na1tzYbrEcjfC8r8m/lSjWKf5tX3fHB8qW45c=";
        };
        "font/noto-sans.ttf" = pkgs.fetchurl {
          # notofonts.github.io publishes no per-face release, so the ref is a
          # commit rather than a tag.
          url = "https://raw.githubusercontent.com/notofonts/notofonts.github.io/28b15b4b43b7bed62b5cf6e6b0b5ff5846270535/fonts/NotoSans/hinted/ttf/NotoSans-Regular.ttf";
          hash = "sha256-R4xVjqcWAzzWDANDj2KN+nVpTc9rX21QWi8F/StPOCM=";
        };
        "font/noto-sans-cjk.otf" = pkgs.fetchurl {
          url = "https://raw.githubusercontent.com/notofonts/noto-cjk/165c01b46ea533872e002e0785ff17e44f6d97d8/Sans/OTF/Japanese/NotoSansCJKjp-Regular.otf";
          hash = "sha256-aKP8mIALKiezcfL7eZkdrzYzvYkwnU/6ppRv1YfzdbU=";
        };
      };

      assets = pkgs.runCommandLocal "unavi-assets" { } (
        lib.concatStrings (
          lib.mapAttrsToList (rel: file: ''
            install -Dm444 ${file} $out/${rel}
          '') sources
        )
      );

      # Read from the crate rather than restated here: the manifest is what the
      # app requests, so a new asset cannot land there without a pin above.
      manifest =
        let
          readLine =
            state: line:
            let
              rel = builtins.match "[[:space:]]*rel_path: \"([^\"]+)\"," line;
              hash = builtins.match "[[:space:]]*hash:[[:space:]]*hex_hash\\(\"([0-9a-f]{64})\"\\)," line;
            in
            if rel != null then
              state // { pending = builtins.head rel; }
            else if hash != null && state.pending != null then
              state
              // {
                found = state.found // {
                  ${state.pending} = builtins.head hash;
                };
              }
            else
              state;
        in
        (builtins.foldl' readLine {
          pending = null;
          found = { };
        } (lib.splitString "\n" (builtins.readFile ./src/lib.rs))).found;

      expected = pkgs.writeText "unavi-assets.blake3" (
        lib.concatStrings (lib.mapAttrsToList (rel: hash: "${hash}  ${rel}\n") manifest)
      );
    in
    {
      packages.unavi-assets = assets;

      checks.unavi-assets-manifest =
        pkgs.runCommand "unavi-assets-manifest" { nativeBuildInputs = [ pkgs.b3sum ]; }
          ''
            cd ${assets}

            # Exactly the manifest's files, each hashing to its blake3.
            diff <(find . -type f -printf '%P\n' | sort) \
              <(cut -d' ' -f3- ${expected} | sort)
            b3sum --check ${expected}

            touch $out
          '';
    };
}
