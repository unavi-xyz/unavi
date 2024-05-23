{
  lib,
  modulesPath,
  pkgs,
  ...
}:
{
  imports = lib.optional (builtins.pathExists ./do-userdata.nix) ./do-userdata.nix ++ [
    (modulesPath + "/virtualisation/digital-ocean-config.nix")
  ];

  boot.tmp.cleanOnBoot = true;

  nix = {
    settings = {
      auto-optimise-store = true;
      experimental-features = [
        "nix-command"
        "flakes"
      ];
    };

    gc = {
      automatic = true;
      dates = "weekly";
      options = "--delete-older-than 7d";
    };
  };

  services.journald.extraConfig = ''
    SystemMaxUse=100M
    MaxFileSec=7day
  '';

  system = {
    autoUpgrade.enable = true;
    stateVersion = "23.11";
  };

  environment.systemPackages = with pkgs; [
    htop
    vim
  ];
}
