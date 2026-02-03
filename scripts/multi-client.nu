# Runs two clients and a server.
# Useful for testing multiplayer.

def main [
  --debug-network
] {
  let client_features = if $debug_network { ["--features", "devtools-network", "--features", "mdns"] } else { [] }
  let client_args = if $debug_network { ["--debug-network"] } else { [] }

  cargo build -p unavi-server
  cargo build -p unavi-client ...$client_features

  let run_client = {||
    cargo run -p unavi-client ...$client_features -- --in-memory ...$client_args
  }

  [
    { cmd: {|| cargo run -p unavi-server } }
    { cmd: {|| sleep 1sec; do $run_client } }
    { cmd: {|| sleep 4sec; do $run_client } }
  ] | par-each { |it| do $it.cmd }
}
