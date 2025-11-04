{ pkgs, inputs, ... }:
{
  imports = [
    ./common.nix
    ./hardware.nix
    ./services/nginx.nix
    ./services/unavi-server.nix
  ];

  networking.hostName = "unavi-stable";

  services.unavi-server = {
    channel = "stable";
    enable = true;
    package = inputs.self.packages.${pkgs.system}.unavi-server;
  };

  sops = {
    age.keyFile = "/var/lib/sops-nix/key.txt";
    defaultSopsFile = ../secrets/secrets.yaml;
  };
}
