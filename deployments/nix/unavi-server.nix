{ pkgs ? import <nixpkgs> { }, ... }:
let
  unavi_server = pkgs.stdenv.mkDerivation {
    name = "unavi-server";
    buildInputs = [ pkgs.unzip ];

    src = ../../x86_64-linux.unavi-server.zip;

    unpackPhase = "unzip $src";
    installPhase = ''
      mkdir -p $out/bin
      cp -r * $out/bin/
    '';
  };
in {
  # systemd.services.unavi_server = {
  #   description = "UNAVI Server";
  #   after = [ "network.target" ];
  #   wantedBy = [ "multi-user.target" ];
  #   serviceConfig = {
  #     ExecStart = "${unavi_server}/bin/unavi-server";
  #     Restart = "always";
  #   };
  # };
}
