{
  config,
  lib,
  deployInfo,
  pkgs,
  self,
  ...
}:
let
  inherit (config.services.unavi-server) channel;
  inherit (deployInfo.${channel}) services;

  # Split services into proxy (with port) and static (with static flag).
  proxyServices = lib.filterAttrs (_: c: c ? port) services;
  staticServices = lib.filterAttrs (_: c: c ? static && c.static) services;

  webClientPkg = self.packages.${pkgs.system}."unavi-client-web-${channel}";

  proxyHosts = lib.mapAttrs (_: cfg: {
    enableACME = true;
    forceSSL = true;
    locations."/" = {
      proxyPass = "http://127.0.0.1:${toString cfg.port}";
      proxyWebsockets = true;
    };
  }) (lib.mapAttrs' (_: c: lib.nameValuePair c.domain c) proxyServices);

  staticHosts = lib.mapAttrs (_: _cfg: {
    enableACME = true;
    forceSSL = true;
    root = webClientPkg;
    locations."/" = {
      tryFiles = "$uri $uri/ /index.html";
    };
  }) (lib.mapAttrs' (_: c: lib.nameValuePair c.domain c) staticServices);
in
{
  services.nginx = {
    enable = true;
    recommendedProxySettings = true;
    recommendedTlsSettings = true;
    recommendedOptimisation = true;
    recommendedGzipSettings = true;

    virtualHosts = proxyHosts // staticHosts;
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
