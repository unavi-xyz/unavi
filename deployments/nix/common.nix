{ lib, modulesPath, ... }: {
  imports =
    lib.optional (builtins.pathExists ./do-userdata.nix) ./do-userdata.nix
    ++ [ (modulesPath + "/virtualisation/digital-ocean-config.nix") ];

  boot.tmp.cleanOnBoot = true;
  nix.settings.auto-optimise-store = true;

  services.journald.extraConfig = ''
    SystemMaxUse=100M
    MaxFileSec=14day
  '';

  deployment.targetUser = "root";

  system.stateVersion = "23.11";
}
