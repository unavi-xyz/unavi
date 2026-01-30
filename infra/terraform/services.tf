locals {
  base_domain = "unavi.xyz"

  services = {
    beta = {
      unavi_server = {
        port      = 5000
        subdomain = "beta.wds"
      }
      web_client = {
        subdomain = "beta.app"
        static    = true
      }
    }
    stable = {
      unavi_server = {
        port      = 5000
        subdomain = "wds"
      }
      web_client = {
        subdomain = "app"
        static    = true
      }
    }
  }

  beta_services = {
    for name, cfg in local.services.beta :
    name => merge(
      { domain = "${cfg.subdomain}.${local.base_domain}" },
      try({ port = cfg.port }, {}),
      try({ static = cfg.static }, {})
    )
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
