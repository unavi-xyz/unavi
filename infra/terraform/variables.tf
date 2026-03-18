variable "create_beta" {
  description = "whether to create the beta droplet and associated resources"
  type        = bool
  default     = true
}

variable "cloudflare_zone_id" {
  description = "cloudflare zone id"
  type        = string
}

variable "droplet_region" {
  description = "digitalocean region"
  type        = string
  default     = "nyc3"
}

variable "droplet_size" {
  description = "digitalocean droplet size"
  type        = string
  default     = "s-1vcpu-1gb"
}
