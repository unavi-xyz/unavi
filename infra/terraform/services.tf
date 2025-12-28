locals {
  base_domain = "unavi.xyz"

  services = {
    beta = {
      unavi_server = {
        port      = 5000
        subdomain = "beta.server"
      }
    }
    stable = {
      unavi_server = {
        port      = 5000
        subdomain = "server"
      }
    }
  }

  # Generate full domains for each service
  beta_services = {
    for service_name, service_config in local.services.beta :
    service_name => {
      port   = service_config.port
      domain = "${service_config.subdomain}.${local.base_domain}"
    }
  }

  stable_services = {
    for service_name, service_config in local.services.stable :
    service_name => {
      port   = service_config.port
      domain = "${service_config.subdomain}.${local.base_domain}"
    }
  }
}
