resource "digitalocean_spaces_bucket" "unavi" {
  name   = "unavi"
  region = "nyc3"

  acl = "public-read"
}
