{ config, ... }: {
  terraform = {
    backend = {
      s3 = {
        endpoints = { s3 = "https://nyc3.digitaloceanspaces.com"; };
        bucket = "unavi";
        key = "terraform/terraform.tfstate";
        region = "us-east-1";
        skip_credentials_validation = true;
        skip_metadata_api_check = true;
        skip_region_validation = true;
        skip_requesting_account_id = true;
        skip_s3_checksum = true;
        use_path_style = true;
      };
    };

    required_providers = {
      digitalocean = {
        source = "digitalocean/digitalocean";
        version = "2.36.0";
      };
    };
  };

  provider.digitalocean = { };

  resource = {
    digitalocean_droplet = {
      unavi_server = {
        name = "unavi-server-\${terraform.workspace}";
        tags = [ "\${terraform.workspace}" ];
        region = "nyc3";
        size = "s-1vcpu-1gb";
        image = 152510211;
        ssh_keys = [ 41375001 41382380 ];
      };
    };
  };

  output = {
    unavi_server = {
      value = {
        name = "\${digitalocean_droplet.unavi_server.name}";
        ip = "\${digitalocean_droplet.unavi_server.ipv4_address}";
      };
    };
  };
}
