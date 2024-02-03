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

  unavi-app = pkgs.callPackage ./unavi-app.nix {
    inherit craneLib crossTarget;
    commonArgs = windowsCommonArgs;
  };
  unavi-server = pkgs.callPackage ./unavi-server.nix {
    inherit craneLib crossTarget;
    commonArgs = windowsCommonArgs;
  };
in {
  windows-app = unavi-app;
  windows-server = unavi-server;

  windows = pkgs.symlinkJoin {
    name = "windows";
    paths = [ unavi-app unavi-server ];
  };
}
