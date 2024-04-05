{ pkgs, unavi-server, ... }: {
  imports = [ ./common.nix ];

  networking.firewall.allowedTCPPorts = [ 443 ];

  systemd.services.unavi_server = {
    description = "UNAVI Server";
    after = [ "network.target" ];
    wantedBy = [ "multi-user.target" ];
    serviceConfig = {
      ExecStart =
        "${unavi-server}/bin/unavi-server --enable-did-host --enable-dwn --enable-world-host --enable-world-registry";
      Restart = "always";
    };
  };
}
