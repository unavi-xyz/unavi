jq -c '.values.root_module.resources[] | select(.name == "unavi-server")' ../nix/show.json | while read -r server; do
  name=$(echo "$server" | jq -r ".values.name")
  ip=$(echo "$server" | jq -r ".values.ipv4_address")
  echo "Uploading to $name ($ip)"

  ssh -n -i "$HOME/.ssh/id_ed25519" "root@$ip" "mkdir -p /var/lib/unavi-server/"
  scp -i "$HOME/.ssh/id_ed25519" "x86_64-linux.unavi-server.zip root@$ip:/var/lib/unavi-server/unavi-server.zip"
done
