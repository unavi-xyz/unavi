resource "digitalocean_ssh_key" "github" {
  name       = "github"
  public_key = file("./keys/github.pub")
}

resource "digitalocean_ssh_key" "kayh" {
  name       = "kayh"
  public_key = file("./keys/kayh.pub")
}

resource "cloudflare_page_rule" "cache_html" {
  zone_id  = var.cloudflare_zone_id
  target   = "*.unavi.xyz/*.html"
  priority = 2
  actions {
    cache_level = "bypass"
  }
}

resource "cloudflare_page_rule" "cache_all" {
  zone_id  = var.cloudflare_zone_id
  target   = "*.unavi.xyz/*"
  priority = 1
  actions {
    cache_level    = "cache_everything"
    edge_cache_ttl = 7200
  }
}
