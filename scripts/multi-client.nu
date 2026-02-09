# Runs two clients and a server.
# Useful for testing multiplayer.

def main [
  --debug-network
  --debug-physics
  --debug-log
] {
  let use_devtools = $debug_network or $debug_physics
  let client_features = if $use_devtools { ["--features", "devtools", "--features", "mdns"] } else { [] }
  let client_args = [
    (if $debug_network { "--debug-network" })
    (if $debug_physics { "--debug-physics" })
    (if $debug_log { "--debug-log" })
  ] | compact

  cargo build -p unavi-server
  cargo build -p unavi-client ...$client_features

  let client_cmd = ["cargo", "run", "-p", "unavi-client"] | append $client_features | append ["--", "--in-memory"] | append $client_args

  [
    { delay: 0sec, cmd: ["cargo", "run", "-p", "unavi-server"] }
    { delay: 1sec, cmd: $client_cmd }
    { delay: 5sec, cmd: $client_cmd }
  ] | par-each { |it| sleep $it.delay; run-external ...$it.cmd }
}
