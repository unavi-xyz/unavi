{ pkgs, inputs, ... }:
{
  imports = [
    ./common.nix
    ./hardware.nix
    ./services/nginx.nix
    ./services/unavi-server.nix
    ./services/wds.nix
  ];

  networking.hostName = "unavi-stable";

  services = {
    unavi-server = {
      channel = "stable";
      enable = true;
      package = inputs.self.packages.${pkgs.system}.unavi-server;
    };
    wds = {
      enable = true;
      package = inputs.self.packages.${pkgs.system}.wds;
    };
  };

  sops = {
    age.keyFile = "/var/lib/sops-nix/key.txt";
    defaultSopsFile = ../secrets/secrets.yaml;
  };
}
