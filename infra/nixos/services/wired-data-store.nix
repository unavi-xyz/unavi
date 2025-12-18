{ config, lib, ... }:

with lib;

let
  cfg = config.services.unavi-server;
in
{
  options.services.wired-data-store = {
    enable = mkEnableOption "wired data store";

    package = mkOption {
      type = types.package;
      description = "wired-data-store package to use";
    };

    dataDir = mkOption {
      type = types.str;
      default = "/var/lib/wired-data-store";
      description = "data directory for state";
    };

    user = mkOption {
      type = types.str;
      default = "unavi-${cfg.channel}";
      description = "user account for the service";
    };

    group = mkOption {
      type = types.str;
      default = "unavi-${cfg.channel}";
      description = "group for the service";
    };
  };

  config = mkIf cfg.enable {
    users.users.${cfg.user} = {
      isSystemUser = true;
      inherit (cfg) group;
      home = cfg.dataDir;
      createHome = true;
    };

    users.groups.${cfg.group} = { };

    systemd.services."wired-data-store" = {
      description = "kired data store";
      after = [ "network.target" ];
      wantedBy = [ "multi-user.target" ];

      serviceConfig = {
        Type = "simple";
        User = cfg.user;
        Group = cfg.group;
        WorkingDirectory = cfg.dataDir;
        Restart = "always";
        RestartSec = "10s";

        NoNewPrivileges = true;
        PrivateTmp = true;
        ProtectSystem = "strict";
        ProtectHome = true;
        ReadWritePaths = [ cfg.dataDir ];
      };

      script = ''
        exec ${cfg.package}/wired-data-store --gossip
      '';
    };
  };
}
