{
  domain-social,
  domain-world,
  domain-web,
  unavi-server,
  unavi-web,
  ...
}:
let
  portSocial = "3000";
  portWorld = "3001";

  extraServerConfig = ''
    if ($request_method = 'OPTIONS') {
        add_header 'Access-Control-Allow-Headers' '*';
        add_header 'Access-Control-Allow-Methods' 'GET, POST, OPTIONS';
        add_header 'Access-Control-Allow-Origin' '*';
        add_header 'Access-Control-Max-Age' 1728000;
        add_header 'Content-Length' 0;
        add_header 'Content-Type' 'text/plain; charset=utf-8';
        return 204;
    }

    add_header 'Access-Control-Allow-Methods' 'GET, POST, OPTIONS';
    add_header 'Access-Control-Allow-Origin' '*';
  '';
in
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
    defaults.email = "admin@${domain-social}";
  };

  services.nginx = {
    enable = true;
    recommendedGzipSettings = true;
    recommendedOptimisation = true;
    recommendedProxySettings = true;
    recommendedTlsSettings = true;
    virtualHosts = {
      ${domain-social} = {
        enableACME = true;
        forceSSL = true;
        http2 = true;
        locations = {
          "/" = {
            proxyPass = "http://localhost:${portSocial}";
            extraConfig = extraServerConfig;
          };
        };
      };
      ${domain-world} = {
        enableACME = true;
        forceSSL = true;
        http2 = true;
        locations = {
          "/" = {
            proxyPass = "http://localhost:${portWorld}";
            extraConfig = extraServerConfig;
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

  systemd.services = {
    unavi-server-social = {
      description = "unavi-server-social";
      after = [ "network.target" ];
      wantedBy = [ "multi-user.target" ];
      serviceConfig = {
        ExecStart = "${unavi-server}/bin/unavi-server --debug social -p ${portSocial} --domain ${domain-social} --enable-world-host";
        Restart = "always";
      };
    };
    unavi-server-world = {
      description = "unavi-server-world";
      after = [ "network.target" ];
      wantedBy = [ "multi-user.target" ];
      serviceConfig = {
        ExecStart = "${unavi-server}/bin/unavi-server --debug world -p ${portWorld}";
        Restart = "always";
      };
    };
  };
}
