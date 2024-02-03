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
in rec {
  linux-app = pkgs.callPackage ./unavi-app.nix {
    inherit craneLib crossTarget;
    commonArgs = linuxCommonArgs;
  };
  linux-server = pkgs.callPackage ./unavi-server.nix {
    inherit craneLib crossTarget;
    commonArgs = linuxCommonArgs;
  };

  linux = pkgs.symlinkJoin {
    name = "linux";
    paths = [ linux-app linux-server ];
  };
}
