{
  lib,
  deployInfo,
  pkgs,
  self,
  ...
}:
let
  inherit (deployInfo.stable) services;

  proxyServices = lib.filterAttrs (_: c: c ? port) services;
  staticServices = lib.filterAttrs (_: c: c ? static && c.static) services;

  webAppPkg = self.packages.${pkgs.stdenv.hostPlatform.system}.unavi-app-web;

  proxyHosts = lib.mapAttrs (_: cfg: {
    enableACME = true;
    forceSSL = true;
    locations."/" = {
      proxyPass = "http://127.0.0.1:${toString cfg.port}";
      proxyWebsockets = true;
    };
  }) (lib.mapAttrs' (_: c: lib.nameValuePair c.domain c) proxyServices);

  # A location's `add_header` replaces every header inherited from the server
  # block. The NixOS nginx module adds none at server level, so each location
  # sets its own Cache-Control and loses nothing.
  cacheControl = value: ''
    add_header Cache-Control "${value}" always;
  '';

  noCache = cacheControl "no-cache";
  immutable = cacheControl "public, max-age=31536000, immutable";
  shortLived = cacheControl "public, max-age=300, must-revalidate";

  staticHosts = lib.mapAttrs (_: _cfg: {
    enableACME = true;
    forceSSL = true;
    root = webAppPkg;

    extraConfig = ''
      brotli_static on;
      gzip_static on;
    '';

    locations = {
      "/" = {
        tryFiles = "$uri $uri/ =404";
      };

      # Trunk content-hashes these names, so a changed file gets a new URL.
      "~ ^/(webgl|webgpu)/[^/]+-[0-9a-f]{8,16}(_bg)?\\.(js|mjs|wasm|css|ico)$" = {
        tryFiles = "$uri =404";
        extraConfig = immutable;
      };

      "~ ^/(webgl|webgpu)/assets/" = {
        tryFiles = "$uri =404";
        extraConfig = shortLived;
      };

      "~ /index\\.html$" = {
        tryFiles = "$uri =404";
        extraConfig = noCache;
      };

      "= /sizes.json" = {
        extraConfig = noCache;
      };
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
    recommendedBrotliSettings = true;

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
