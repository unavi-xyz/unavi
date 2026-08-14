# Runs a server and N clients, streaming every process's output to the
# terminal.
#
# Clients follow the local server so they share a registry; without a sync
# target a client has none, publishes no presence, and discovers nothing.
#
# `--join` has each later client enter the first client's space directly, the
# namespace pulled from the first client's log. Space entry otherwise needs a
# portal walked by hand, so multiplayer could only be reproduced interactively.
#
# `--logs` writes each process's output to files in that directory; otherwise
# a temp dir is used (printed at startup).

def stream-cmd [
  cmd: list
  log: string
  --err  # the process logs to stderr rather than stdout
] {
  if $err {
    if $log != "" {
      job spawn {
        run-external ...$cmd e>| tee { save $"($log)" } | lines | each { |l| print -n $"($l)\n" }
      }
    } else {
      job spawn {
        run-external ...$cmd e>| lines | each { |l| print -n $"($l)\n" }
      }
    }
  } else {
    if $log != "" {
      job spawn {
        run-external ...$cmd | tee { save $"($log)" } | lines | each { |l| print -n $"($l)\n" }
      }
    } else {
      job spawn {
        run-external ...$cmd | lines | each { |l| print -n $"($l)\n" }
      }
    }
  }
}

def main [
  --debug-log
  --port: int = 5000
  --clients: int = 2
  --seconds: int = 9999
  --join
  --logs: string
] {
  $env.UNAVI_SYNC_TARGETS = $"did:web:localhost%3A($port)"
  # Launching the binary directly leaves Bevy resolving assets next to the
  # executable, rather than from the client crate as `cargo run` does.
  $env.BEVY_ASSET_ROOT = "crates/unavi-client"

  cargo build -p unavi-server -p unavi-client

  let dir = if $logs != null {
    mkdir $logs
    $logs
  } else {
    (mktemp -d)
  }
  print $"logs: ($dir)"

  let args = [(if $debug_log { "--debug-log" })] | compact

  let server_id = (stream-cmd ["./target/debug/unavi-server", "--port", ($port | into string)] $"($dir)/server.log")
  sleep 2sec

  let client_0_cmd = ["./target/debug/unavi-client", "--in-memory"] | append $args
  mut client_ids = [
    (stream-cmd $client_0_cmd $"($dir)/client-0.log" --err)
  ]

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
      try { job kill $server_id }
      for $id in $client_ids { try { job kill $id } }
      return
    }
    print $"first client space = ($ns)"
  }

  if $clients > 1 {
    for i in 1..($clients - 1) {
      let join_args = if $ns != null { ["--join", $ns] } else { [] }
      let cmd = ["./target/debug/unavi-client", "--in-memory"] | append $join_args | append $args
      $client_ids = ($client_ids | append (stream-cmd $cmd $"($dir)/client-($i).log" --err))
      sleep 1sec
    }
  }

  sleep ($seconds * 1sec)

  try { job kill $server_id }
  for $id in $client_ids { try { job kill $id } }

  print "=== discovery ==="
  for i in 0..($clients - 1) {
    ^grep -ah "+neighbor|+peer|No bootstrap|Gossip bootstrap" $"($dir)/client-($i).log"
  }
}
