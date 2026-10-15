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
      package = inputs.self.packages.${pkgs.system}.unavi-node;
    };
  };

  sops = {
    age.keyFile = "/var/lib/sops-nix/key.txt";
    defaultSopsFile = ../secrets/secrets.yaml;
  };
}
