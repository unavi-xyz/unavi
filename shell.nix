{ pkgs ? import <nixpkgs> { } }:

pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    python310
    python310Packages.pip
    gnumake
    cmake
    gcc
  ];

  shellHook = ''
    set -a
    source ${toString ./apps/client/.env}
    set +a
  '';
}
