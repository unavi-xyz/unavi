{ stdenv, qemu, craneLib, commonArgs, crossTarget, pkgs }:
craneLib.buildPackage commonArgs // {
  pname = "windows-server";
  cargoExtraArgs = "-p unavi-server --target ${crossTarget}";

  depsBuildBuild = [ qemu ];

  CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER = "${stdenv.cc.targetPrefix}link";
  CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUNNER = "qemu-x86_64";
}
