{ craneLib, stdenv, qemu }:
craneLib.buildPackage {
  src = craneLib.cleanCargoSource (craneLib.path ../crates/unavi-server);
  cargoVendorDir = craneLib.vendorCargoDeps { src = ../.; };
  strictDeps = true;

  nativeBuildInputs = with pkgs; [
    cargo-auditable
    clang
    cmake
    pkg-config
    protobuf
  ];

  depsBuildBuild = [ qemu ];

  CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER = "${stdenv.cc.targetPrefix}cc";
  CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER = "qemu-x86_64";

  cargoExtraArgs = "--target ${crossTarget}";
}
