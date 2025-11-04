cloudflare_zone_id = "c9bd6fd96e22ba8f6f6bb0373883919c"

droplet_region = "nyc3"
droplet_size   = "s-1vcpu-1gb"

# get from: doctl compute ssh-key list --format ID,FingerPrint
ssh_keys = [
  "09:eb:2a:12:3d:36:c7:d6:fd:e6:ac:37:8d:e3:7d:8c" # kayh
]
