{ pkgs, deployInfo, ... }: {
  boot.tmp.cleanOnBoot = true;

  environment = {
    systemPackages = with pkgs; [
      curl
      git
      htop
      jq
      vim
      wget
    ];
  };

  networking.firewall.enable = true;

  nix = {
    gc = {
      automatic = true;
      dates = "weekly";
      options = "--delete-older-than 30d";
    };
    settings = {
      auto-optimise-store = true;
      experimental-features = [
        "flakes"
        "nix-command"
      ];
    };
  };

  programs.fish = {
    enable = true;
    interactiveShellInit = ''
      fish_vi_key_bindings
    '';
  };

  security.sudo.wheelNeedsPassword = true;

  services = {
    journald.extraConfig = ''
      MaxRetentionSec=7d
      SystemMaxUse=1G
    '';
    openssh = {
      enable = true;
      settings = {
        PasswordAuthentication = false;
        PermitRootLogin = "prohibit-password";
      };
    };
  };

  # deploy-rs pushes are the only update path; a second, independently-timed
  # autoUpgrade would race it and make "deployed" drift from "what CI last pushed."
  system.stateVersion = "23.11";

  time.timeZone = "America/New_York";

  users.users.root = {
    openssh.authorizedKeys.keys = builtins.attrValues deployInfo.ssh_public_keys;
    shell = pkgs.fish;
  };
}
