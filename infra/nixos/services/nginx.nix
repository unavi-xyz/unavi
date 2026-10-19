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
  # sets its own headers and loses nothing.
  immutable = ''
    add_header Cache-Control "public, max-age=31536000, immutable" always;
  '';

  # nixpkgs' nginx tags a store file with its store path's hash and size and
  # sends no Last-Modified, so a revalidation is a 304 until a deploy changes
  # the package, then a full fetch.
  revalidate = ''
    add_header Cache-Control "no-cache" always;
  '';

  staticHosts = lib.mapAttrs (_: _cfg: {
    enableACME = true;
    forceSSL = true;
    root = webAppPkg;

    extraConfig = ''
      brotli_static on;
      gzip_static on;
    '';

    locations = {
      # Everything without a content hash in its name: the loader and each
      # bundle's index.html, sizes.json, wasm-bindgen snippets (named for the
      # crate) and copied assets. These ship together, so all revalidate
      # rather than letting a stale asset meet a newer wasm.
      "/" = {
        tryFiles = "$uri $uri/ =404";
        extraConfig = revalidate;
      };

      # Trunk content-hashes these names, so a changed file gets a new URL.
      # Quoted, since nginx ends an unquoted word at `{`.
      "~ \"^/(webgl|webgpu)/[^/]+-[0-9a-f]{8,16}(_bg)?\\.(js|mjs|wasm|css|ico)$\"" = {
        tryFiles = "$uri =404";
        extraConfig = immutable;
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
