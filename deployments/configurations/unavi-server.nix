{
  domain-server,
  domain-web,
  pkgs,
  unavi-server,
  unavi-web,
  ...
}:
{
  imports = [ ./common.nix ];

  networking.firewall = {
    enable = true;
    allowedTCPPorts = [
      80
      443
    ];
  };

  security.acme = {
    acceptTerms = true;
    defaults.email = "admin@${domain-server}";
  };

  services.nginx = {
    enable = true;
    recommendedGzipSettings = true;
    recommendedOptimisation = true;
    recommendedProxySettings = true;
    recommendedTlsSettings = true;
    virtualHosts = {
      ${domain-server} = {
        enableACME = true;
        forceSSL = true;
        http2 = true;
        locations = {
          "/" = {
            proxyPass = "http://localhost:3000";
          };
        };
      };
      ${domain-web} = {
        enableACME = true;
        forceSSL = true;
        http2 = true;
        root = unavi-web;
        locations = {
          "/" = {
            index = "index.html";
          };
        };
      };
    };
  };

  systemd.services.unavi_server = {
    description = "UNAVI Server";
    after = [ "network.target" ];
    wantedBy = [ "multi-user.target" ];
    serviceConfig = {
      ExecStart = "${unavi-server}/bin/unavi-server --debug --domain ${domain-server} --enable-did-host --enable-dwn --enable-world-host --enable-world-registry";
      Restart = "always";
    };
  };
}
