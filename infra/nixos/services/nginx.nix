{
  config,
  lib,
  deployInfo,
  ...
}:
let
  inherit (config.services.unavi-server) channel;
  inherit (deployInfo.${channel}) services;

  virtualHosts = lib.mapAttrs (_serviceName: serviceConfig: {
    enableACME = true;
    forceSSL = true;
    locations."/" = {
      proxyPass = "http://127.0.0.1:${toString serviceConfig.port}";
      proxyWebsockets = true;
    };
  }) (lib.mapAttrs' (_name: config: lib.nameValuePair config.domain config) services);
in
{
  services.nginx = {
    enable = true;
    recommendedProxySettings = true;
    recommendedTlsSettings = true;
    recommendedOptimisation = true;
    recommendedGzipSettings = true;

    inherit virtualHosts;
  };

  security.acme = {
    acceptTerms = true;
    defaults.email = "admin@unavi.xyz";
  };

  networking.firewall.allowedTCPPorts = [
    80
    443
  ];
}
