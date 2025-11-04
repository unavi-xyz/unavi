data "digitalocean_ssh_key" "kayh" {
  name = "kayh"
}

data "digitalocean_ssh_key" "gh_unavi" {
  name = "gh-unavi"
}

output "ssh_public_keys" {
  description = "ssh public keys for authorized_keys"
  value = {
    kayh     = data.digitalocean_ssh_key.kayh.public_key
    gh_unavi = data.digitalocean_ssh_key.gh_unavi.public_key
  }
}
