{
  deploy-rs,
  localSystem,
  nixpkgs,
  nixpkgs-stable,
  self,
  ...
}:
let
  pkgs = import nixpkgs {
    inherit localSystem;
    config.allowUnfree = true;
  };

  mkServer = resource: {
    name = resource.value.name;
    value = {
      hostname = resource.value.ip;
      sshUser = "root";

      profiles.system =
        let
          config = nixpkgs-stable.lib.nixosSystem {
            system = "x86_64-linux";
            modules = [ ./configurations/unavi-server.nix ];
            specialArgs = {
              domain = resource.value.domain;
              unavi-server = self.packages.x86_64-linux.unavi-server;
            };
          };
        in
        {
          path = deploy-rs.lib.${localSystem}.activate.nixos config;
        };
    };
  };

  terraformDir = ./terraform;

  subdirs = builtins.attrNames (builtins.readDir terraformDir);

  loadTfOutput =
    subdir: builtins.fromJSON (builtins.readFile "${terraformDir}/${subdir}/terraform-output.json");

  nodeList = map (subdir: map mkServer (builtins.attrValues (loadTfOutput subdir))) subdirs;
in
{
  checks = deploy-rs.lib.${localSystem}.deployChecks self.deploy;
  deploy.nodes = builtins.listToAttrs (builtins.concatLists nodeList);
}
