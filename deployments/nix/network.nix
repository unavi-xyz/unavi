let
  resourcesByType = (import ./parse.nix { }).resourcesByType;

  droplets = resourcesByType "digitalocean_droplet";
  servers = builtins.filter (d: d.name == "server") droplets;

  mkServer = resource:
    { modulesPath, lib, name, ... }: {
      imports =
        lib.optional (builtins.pathExists ./do-userdata.nix) ./do-userdata.nix
        ++ [
          (modulesPath + "/virtualisation/digital-ocean-config.nix")
          ./common.nix
        ];

      deployment.targetHost = resource.values.ipv4_address;
      deployment.targetUser = "root";

      networking.hostName = resource.values.name;

      system.stateVersion = "23.11";
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
