provider "digitalocean" {}

resource "digitalocean_droplet" "server" {
  name     = "server${count.index + 1}"
  region   = "nyc3"
  size     = "s-1vcpu-1gb"
  image    = 152510211
  ssh_keys = [41375001]

  count = 2
}

output "server_ip" {
  value = digitalocean_droplet.server.*.ipv4_address
}
