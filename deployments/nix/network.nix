let
  resourcesByType = (import ./parse.nix { }).resourcesByType;

  droplets = resourcesByType "digitalocean_droplet";
  servers = builtins.filter (d: d.name == "unavi-server") droplets;

  mkServer = resource:
    { lib, modulesPath, name, ... }: {
      imports = [ ./common.nix ./unavi-server.nix ];
      deployment.targetHost = resource.values.ipv4_address;
      networking.hostName = resource.values.name;
    };
in {
  network = {
    pkgs = import (builtins.fetchGit {
      url = "https://github.com/NixOS/nixpkgs";
      ref = "nixos-23.11";
      rev = "f091af045dff8347d66d186a62d42aceff159456";
    }) { };
  };

} // builtins.listToAttrs (map (r: {
  name = r.values.name;
  value = mkServer r;
}) servers)
