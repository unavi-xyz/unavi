{ pkgs ? import <nixpkgs> { }, ... }:
let
  unavi_server = pkgs.stdenv.mkDerivation {
    name = "unavi-server";
    buildInputs = [ pkgs.unzip ];

    # Our ZIP file needs to be manually uploaded to the server.
    # This is handled by the CI/CD pipeline.
    unpackPhase = "unzip /var/lib/unavi-server/unavi-server.zip";

    installPhase = ''
      mkdir -p $out/bin
      cp -r * $out/bin/
    '';
  };
in {
  systemd.services.unavi_server = {
    description = "UNAVI Server";
    after = [ "network.target" ];
    wantedBy = [ "multi-user.target" ];
    serviceConfig = {
      ExecStart = "${unavi_server}/bin/unavi-server";
      Restart = "always";
    };
  };
}
