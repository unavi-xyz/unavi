{ ... }:
{
  # boot.tmp.cleanOnBoot = true;
  # nix.settings.auto-optimise-store = true;
  #
  # services.journald.extraConfig = ''
  #   SystemMaxUse=100M
  #   MaxFileSec=30day
  # '';
}
