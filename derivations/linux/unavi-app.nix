{ craneLib, stdenv, qemu }:
craneLib.buildPackage {
  src = craneLib.cleanCargoSource (craneLib.path ../crates/unavi-app);
  cargoVendorDir = craneLib.vendorCargoDeps { src = ../.; };
  strictDeps = true;

  buildInputs = with pkgs; [
    alsa-lib.dev
    libxkbcommon
    udev
    vulkan-loader
    wayland
    xorg.libX11
    xorg.libXcursor
    xorg.libXi
    xorg.libXrandr
  ];

  nativeBuildInputs = with pkgs; [ cargo-auditable pkg-config protobuf ];

  depsBuildBuild = [ qemu ];

  CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER = "${stdenv.cc.targetPrefix}cc";
  CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER = "qemu-x86_64";

  cargoExtraArgs = "--target ${crossTarget}";

  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [ pkgs.vulkan-loader ];
}
