jq -c '.values.root_module.resources[]' ../nix/show.json | while read -r server; do
  ip=$(echo "$server" | jq -r ".values.ipv4_address")
  echo "Adding $ip to known hosts"
  ssh-keyscan -H "$ip" >> "$HOME/.ssh/known_hosts"
done
