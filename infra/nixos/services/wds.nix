{ config, lib, ... }:

with lib;

let
  cfg = config.services.unavi-server;
in
{
  options.services.wds = {
    enable = mkEnableOption "wired data store";

    package = mkOption {
      type = types.package;
      description = "wds package to use";
    };

    dataDir = mkOption {
      type = types.str;
      default = "/var/lib/wds";
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

    systemd.services."wds" = {
      description = "wired data store";
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
        exec ${cfg.package}/wds --gossip
      '';
    };
  };
}
