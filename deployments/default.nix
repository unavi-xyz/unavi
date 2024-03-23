{ self, nixpkgs, deploy-rs }: {
  nixosConfigurations = {
    unavi-server = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [ ./nix/unavi-server.nix ];
    };
  };

  deploy.nodes = {
    server1 = {
      hostname = "104.236.196.193";
      sshUser = "root";

      profiles.system = {
        path = deploy-rs.lib.x86_64-linux.activate.nixos
          self.nixosConfigurations.unavi-server;
      };
    };
  };
}
