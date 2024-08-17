data "digitalocean_ssh_key" "github" {
  name = "github"
}

data "digitalocean_ssh_key" "kayh" {
  name = "kayh"
}

resource "digitalocean_droplet" "unavi_server" {
  name   = "unavi-server-${terraform.workspace}"
  tags   = [terraform.workspace]
  region = "nyc3"
  size   = "s-1vcpu-1gb"
  image  = 152510211
  ssh_keys = [
    data.digitalocean_ssh_key.github.id,
    data.digitalocean_ssh_key.kayh.id
  ]
}

resource "cloudflare_record" "subdomain_dwn" {
  name    = terraform.workspace == "stable" ? "dwn" : "${terraform.workspace}-dwn"
  proxied = true
  ttl     = 1
  type    = "A"
  value   = digitalocean_droplet.unavi_server.ipv4_address
  zone_id = var.cloudflare_zone_id
}

resource "cloudflare_record" "subdomain_web" {
  name    = terraform.workspace == "stable" ? "app" : terraform.workspace
  proxied = true
  ttl     = 1
  type    = "A"
  value   = digitalocean_droplet.unavi_server.ipv4_address
  zone_id = var.cloudflare_zone_id
}

resource "cloudflare_record" "subdomain_world" {
  name    = terraform.workspace == "stable" ? "world" : "${terraform.workspace}-world"
  proxied = true
  ttl     = 1
  type    = "A"
  value   = digitalocean_droplet.unavi_server.ipv4_address
  zone_id = var.cloudflare_zone_id
}

output "unavi_server" {
  value = {
    domain_dwn   = "${cloudflare_record.subdomain_dwn.name}.unavi.xyz"
    domain_web   = "${cloudflare_record.subdomain_web.name}.unavi.xyz"
    domain_world = "${cloudflare_record.subdomain_world.name}.unavi.xyz"
    ip           = digitalocean_droplet.unavi_server.ipv4_address
    name         = digitalocean_droplet.unavi_server.name
  }
}
