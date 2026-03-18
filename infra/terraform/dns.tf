resource "cloudflare_record" "app_beta" {
  count   = var.create_beta ? 1 : 0
  zone_id = var.cloudflare_zone_id
  name    = "beta.app"
  content = digitalocean_droplet.beta[0].ipv4_address
  type    = "A"
  ttl     = 1
  proxied = false
}

resource "cloudflare_record" "app_stable" {
  zone_id = var.cloudflare_zone_id
  name    = "app"
  content = digitalocean_droplet.stable.ipv4_address
  type    = "A"
  ttl     = 1
  proxied = false
}

resource "cloudflare_record" "wds_beta" {
  count   = var.create_beta ? 1 : 0
  zone_id = var.cloudflare_zone_id
  name    = "beta.wds"
  content = digitalocean_droplet.beta[0].ipv4_address
  type    = "A"
  ttl     = 1
  proxied = false
}

resource "cloudflare_record" "wds_stable" {
  zone_id = var.cloudflare_zone_id
  name    = "wds"
  content = digitalocean_droplet.stable.ipv4_address
  type    = "A"
  ttl     = 1
  proxied = false
}
