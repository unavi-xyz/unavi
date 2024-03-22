{
  network = {
    pkgs = import (builtins.fetchGit {
      url = "https://github.com/NixOS/nixpkgs";
      ref = "nixos-23.11";
      rev = "f091af045dff8347d66d186a62d42aceff159456";
    }) { };
  };

  unavi-server-main = { lib, modulesPath, name, ... }: {
    imports = [
      ./common.nix
      (modulesPath + "/virtualisation/digital-ocean-config.nix")
    ] ++ lib.optional (builtins.pathExists ./do-userdata.nix) ./do-userdata.nix;

    deployment.targetHost = "45.55.50.118";
    deployment.targetUser = "root";

    networking.hostName = name;

    system.stateVersion = "23.11";
  };
}
