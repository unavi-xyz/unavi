{ flake-utils, pkgs, ... }:
{
  apps = {
    tf-universal-apply = flake-utils.lib.mkApp {
      drv = pkgs.writeShellScriptBin "universal-apply" ''
        cd deployments/environments/universal

        ${pkgs.terraform}/bin/terraform init -upgrade \
          && ${pkgs.terraform}/bin/terraform apply -auto-approve
      '';
    };

    tf-format = flake-utils.lib.mkApp {
      drv = pkgs.writeShellScriptBin "format" ''
        cd deployments/environments/universal
        ${pkgs.terraform}/bin/terraform fmt . 
        cd ../common
        ${pkgs.terraform}/bin/terraform fmt . 
        cd ../channel
        ${pkgs.terraform}/bin/terraform fmt . 
      '';
    };

    tf-channel-apply = flake-utils.lib.mkApp {
      drv = pkgs.writeShellScriptBin "channel-apply" ''
        cd deployments/environments/channel

        ${pkgs.terraform}/bin/terraform init -upgrade \
          && ${pkgs.terraform}/bin/terraform apply -auto-approve

        mkdir -p ../../output/$TF_WORKSPACE
          
        ${pkgs.terraform}/bin/terraform output -json > ../../output/$TF_WORKSPACE/terraform-output.json

        jq -rc '.[].value.ip' ../../output/$TF_WORKSPACE/terraform-output.json | while read -r ip; do
          echo "Adding $ip to known hosts"
          ssh-keyscan -H $ip >> $HOME/.ssh/known_hosts
        done
      '';
    };

    tf-channel-destroy = flake-utils.lib.mkApp {
      drv = pkgs.writeShellScriptBin "channel-destroy" ''
        cd deployments/environments/channel

        ${pkgs.terraform}/bin/terraform init \
          && ${pkgs.terraform}/bin/terraform destroy -auto-approve

        rm -rf ../../output/$TF_WORKSPACE
      '';
    };
  };
}
