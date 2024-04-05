{ domain, pkgs, unavi-server, ... }: {
  imports = [ ./common.nix ];

  networking.firewall = {
    enable = true;
    allowedTCPPorts = [ 443 ];
  };

  security.acme = {
    acceptTerms = true;
    defaults.email = "admin@${domain}";
    certs = {
      ${domain} = {
        webroot = "/var/www/${domain}";
        postRun = "systemctl reload nginx.service";
      };
    };
  };

  services.nginx = {
    enable = true;
    recommendedGzipSettings = true;
    recommendedOptimisation = true;
    recommendedProxySettings = true;
    recommendedTlsSettings = true;
    virtualHosts.${domain} = {
      enableACME = true;
      forceSSL = true;
      http2 = true;
      locations."/" = { proxyPass = "http://localhost:3000"; };
    };
  };

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
