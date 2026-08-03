# Runs two clients and a server.
# Useful for testing multiplayer.

def main [
  --debug-log
  --port: int = 5000
] {
  let client_features =  []
  let client_args = [
    (if $debug_log { "--debug-log" })
  ] | compact

  # Clients follow the local server so they share a registry; without a sync
  # target a client has none, publishes no presence, and discovers nothing.
  $env.UNAVI_SYNC_TARGETS = $"did:web:localhost%3A($port)"

  cargo build -p unavi-server
  cargo build -p unavi-client ...$client_features

  let client_cmd = ["cargo", "run", "-p", "unavi-client"] | append $client_features | append ["--", "--in-memory"] | append $client_args

  [
    { delay: 0sec, cmd: ["cargo", "run", "-p", "unavi-server", "--", "--port", ($port | into string)] }
    { delay: 1sec, cmd: $client_cmd }
    { delay: 3sec, cmd: $client_cmd }
  ] | par-each { |it| sleep $it.delay; run-external ...$it.cmd }
}
