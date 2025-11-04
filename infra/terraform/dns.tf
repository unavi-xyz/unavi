resource "cloudflare_record" "world_stable" {
  zone_id = var.cloudflare_zone_id
  name    = "world"
  content = digitalocean_droplet.unavi.ipv4_address
  type    = "A"
  ttl     = 1 # automatic
  proxied = false
}

resource "cloudflare_record" "dwn_stable" {
  zone_id = var.cloudflare_zone_id
  name    = "dwn"
  content = digitalocean_droplet.unavi.ipv4_address
  type    = "A"
  ttl     = 1
  proxied = false
}

resource "cloudflare_record" "world_beta" {
  zone_id = var.cloudflare_zone_id
  name    = "beta.world"
  content = digitalocean_droplet.unavi.ipv4_address
  type    = "A"
  ttl     = 1
  proxied = false
}

resource "cloudflare_record" "dwn_beta" {
  zone_id = var.cloudflare_zone_id
  name    = "beta.dwn"
  content = digitalocean_droplet.unavi.ipv4_address
  type    = "A"
  ttl     = 1
  proxied = false
}

# ipv6 records
resource "cloudflare_record" "world_stable_ipv6" {
  zone_id = var.cloudflare_zone_id
  name    = "world"
  content = digitalocean_droplet.unavi.ipv6_address
  type    = "AAAA"
  ttl     = 1
  proxied = false
}

resource "cloudflare_record" "dwn_stable_ipv6" {
  zone_id = var.cloudflare_zone_id
  name    = "dwn"
  content = digitalocean_droplet.unavi.ipv6_address
  type    = "AAAA"
  ttl     = 1
  proxied = false
}

resource "cloudflare_record" "world_beta_ipv6" {
  zone_id = var.cloudflare_zone_id
  name    = "beta.world"
  content = digitalocean_droplet.unavi.ipv6_address
  type    = "AAAA"
  ttl     = 1
  proxied = false
}

resource "cloudflare_record" "dwn_beta_ipv6" {
  zone_id = var.cloudflare_zone_id
  name    = "beta.dwn"
  content = digitalocean_droplet.unavi.ipv6_address
  type    = "AAAA"
  ttl     = 1
  proxied = false
}
