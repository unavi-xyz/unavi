jq -rc '.[].value.ip' ./terraform-output.json | while read -r ip; do
  echo "Adding $ip to known hosts"
  ssh-keyscan -H "$ip" >> "$HOME/.ssh/known_hosts"
done
