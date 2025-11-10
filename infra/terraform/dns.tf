resource "cloudflare_record" "space_stable" {
  zone_id = var.cloudflare_zone_id
  name    = "space"
  content = digitalocean_droplet.stable.ipv4_address
  type    = "A"
  ttl     = 1 # automatic
  proxied = false
}

resource "cloudflare_record" "dwn_stable" {
  zone_id = var.cloudflare_zone_id
  name    = "dwn"
  content = digitalocean_droplet.stable.ipv4_address
  type    = "A"
  ttl     = 1
  proxied = false
}

resource "cloudflare_record" "space_beta" {
  zone_id = var.cloudflare_zone_id
  name    = "beta.space"
  content = digitalocean_droplet.beta.ipv4_address
  type    = "A"
  ttl     = 1
  proxied = false
}

resource "cloudflare_record" "dwn_beta" {
  zone_id = var.cloudflare_zone_id
  name    = "beta.dwn"
  content = digitalocean_droplet.beta.ipv4_address
  type    = "A"
  ttl     = 1
  proxied = false
}
