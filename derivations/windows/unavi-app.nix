{ stdenv, qemu, craneLib, commonArgs, crossTarget, pkgs }:
craneLib.buildPackage commonArgs // {
  pname = "windows-app";
  cargoExtraArgs = "-p unavi-app --target ${crossTarget}";

  depsBuildBuild = [ qemu ];

  CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER = "${stdenv.cc.targetPrefix}link";
  CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUNNER = "qemu-x86_64";

  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [ pkgs.vulkan-loader ];
}
