{ pkgs, inputs, ... }:
{
  imports = [
    ./common.nix
    ./hardware.nix
    ./services/nginx.nix
    ./services/unavi-server.nix
    ./services/wired-data-store.nix
  ];

  networking.hostName = "unavi-stable";

  services = {
    unavi-server = {
      channel = "stable";
      enable = true;
      package = inputs.self.packages.${pkgs.system}.unavi-server;
    };
    wired-data-store = {
      enable = true;
      package = inputs.self.packages.${pkgs.system}.wired-data-store;
    };
  };

  sops = {
    age.keyFile = "/var/lib/sops-nix/key.txt";
    defaultSopsFile = ../secrets/secrets.yaml;
  };
}
