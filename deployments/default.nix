{ deploy-rs, nixpkgs-stable, self, ... }:
let
  pkgs = import nixpkgs-stable {
    localSystem = "x86_64-linux";
    config.allowUnfree = true;
  };

  resourcesByType = (import ./parse.nix { inherit pkgs; }).resourcesByType;

  droplets = resourcesByType "digitalocean_droplet";
  servers = builtins.filter (d: d.name == "unavi-server") droplets;

  mkServer = resource: {
    hostname = resource.values.ipv4_address;
    sshUser = "root";

    profiles.system = {
      path = deploy-rs.lib.x86_64-linux.activate.nixos
        self.nixosConfigurations.unavi-server;
    };
  };
in {
  deploy.nodes = builtins.listToAttrs (map (r: {
    name = r.values.name;
    value = mkServer r;
  }) servers);

  nixosConfigurations = {
    unavi-server = pkgs.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [ ./nix/unavi-server.nix ];
    };
  };
}
