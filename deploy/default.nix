{
  flake-utils,
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

  mkAppInput = resource: { worldHostDid = mkDidWeb resource.value.domain_world; };

  mkAppPackage = resource: self.crates.${localSystem}.mkUnaviApp (mkAppInput resource);
  mkWebPackage = resource: self.crates.${localSystem}.mkUnaviWeb (mkAppInput resource);

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
          "unavi-web-"
          subdir
        ];
        value = mkWebPackage resource;
      }
      {
        name = pkgs.lib.concatStrings [
          "unavi-server-"
          subdir
        ];
        value = self.packages.${localSystem}.unavi-server;
      }
    ];

  mkNode =
    { resource, subdir }:
    {
      name = subdir;
      value = {
        hostname = resource.value.ip;
        sshUser = "root";

        profiles.system =
          let
            config = nixpkgs-stable.lib.nixosSystem rec {
              system = "x86_64-linux";
              modules = [ ./config/unavi-server.nix ];
              specialArgs = {
                domainDwn = resource.value.domain_dwn;
                domainWeb = resource.value.domain_web;
                domainWorld = resource.value.domain_world;
                unavi-server = self.packages.${system}."unavi-server-${subdir}";
                unavi-web = self.packages.${system}."unavi-web-${subdir}";
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

  forEachResource =
    fn:
    map (
      subdir: map (resource: fn { inherit resource subdir; }) (builtins.attrValues (loadTfOutput subdir))
    ) subdirs;

  nodeList = forEachResource mkNode;
  packageList = forEachResource mkPackages;
in
{
  apps = {
    deploy = flake-utils.lib.mkApp {
      drv = pkgs.writeShellApplication {
        name = "deploy";
        runtimeInputs = [ pkgs.deploy-rs ];
        text = ''
          deploy ".?submodules=1#$TF_WORKSPACE" --skip-checks
        '';
      };
    };
  };
  checks = deploy-rs.lib.${localSystem}.deployChecks self.deploy;
  deploy.nodes = builtins.listToAttrs (builtins.concatLists nodeList);
  packages = builtins.listToAttrs (builtins.concatLists (builtins.concatLists packageList));
}
