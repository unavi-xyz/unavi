resource "digitalocean_droplet" "beta" {
  name   = "unavi-beta"
  region = var.droplet_region
  size   = var.droplet_size

  image = "152510211" # nixos-23.11

  ssh_keys = [
    data.digitalocean_ssh_key.kayh.id,
    data.digitalocean_ssh_key.gh_unavi.id,
  ]

  tags = ["beta", "unavi"]
}

resource "digitalocean_droplet" "stable" {
  name   = "unavi-stable"
  region = var.droplet_region
  size   = var.droplet_size

  image = "152510211" # nixos-23.11

  ssh_keys = [
    data.digitalocean_ssh_key.kayh.id,
    data.digitalocean_ssh_key.gh_unavi.id,
  ]

  tags = ["production", "stable", "unavi"]
}
