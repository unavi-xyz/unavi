# Runs two clients and a server.
# Useful for testing multiplayer.

def main [
  --debug-network
] {
  cargo build -p unavi-server
  cargo build -p unavi-client

  let run_client = {||
    cargo run -p unavi-client -- --in-memory ...(if $debug_network { ["--debug-network"] } else { [] })
  }

  [
    { cmd: {|| cargo run -p unavi-server } }
    { cmd: {|| sleep 1sec; do $run_client } }
    { cmd: {|| sleep 3sec; do $run_client } }
  ] | par-each { |it| do $it.cmd }
}
