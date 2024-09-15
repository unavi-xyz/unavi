{
  domainDwn,
  domainWeb,
  domainWorld,
  pkgs,
  unavi-server,
  # unavi-web,
  ...
}:
let
  portSocial = "3000";
  portWorld = "3001";

  urlSocial = "http://localhost:${portSocial}";
  urlWorld = "http://localhost:${portWorld}";
  urlDwn = "https://${domainDwn}";

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
    defaults.email = "admin@${domainWeb}";
  };

  services.nginx = {
    enable = true;
    recommendedGzipSettings = true;
    recommendedOptimisation = true;
    recommendedProxySettings = true;
    recommendedTlsSettings = true;
    virtualHosts = {
      ${domainDwn} = {
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
      # ${domainWeb} = {
      #   enableACME = true;
      #   forceSSL = true;
      #   http2 = true;
      #   root = unavi-web;
      #   locations = {
      #     "/" = {
      #       index = "index.html";
      #     };
      #   };
      # };
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
    };
  };

  systemd.services = {
    unavi-social = {
      description = "unavi-social";
      after = [ "network.target" ];
      wantedBy = [ "multi-user.target" ];
      serviceConfig = {
        Environment = [ "LD_LIBRARY_PATH=${pkgs.openssl.out}/lib" ];
        ExecStart = "${unavi-server}/unavi-server --debug social -p ${portSocial}";
        Restart = "always";
      };
    };
    unavi-world = {
      description = "unavi-world";
      after = [ "network.target" ];
      wantedBy = [ "multi-user.target" ];
      serviceConfig = {
        Environment = [ "LD_LIBRARY_PATH=${pkgs.openssl.out}/lib" ];
        ExecStart = "${unavi-server}/unavi-server --debug world -d ${domainWorld} -p ${portWorld} --remote-dwn ${urlDwn}";
        Restart = "always";
      };
    };
  };
}
