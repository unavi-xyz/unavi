{ pkgs, inputs, ... }:
{
  imports = [
    ./common.nix
    ./hardware.nix
    ./services/unavi-server.nix
    ./services/nginx.nix
  ];

  networking.hostName = "unavi-server";

  services.unavi-server = {
    enable = true;
    package = inputs.self.packages.${pkgs.system}.unavi-server;
    channel = "beta";
  };

  sops.defaultSopsFile = ../secrets/secrets.yaml;
  sops.age.keyFile = "/var/lib/sops-nix/key.txt";
}
