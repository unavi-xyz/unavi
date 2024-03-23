{ flake-utils, localSystem, pkgs, terranix, ... }:
let
  terraformConfiguration = terranix.lib.terranixConfiguration {
    system = localSystem;
    modules = [ ./config.nix ];
  };

  workspaceInput = ''
    if [ $# -eq 0 ]; then
      workspace="default"
    else
      workspace=$1
    fi
  '';
in {
  apps = {
    tf-apply = flake-utils.lib.mkApp {
      drv = pkgs.writeShellScriptBin "apply" ''
        ${workspaceInput}

        if [[ -e config.tf.json ]]; then rm -f config.tf.json; fi

        cp ${terraformConfiguration} config.tf.json \
          && ${pkgs.terraform}/bin/terraform init \
          && ${pkgs.terraform}/bin/terraform workspace select -or-create $workspace \
          && ${pkgs.terraform}/bin/terraform apply -auto-approve

        mkdir -p deployments/terraform/$workspace
          
        ${pkgs.terraform}/bin/terraform output -json > deployments/terraform/$workspace/terraform-output.json

        jq -rc '.[].value.ip' deployments/terraform/$workspace/terraform-output.json | while read -r ip; do
          echo "Adding $ip to known hosts"
          ssh-keyscan -H "$ip" >> "$HOME/.ssh/known_hosts"
        done
      '';
    };

    tf-destroy = flake-utils.lib.mkApp {
      drv = pkgs.writeShellScriptBin "destroy" ''
        ${workspaceInput}

        if [[ -e config.tf.json ]]; then rm -f config.tf.json; fi

        cp ${terraformConfiguration} config.tf.json \
          && ${pkgs.terraform}/bin/terraform init \
          && ${pkgs.terraform}/bin/terraform workspace select -or-create $workspace \
          && ${pkgs.terraform}/bin/terraform destroy -auto-approve

        rm -rf deployments/terraform/$workspace
      '';
    };
  };

  packages = { inherit terraformConfiguration; };
}
