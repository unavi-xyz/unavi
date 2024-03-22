# Deployments

UNAVI deployment using Terraform and NixOS [Morph](https://github.com/DBCDK/morph/tree/master).

## Usage

### Deploying Nix configurations

1. Pull latest terraform state

```bash
export TERRAFORM_TOKEN="..."
export TERRAFORM_WORKSPACE="..."

cd scripts
sh pull.sh
```

2. Morph deploy

```bash
export DIGITALOCEAN_TOKEN="..."

cd nix
morph deploy network.nix switch
```
