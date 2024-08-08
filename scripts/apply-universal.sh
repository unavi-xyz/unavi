cd deployments/environments/universal || exit

terraform init -upgrade && terraform apply -auto-approve
