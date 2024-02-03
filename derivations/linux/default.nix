{ nixpkgs, rust-overlay, crane, system, commonArgs }:
let
  crossSystem = "x86_64-linux";
  crossTarget = "x86_64-unknown-linux-gnu";

  pkgs = import nixpkgs {
    inherit crossSystem system;
    overlays = [ (import rust-overlay) ];
  };

  rustToolchain = pkgs.pkgsBuildHost.rust-bin.stable.latest.default.override {
    targets = [ crossTarget ];
  };

  craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

  cargoArtifacts =
    craneLib.buildDepsOnly (commonArgs // { pname = "linux-deps"; });

  linuxCommonArgs = commonArgs // { inherit cargoArtifacts; };

  unavi-app = pkgs.callPackage ./unavi-app.nix {
    inherit craneLib crossTarget;
    commonArgs = linuxCommonArgs;
  };
  unavi-server = pkgs.callPackage ./unavi-server.nix {
    inherit craneLib crossTarget;
    commonArgs = linuxCommonArgs;
  };
in {
  linux-app = unavi-app;
  linux-server = unavi-server;

  linux = pkgs.symlinkJoin {
    name = "linux";
    paths = [ unavi-app unavi-server ];
  };
}
