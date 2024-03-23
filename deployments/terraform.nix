{ flake-utils, localSystem, pkgs, terranix, ... }:
let
  terraformConfiguration = terranix.lib.terranixConfiguration {
    system = localSystem;
    modules = [ ./config.nix ];
  };
in {
  apps = {
    tf-apply = flake-utils.lib.mkApp {
      drv = pkgs.writeShellScriptBin "apply" ''
        if [[ -e config.tf.json ]]; then rm -f config.tf.json; fi
        cp ${terraformConfiguration} config.tf.json \
          && ${pkgs.terraform}/bin/terraform init \
          && ${pkgs.terraform}/bin/terraform apply
      '';
    };

    tf-destroy = flake-utils.lib.mkApp {
      drv = pkgs.writeShellScriptBin "destroy" ''
        if [[ -e config.tf.json ]]; then rm -f config.tf.json; fi
        cp ${terraformConfiguration} config.tf.json \
          && ${pkgs.terraform}/bin/terraform init \
          && ${pkgs.terraform}/bin/terraform destroy
      '';
    };
  };

  packages = { inherit terraformConfiguration; };
}
