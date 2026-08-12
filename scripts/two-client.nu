# Runs a server and two clients, the second joining the first's space directly.
#
# Space entry otherwise needs a portal walked by hand, so multiplayer could only
# be reproduced interactively. `--join` takes the namespace out of the first
# client's log and hands it to the second.

def main [
  --debug-log
  --port: int = 5000
  --seconds: int = 45
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

  ^./target/debug/unavi-client --in-memory ...$args out> $"($dir)/alice.log" err> $"($dir)/alice.err" &

  mut ns = null
  for _ in 1..40 {
    let found = (try {
      open $"($dir)/alice.log"
      | parse --regex 'Joining home.*?(?<ns>[0-9a-f]{64})'
      | get ns.0?
    })
    if $found != null { $ns = $found; break }
    sleep 1sec
  }

  if $ns == null {
    print "failed to read alice's space id"
    return
  }
  print $"alice space = ($ns)"

  ^./target/debug/unavi-client --in-memory --join $ns ...$args out> $"($dir)/bob.log" err> $"($dir)/bob.err" &

  sleep ($seconds * 1sec)
  ^pkill -f 'unavi-server|unavi-client'

  print "=== discovery ==="
  ^grep -ah "+neighbor|+peer|No bootstrap|Gossip bootstrap" $"($dir)/alice.log" $"($dir)/bob.log"
}
