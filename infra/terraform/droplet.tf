resource "digitalocean_droplet" "unavi" {
  name   = "unavi-server"
  region = var.droplet_region
  size   = var.droplet_size

  image = "152510211" # nixos-23.11

  ssh_keys = var.ssh_keys

  tags = ["stable", "beta", "production"]
}
