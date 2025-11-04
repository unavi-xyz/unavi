variable "cloudflare_zone_id" {
  description = "cloudflare zone id for unavi.xyz"
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
  default     = "s-2vcpu-4gb"
}

variable "ssh_keys" {
  description = "ssh key fingerprints for droplet access"
  type        = list(string)
}
