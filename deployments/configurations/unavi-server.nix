{ pkgs, ... }:
let
  unavi-server = pkgs.rustPlatform.buildRustPackage {
    buildAndTestSubdir = "crates/unavi-server";
    name = "unavi-server";
    src = pkgs.lib.cleanSource ../../.;

    cargoLock = {
      lockFile = ../../Cargo.lock;
      outputHashes = {
        "wasm-bridge-0.3.0" =
          "sha256-R7qLWzyd0DlU6/rAr2/T3kuhRLTcz3jAHmLzlznRp+s=";
      };
    };
  };
in {
  imports = [ ./common.nix ];

  networking.firewall.allowedTCPPorts = [ 3000 3001 ];

  systemd.services.unavi_server = {
    description = "UNAVI Server";
    after = [ "network.target" ];
    wantedBy = [ "multi-user.target" ];
    serviceConfig = {
      ExecStart = "${unavi-server}/bin/unavi-server";
      Restart = "always";
    };
  };
}
