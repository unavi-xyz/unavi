{
  config,
  lib,
  deployInfo,
  ...
}:

with lib;

let
  cfg = config.services.unavi-server;
in
{
  options.services.unavi-server = {
    enable = mkEnableOption "unavi server";

    package = mkOption {
      type = types.package;
      description = "unavi-server package to use";
    };

    channel = mkOption {
      type = types.enum [
        "beta"
        "stable"
      ];
      description = "deployment channel";
    };

    domain = mkOption {
      type = types.str;
      default = deployInfo.${cfg.channel}.services.unavi_server.domain;
      description = "Server domain for DID generation";
    };

    port = mkOption {
      type = types.port;
      default = deployInfo.${cfg.channel}.services.unavi_server.port;
      description = "port for unavi-server";
    };

    dataDir = mkOption {
      type = types.str;
      default = "/var/lib/unavi-${cfg.channel}";
      description = "data directory for server state";
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

    systemd.services."unavi-server-${cfg.channel}" = {
      description = "unavi server (${cfg.channel})";
      after = [ "network.target" ];
      wantedBy = [ "multi-user.target" ];

      serviceConfig = {
        Type = "simple";
        User = cfg.user;
        Group = cfg.group;
        WorkingDirectory = cfg.dataDir;
        Restart = "always";
        RestartSec = "10s";
        Environment = "UNAVI_DOMAIN=${cfg.domain}";

        NoNewPrivileges = true;
        PrivateTmp = true;
        ProtectSystem = "strict";
        ProtectHome = true;
        ReadWritePaths = [ cfg.dataDir ];
      };

      script = ''
        exec ${cfg.package}/unavi-server \
          --port ${toString cfg.port}
      '';
    };
  };
}
