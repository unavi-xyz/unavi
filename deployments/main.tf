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
      version = "3.010"
    }
    digitalocean = {
      source  = "digitalocean/digitalocean"
      version = "2.36.0"
    }
  }
}

provider "cloudflare" {}
provider "digitalocean" {}

resource "digitalocean_domain" "unavi_domain" {
  name = "unavi.xyz"
}

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

output "unavi_server" {
  value = {
    domain = "${cloudflare_record.unavi_subdomain.name}.unavi.xyz"
    ip     = digitalocean_droplet.unavi_server.ipv4_address
    name   = digitalocean_droplet.unavi_server.name
  }
}
