{ flake-utils, pkgs, ... }:
{
  apps = {
    tf-universal-apply = flake-utils.lib.mkApp {
      drv = pkgs.writeShellApplication {
        name = "tf-universal-apply";
        runtimeInputs = (with pkgs; [ terraform ]);
        text = ''
          cd deployments/environments/universal

          terraform init -upgrade \
            && terraform apply -auto-approve
        '';
      };
    };

    tf-format = flake-utils.lib.mkApp {
      drv = pkgs.writeShellApplication {
        name = "tf-format";
        runtimeInputs = (with pkgs; [ terraform ]);
        text = ''
          cd deployments/environments/universal
          terraform fmt . 
          cd ../common
          terraform fmt . 
          cd ../channel
          terraform fmt . 
        '';
      };
    };

    tf-channel-apply = flake-utils.lib.mkApp {
      drv = pkgs.writeShellApplication {
        name = "tf-channel-apply";
        runtimeInputs = (with pkgs; [ terraform ]);
        text = ''
          cd deployments/environments/channel

          terraform init -upgrade \
            && terraform apply -auto-approve

          mkdir -p "../../output/$TF_WORKSPACE"
            
          terraform output -json > "../../output/$TF_WORKSPACE/terraform-output.json"

          jq -rc '.[].value.ip' "../../output/$TF_WORKSPACE/terraform-output.json" | while read -r ip; do
            echo "Adding $ip to known hosts"
            ssh-keyscan -H "$ip" >> "$HOME/.ssh/known_hosts"
          done
        '';
      };
    };

    tf-channel-destroy = flake-utils.lib.mkApp {
      drv = pkgs.writeShellApplication {
        name = "tf-channel-destroy";
        runtimeInputs = (with pkgs; [ terraform ]);
        text = ''
          cd deployments/environments/channel

          terraform init \
            && terraform destroy -auto-approve

          rm -rf "../../output/$TF_WORKSPACE"
        '';
      };
    };
  };
}
