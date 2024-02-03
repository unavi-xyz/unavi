{ nixpkgs, rust-overlay, crane, system, commonArgs }:
let
  crossSystem = "x86_64-pc-windows-msvc";
  crossTarget = "x86_64-pc-windows-msvc";

  pkgs = import nixpkgs {
    inherit crossSystem system;
    overlays = [ (import rust-overlay) ];
  };

  rustToolchain = pkgs.pkgsBuildHost.rust-bin.stable.latest.default.override {
    targets = [ crossTarget ];
  };

  craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

  cargoArtifacts =
    craneLib.buildDepsOnly (commonArgs // { pname = "windows-deps"; });

  windowsCommonArgs = commonArgs // { inherit cargoArtifacts; };
in rec {
  windows-app = pkgs.callPackage ./unavi-app.nix {
    inherit craneLib crossTarget;
    commonArgs = windowsCommonArgs;
  };
  windows-server = pkgs.callPackage ./unavi-server.nix {
    inherit craneLib crossTarget;
    commonArgs = windowsCommonArgs;
  };

  windows = pkgs.symlinkJoin {
    name = "windows";
    paths = [ windows-app windows-server ];
  };
}
