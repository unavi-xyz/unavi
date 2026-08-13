# Runs a server and N clients.
#
# Clients follow the local server so they share a registry; without a sync
# target a client has none, publishes no presence, and discovers nothing.
#
# `--join` has each later client enter the first client's space directly, the
# namespace pulled from the first client's log. Space entry otherwise needs a
# portal walked by hand, so multiplayer could only be reproduced interactively.

def main [
  --debug-log
  --port: int = 5000
  --clients: int = 2
  --seconds: int = 9999
  --join
] {
  $env.UNAVI_SYNC_TARGETS = $"did:web:localhost%3A($port)"
  # Launching the binary directly leaves Bevy resolving assets next to the
  # executable, rather than from the client crate as `cargo run` does.
  $env.BEVY_ASSET_ROOT = "crates/unavi-client"

  cargo build -p unavi-server -p unavi-client

  let dir = (mktemp -d)
  print $"logs: ($dir)"

  let args = [(if $debug_log { "--debug-log" })] | compact

  ^./target/debug/unavi-server --port ($port | into string) out> $"($dir)/server.log" err> $"($dir)/server.err" &
  sleep 4sec

  ^./target/debug/unavi-client --in-memory ...$args out> $"($dir)/client-0.log" err> $"($dir)/client-0.err" &

  mut ns = null
  if $join {
    for _ in 1..40 {
      let found = (try {
        open $"($dir)/client-0.log"
        | parse --regex 'Joining home.*?(?<ns>[0-9a-f]{64})'
        | get ns.0?
      })
      if $found != null { $ns = $found; break }
      sleep 1sec
    }

    if $ns == null {
      print "failed to read first client's space id"
      ^pkill -f 'unavi-server|unavi-client'
      return
    }
    print $"first client space = ($ns)"
  }

  if $clients > 1 {
    for i in 1..($clients - 1) {
      let join_args = if $ns != null { ["--join", $ns] } else { [] }
      ^./target/debug/unavi-client --in-memory ...$join_args ...$args out> $"($dir)/client-($i).log" err> $"($dir)/client-($i).err" &
    }
  }

  sleep ($seconds * 1sec)
  ^pkill -f 'unavi-server|unavi-client'

  print "=== discovery ==="
  for i in 0..($clients - 1) {
    ^grep -ah "+neighbor|+peer|No bootstrap|Gossip bootstrap" $"($dir)/client-($i).log"
  }
}
