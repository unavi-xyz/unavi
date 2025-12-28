resource "cloudflare_record" "server_stable" {
  zone_id = var.cloudflare_zone_id
  name    = "server"
  content = digitalocean_droplet.stable.ipv4_address
  type    = "A"
  ttl     = 1 # automatic
  proxied = false
}

resource "cloudflare_record" "server_beta" {
  zone_id = var.cloudflare_zone_id
  name    = "beta.server"
  content = digitalocean_droplet.beta.ipv4_address
  type    = "A"
  ttl     = 1
  proxied = false
}
