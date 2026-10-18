{
  config,
  lib,
  deployInfo,
  ...
}:

with lib;

let
  cfg = config.services.unavi-node;
in
{
  options.services.unavi-node = {
    enable = mkEnableOption "unavi node";

    package = mkOption {
      type = types.package;
      description = "unavi-node package to use";
    };

    domain = mkOption {
      type = types.str;
      default = deployInfo.stable.services.unavi_node.domain;
      description = "node domain for DID generation";
    };

    port = mkOption {
      type = types.port;
      default = deployInfo.stable.services.unavi_node.port;
      description = "port for unavi-node";
    };

    dataDir = mkOption {
      type = types.str;
      default = "/var/lib/unavi-node";
      description = "data directory for node state";
    };

    user = mkOption {
      type = types.str;
      default = "unavi-node";
      description = "user account for the service";
    };

    group = mkOption {
      type = types.str;
      default = "unavi-node";
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

    systemd.services.unavi-node = {
      description = "unavi node";
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
        exec ${cfg.package}/bin/unavi-node \
          --domain ${cfg.domain} \
          --port ${toString cfg.port}
      '';
    };
  };
}
