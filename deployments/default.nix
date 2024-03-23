{ deploy-rs, localSystem, nixpkgs, nixpkgs-stable, self, ... }:
let
  pkgs = import nixpkgs {
    inherit localSystem;
    config.allowUnfree = true;
  };

  tfOutput = builtins.fromJSON (builtins.readFile ./terraform-output.json);

  mkServer = resource: {
    name = resource.value.name;
    value = {
      hostname = resource.value.ip;
      sshUser = "root";

      profiles.system = {
        path = deploy-rs.lib.${localSystem}.activate.nixos
          self.nixosConfigurations.unavi-server;
      };
    };
  };

  outputValues = builtins.attrValues tfOutput;
  nodeList = builtins.map mkServer outputValues;
in {
  checks = deploy-rs.lib.${localSystem}.deployChecks self.deploy;

  deploy.nodes = builtins.listToAttrs nodeList;

  nixosConfigurations = {
    unavi-server = nixpkgs-stable.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [ ./configurations/unavi-server.nix ];
    };
  };
}
