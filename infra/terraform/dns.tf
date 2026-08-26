resource "cloudflare_record" "app_stable" {
  zone_id = var.cloudflare_zone_id
  name    = "app"
  content = digitalocean_droplet.stable.ipv4_address
  type    = "A"
  ttl     = 1
  proxied = true
}

resource "cloudflare_record" "node_stable" {
  zone_id = var.cloudflare_zone_id
  name    = "node"
  content = digitalocean_droplet.stable.ipv4_address
  type    = "A"
  ttl     = 1
  proxied = true
}
