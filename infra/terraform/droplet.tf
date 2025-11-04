resource "digitalocean_droplet" "unavi" {
  name   = "unavi-server"
  region = var.droplet_region
  size   = var.droplet_size

  image = "152510211" # nixos-23.11

  ssh_keys = [
    data.digitalocean_ssh_key.kayh.id,
    data.digitalocean_ssh_key.gh_unavi.id,
  ]

  tags = ["beta", "production", "stable"]
}
