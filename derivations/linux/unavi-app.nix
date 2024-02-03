{ stdenv, qemu, craneLib, commonArgs, crossTarget, pkgs }:
craneLib.buildPackage commonArgs // {
  pname = "linux-app";
  cargoExtraArgs = "-p unavi-app --target ${crossTarget}";

  src = lib.cleanSourceWith {
    src = ./.;
    filter = path: type:
      (lib.hasInfix "/assets/" path) || (lib.hasInfix "/wired-protocol/" path)
      || (craneLib.filterCargoSources path type);
  };

  depsBuildBuild = [ qemu ];

  CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER = "${stdenv.cc.targetPrefix}cc";
  CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER = "qemu-x86_64";

  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [ pkgs.vulkan-loader ];
}
