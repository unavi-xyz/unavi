{
  domainSocial,
  domainWorld,
  domainWeb,
  unavi-server,
  unavi-web,
  ...
}:
let
  portSocial = "3000";
  portWorld = "3001";

  urlSocial = "http://localhost:${portSocial}";
  urlWorld = "http://localhost:${portWorld}";

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
    defaults.email = "admin@${domainSocial}";
  };

  services.nginx = {
    enable = true;
    recommendedGzipSettings = true;
    recommendedOptimisation = true;
    recommendedProxySettings = true;
    recommendedTlsSettings = true;
    virtualHosts = {
      ${domainSocial} = {
        enableACME = true;
        forceSSL = true;
        http2 = true;
        locations = {
          "/" = {
            proxyPass = urlSocial;
            extraConfig = extraServerConfig;
          };
        };
      };
      ${domainWorld} = {
        enableACME = true;
        forceSSL = true;
        http2 = true;
        locations = {
          "/" = {
            proxyPass = urlWorld;
            extraConfig = extraServerConfig;
          };
        };
      };
      ${domainWeb} = {
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
    unavi-social-server = {
      description = "unavi-social-server";
      after = [ "network.target" ];
      wantedBy = [ "multi-user.target" ];
      serviceConfig = {
        ExecStart = "${unavi-server}/bin/unavi-server --debug social -p ${portSocial} --d ${domainSocial}";
        Restart = "always";
      };
    };
    unavi-world-host = {
      description = "unavi-world-host";
      after = [ "network.target" ];
      wantedBy = [ "multi-user.target" ];
      serviceConfig = {
        ExecStart = "${unavi-server}/bin/unavi-server --debug world -p ${portWorld} --d ${domainWorld} --dwn-url ${urlSocial}";
        Restart = "always";
      };
    };
  };
}
