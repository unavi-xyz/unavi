output "beta_ipv4_address" {
  description = "public ipv4 address of the beta server"
  value       = var.create_beta ? digitalocean_droplet.beta[0].ipv4_address : null
}

output "beta_id" {
  description = "beta droplet id"
  value       = var.create_beta ? digitalocean_droplet.beta[0].id : null
}

output "stable_ipv4_address" {
  description = "public ipv4 address of the stable server"
  value       = digitalocean_droplet.stable.ipv4_address
}

output "stable_id" {
  description = "stable droplet id"
  value       = digitalocean_droplet.stable.id
}

resource "local_file" "deploy_info" {
  content = jsonencode({
    beta = var.create_beta ? {
      server_ipv4 = digitalocean_droplet.beta[0].ipv4_address
      services    = local.beta_services
    } : null
    stable = {
      server_ipv4 = digitalocean_droplet.stable.ipv4_address
      services    = local.stable_services
    }
    ssh_public_keys = {
      kayh     = data.digitalocean_ssh_key.kayh.public_key
      gh_unavi = data.digitalocean_ssh_key.gh_unavi.public_key
    }
  })
  filename = "${path.module}/deploy.json"
}
