{ config, lib, ... }:

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

    port = mkOption {
      type = types.port;
      default = if cfg.channel == "beta" then 5000 else 5001;
      description = "port for unavi-server";
    };

    dwnPort = mkOption {
      type = types.port;
      default = if cfg.channel == "beta" then 8080 else 8081;
      description = "port for dwn-server";
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

        NoNewPrivileges = true;
        PrivateTmp = true;
        ProtectSystem = "strict";
        ProtectHome = true;
        ReadWritePaths = [ cfg.dataDir ];
      };

      script = ''
        exec ${cfg.package}/unavi-server \
          --port ${toString cfg.port} \
          --dwn \
          --dwn-port ${toString cfg.dwnPort}
      '';
    };
  };
}
