{ }:
let
  crossSystem = "x86_64-linux";
  crossTarget = "x86_64-unknown-linux-gnu";

  pkgs = import nixpkgs {
    inherit crossSystem localSystem;
    overlays = [ (import rust-overlay) ];
  };

  rustToolchain = pkgs.pkgsBuildHost.rust-bin.stable.latest.default.override {
    targets = [ crossTarget ];
  };

  craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;
in {
  unavi-app = pkgs.callPackage ./unavi-app.nix { inherit craneLib; };
  unavi-server = pkgs.callPackage ./unavi-server.nix { inherit craneLib; };
}
