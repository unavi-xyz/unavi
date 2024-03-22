{ pkgs ? import <nixpkgs> { } }:
let
  unavi_server_zip = pkgs.fetchzip {
    url = "placeholder-url";
    sha256 = "placeholder-sha";
  };

  unavi_server = pkgs.stdenv.mkDerivation {
    name = "unavi-server";
    buildInputs = [ pkgs.unzip ];

    src = unavi_server_zip;

    unpackPhase = "unzip $src";
    installPhase = ''
      mkdir -p $out/bin
      cp -r * $out/bin/
    '';
  };
in {
  environment.systemPackages = [ unavi_server ];

  systemd.services.unavi_server = {
    description = "UNAVI Server";
    after = [ "network.target" ];
    wantedBy = [ "multi-user.target" ];
    serviceConfig = {
      ExecStart = "${pkgs.unavi_server}/bin/unavi_server";
      Restart = "always";
    };
  };
}
