data "digitalocean_project" "unavi" {
  name = "unavi"
}

resource "digitalocean_project_resources" "unavi" {
  project = data.digitalocean_project.unavi.id
  resources = concat(
    var.create_beta ? [digitalocean_droplet.beta[0].urn] : [],
    [
      digitalocean_droplet.stable.urn,
      digitalocean_spaces_bucket.unavi.urn,
    ],
  )
}
