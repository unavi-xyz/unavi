data "digitalocean_project" "unavi" {
  name = "unavi"
}

resource "digitalocean_project_resources" "unavi" {
  project = data.digitalocean_project.unavi.id
  resources = [
    digitalocean_droplet.beta.urn,
    digitalocean_droplet.stable.urn,
    digitalocean_spaces_bucket.unavi.urn,
  ]
}
