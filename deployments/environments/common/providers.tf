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
      version = "4.38.0"
    }
    digitalocean = {
      source  = "digitalocean/digitalocean"
      version = "2.39.2"
    }
  }
}

provider "cloudflare" {}
provider "digitalocean" {}
