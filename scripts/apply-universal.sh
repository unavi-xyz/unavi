cd deploy/profile/universal || exit

terraform init -upgrade && terraform apply -auto-approve
