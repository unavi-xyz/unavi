curl \
  --header "Authorization: Bearer $TERRAFORM_TOKEN" \
  --header "Content-Type: application/vnd.api+json" \
  "https://app.terraform.io/api/v2/workspaces/$TERRAFORM_WORKSPACE/current-state-version" \
  | jq -r '.data.attributes.["hosted-state-download-url"]' \
  | xargs curl -L \
  --header "Authorization: Bearer $TERRAFORM_TOKEN" \
  --output ../terraform/terraform.tfstate

cd ../terraform && terraform show -json > ../nix/show.json
