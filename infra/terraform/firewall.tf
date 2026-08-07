resource "digitalocean_firewall" "unavi" {
  name = "unavi-server-firewall"

  droplet_ids = concat(
    var.create_beta ? [digitalocean_droplet.beta[0].id] : [],
    [digitalocean_droplet.stable.id],
  )

  dynamic "inbound_rule" {
    for_each = ["22", "80", "443"]
    content {
      protocol         = "tcp"
      port_range       = inbound_rule.value
      source_addresses = ["0.0.0.0/0", "::/0"]
    }
  }

  dynamic "outbound_rule" {
    for_each = ["tcp", "udp"]
    content {
      protocol              = outbound_rule.value
      port_range            = "1-65535"
      destination_addresses = ["0.0.0.0/0", "::/0"]
    }
  }

  outbound_rule {
    protocol              = "icmp"
    destination_addresses = ["0.0.0.0/0", "::/0"]
  }
}
