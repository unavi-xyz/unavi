cd deployments/environments/channel || exit

terraform init -upgrade && terraform apply -auto-approve

mkdir -p "../../output/$TF_WORKSPACE"
terraform output -json > "../../output/$TF_WORKSPACE/terraform-output.json"

jq -rc '.[].value.ip' "../../output/$TF_WORKSPACE/terraform-output.json" | while read -r ip; do
  echo "Adding $ip to known hosts"
  ssh-keyscan -H "$ip" >> "$HOME/.ssh/known_hosts"
done
