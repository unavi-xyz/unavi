output "server_ipv4_address" {
  description = "public ipv4 address of the unavi server"
  value       = digitalocean_droplet.unavi.ipv4_address
}

output "server_id" {
  description = "droplet id"
  value       = digitalocean_droplet.unavi.id
}

resource "local_file" "deploy_info" {
  content = jsonencode({
    beta = {
      server_ipv4 = digitalocean_droplet.unavi.ipv4_address
    }
    stable = {
      server_ipv4 = digitalocean_droplet.unavi.ipv4_address
    }
    ssh_public_keys = {
      kayh     = data.digitalocean_ssh_key.kayh.public_key
      gh_unavi = data.digitalocean_ssh_key.gh_unavi.public_key
    }
  })
  filename = "${path.module}/deploy.json"
}
