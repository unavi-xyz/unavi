{ pkgs, inputs, ... }:
{
  imports = [
    ./common.nix
    ./hardware.nix
    ./services/nginx.nix
    ./services/unavi-server.nix
  ];

  networking.hostName = "unavi-beta";

  services.unavi-server = {
    channel = "beta";
    enable = true;
    package = inputs.self.packages.${pkgs.system}.unavi-server;
  };

  sops = {
    age.keyFile = "/var/lib/sops-nix/key.txt";
    defaultSopsFile = ../secrets/secrets.yaml;
  };
}
