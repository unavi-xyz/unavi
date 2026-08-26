locals {
  base_domain = "unavi.xyz"

  services = {
    stable = {
      unavi_server = {
        port      = 5000
        subdomain = "node"
      }
      web_client = {
        subdomain = "app"
        static    = true
      }
    }
  }

  stable_services = {
    for name, cfg in local.services.stable :
    name => merge(
      { domain = "${cfg.subdomain}.${local.base_domain}" },
      try({ port = cfg.port }, {}),
      try({ static = cfg.static }, {})
    )
  }
}
