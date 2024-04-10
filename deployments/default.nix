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

  mkDidWeb =
    domain:
    pkgs.lib.concatStrings [
      "did:web:"
      domain
    ];

  mkAppInput = resource: { registry = mkDidWeb resource.value.domain; };

  mkAppPackage = resource: self.crates.${localSystem}.mkUnaviApp (mkAppInput resource);
  mkServerPackage = resource: self.crates.${localSystem}.mkUnaviServer (mkAppInput resource);

  mkPackages =
    { resource, subdir }:
    [
      {
        name = pkgs.lib.concatStrings [
          "unavi-app-"
          subdir
        ];
        value = mkAppPackage resource;
      }
      {
        name = pkgs.lib.concatStrings [
          "unavi-server-"
          subdir
        ];
        value = mkServerPackage resource;
      }
    ];

  mkServerNode = resource: {
    name = resource.value.name;
    value = {
      hostname = resource.value.ip;
      sshUser = "root";

      profiles.system =
        let
          config = nixpkgs-stable.lib.nixosSystem rec {
            system = "x86_64-linux";
            modules = [ ./configurations/unavi-server.nix ];
            specialArgs = {
              domain = resource.value.domain;
              unavi-server = self.crates.${system}.mkUnaviServer (mkAppInput resource);
            };
          };
        in
        {
          path = deploy-rs.lib.${localSystem}.activate.nixos config;
        };
    };
  };

  outputDir = ./output;
  subdirs = builtins.attrNames (builtins.readDir outputDir);

  loadTfOutput =
    subdir: builtins.fromJSON (builtins.readFile "${outputDir}/${subdir}/terraform-output.json");

  nodeList = map (subdir: map mkServerNode (builtins.attrValues (loadTfOutput subdir))) subdirs;

  packageList = map (
    subdir:
    map mkPackages (
      map (resource: { inherit resource subdir; }) (builtins.attrValues (loadTfOutput subdir))
    )
  ) subdirs;
in
{
  checks = deploy-rs.lib.${localSystem}.deployChecks self.deploy;
  deploy.nodes = builtins.listToAttrs (builtins.concatLists nodeList);
  packages = builtins.listToAttrs (builtins.concatLists (builtins.concatLists packageList));
}
