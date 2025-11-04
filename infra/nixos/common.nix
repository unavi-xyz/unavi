{ pkgs, deployInfo, ... }:
{
  system.stateVersion = "23.11";

  nix.settings.experimental-features = [
    "nix-command"
    "flakes"
  ];

  networking.firewall.enable = false;

  services.openssh = {
    enable = true;
    settings = {
      PermitRootLogin = "prohibit-password";
      PasswordAuthentication = false;
    };
  };

  users.users.root.openssh.authorizedKeys.keys = builtins.attrValues deployInfo.ssh_public_keys;

  system.autoUpgrade = {
    enable = true;
    allowReboot = false;
  };

  nix.gc = {
    automatic = true;
    dates = "weekly";
    options = "--delete-older-than 30d";
  };

  security.sudo.wheelNeedsPassword = true;

  environment.systemPackages = with pkgs; [
    curl
    git
    htop
    jq
    vim
    wget
  ];

  time.timeZone = "UTC";
}
