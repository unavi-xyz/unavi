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
    server_ipv4 = digitalocean_droplet.unavi.ipv4_address
  })
  filename = "${path.module}/deploy.json"
}
