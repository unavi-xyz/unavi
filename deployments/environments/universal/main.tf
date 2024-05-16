resource "digitalocean_ssh_key" "github" {
  name       = "github"
  public_key = file("./keys/github.pub")
}

resource "digitalocean_ssh_key" "kayh" {
  name       = "kayh"
  public_key = file("./keys/kayh.pub")
}
