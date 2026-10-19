{ pkgs, inputs, ... }: {
  imports = [
    ./common.nix
    ./hardware.nix
    ./services/nginx.nix
    ./services/unavi-node.nix
  ];

  networking.hostName = "unavi-stable";

  services = {
    unavi-node = {
      enable = true;
      package = inputs.self.packages.${pkgs.stdenv.hostPlatform.system}.unavi-node;
      assets = inputs.self.packages.${pkgs.stdenv.hostPlatform.system}.unavi-assets;
    };
  };

  sops = {
    age.keyFile = "/var/lib/sops-nix/key.txt";
    defaultSopsFile = ../secrets/secrets.yaml;
  };
}
