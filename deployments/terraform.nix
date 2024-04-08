{
  flake-utils,
  localSystem,
  pkgs,
  ...
}:
{
  apps = {
    tf-apply = flake-utils.lib.mkApp {
      drv = pkgs.writeShellScriptBin "apply" ''
        cd deployments

        ${pkgs.terraform}/bin/terraform init \
          && ${pkgs.terraform}/bin/terraform apply -auto-approve

        mkdir -p terraform/$TF_WORKSPACE
          
        ${pkgs.terraform}/bin/terraform output -json > terraform/$TF_WORKSPACE/terraform-output.json

        jq -rc '.[].value.ip' terraform/$TF_WORKSPACE/terraform-output.json | while read -r ip; do
          echo "Adding $ip to known hosts"
          ssh-keyscan -H "$ip" >> "$HOME/.ssh/known_hosts"
        done
      '';
    };

    tf-format = flake-utils.lib.mkApp {
      drv = pkgs.writeShellScriptBin "format" ''
        cd deployments
        ${pkgs.terraform}/bin/terraform fmt . 
      '';
    };

    tf-destroy = flake-utils.lib.mkApp {
      drv = pkgs.writeShellScriptBin "destroy" ''
        cd deployments

        ${pkgs.terraform}/bin/terraform init \
          && ${pkgs.terraform}/bin/terraform destroy -auto-approve

        rm -rf terraform/$TF_WORKSPACE
      '';
    };
  };
}
