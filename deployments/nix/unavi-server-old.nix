{ nixpkgs, ... }:
let
  unavi_server = nixpkgs.stdenv.mkDerivation {
    name = "unavi-server";
    buildInputs = [ nixpkgs.unzip ];

    src = ../../x86_64-linux.unavi-server.zip;

    unpackPhase = "unzip $src";
    installPhase = ''
      mkdir -p $out/bin
      cp -r * $out/bin/
    '';
  };
in {
  networking.firewall.allowedTCPPorts = [ 3000 3001 ];

  systemd.services.unavi_server = {
    description = "UNAVI Server";
    after = [ "network.target" ];
    wantedBy = [ "multi-user.target" ];
    serviceConfig = {
      ExecStart = "${unavi_server}/bin/unavi-server";
      Restart = "always";
    };
  };

  sound.enable = true;
  hardware.pulseaudio.enable = false;
  security.rtkit.enable = true;
  services.pipewire = {
    enable = true;
    alsa.enable = true;
    alsa.support32Bit = true;
    pulse.enable = true;
    jack.enable = true;
  };

  environment.systemPackages = with nixpkgs; [
    gcc
    glibc
    jack2
    pipewire
    pipewire.jack
  ];
}
