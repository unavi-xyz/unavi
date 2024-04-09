variable "cloudflare_zone_id" {
  type = string
}

terraform {
  backend "s3" {
    endpoint                    = "https://nyc3.digitaloceanspaces.com"
    bucket                      = "unavi"
    key                         = "terraform/terraform.tfstate"
    region                      = "us-east-1"
    skip_credentials_validation = true
    skip_metadata_api_check     = true
    skip_region_validation      = true
    skip_requesting_account_id  = true
    skip_s3_checksum            = true
    use_path_style              = true
  }

  required_providers {
    cloudflare = {
      source  = "cloudflare/cloudflare"
      version = "4.28.0"
    }
    digitalocean = {
      source  = "digitalocean/digitalocean"
      version = "2.36.0"
    }
  }
}

provider "cloudflare" {}
provider "digitalocean" {}

resource "digitalocean_droplet" "unavi_server" {
  name   = "unavi-server-${terraform.workspace}"
  tags   = [terraform.workspace]
  region = "nyc3"
  size   = "s-1vcpu-1gb"
  image  = 152510211
  ssh_keys = [
    41375001, // GitHub
    41382380, // Kayh
  ]
}

resource "cloudflare_record" "unavi_subdomain" {
  name    = terraform.workspace == "stable" ? "app" : terraform.workspace
  proxied = true
  ttl     = 1
  type    = "A"
  value   = digitalocean_droplet.unavi_server.ipv4_address
  zone_id = var.cloudflare_zone_id
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

output "unavi_server" {
  value = {
    domain = "${cloudflare_record.unavi_subdomain.name}.unavi.xyz"
    ip     = digitalocean_droplet.unavi_server.ipv4_address
    name   = digitalocean_droplet.unavi_server.name
  }
}
