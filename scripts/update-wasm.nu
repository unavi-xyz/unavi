const protocol_src = "protocol/wit"
const component_srcs = ["crates", "wasm"]
const max_passes = 10

# wit-deps hoists a path dependency's transitive deps by reading its
# materialized `deps` dir, so results depend on what already sits on disk.
# Clearing everything, then updating until the tree stops changing, makes the
# output a function of the manifests alone.
def dep_state [dirs: list<string>] {
  $dirs
    | each {|d| glob $"($d)/**/*" --no-dir }
    | flatten
    | sort
    | each {|f| $"($f) (open --raw $f | hash sha256)" }
    | str join "\n"
}

let root = $env.PWD

let protocol_dirs = ls $protocol_src
  | where type == "dir"
  | get name
  | where {|d| $"($d)/deps.toml" | path exists }

let component_dirs = $component_srcs
  | each {|src| ls $src | where type == "dir" | get name }
  | flatten
  | where {|d| $"($d)/wit/deps.toml" | path exists }
  | each {|d| $"($d)/wit" }

let dirs = $protocol_dirs | append $component_dirs

for dir in $dirs {
  rm -rf $"($dir)/deps"
  rm -f $"($dir)/deps.lock"
}

mut state = ""

for pass in 1..$max_passes {
  print $"Updating deps, pass ($pass)"

  for dir in $dirs {
    print $"→ ($dir)"

    cd $"($root)/($dir)"
    rm -rf deps
    rm -f deps.lock
    wit-deps -d deps -m deps.toml -l deps.lock update
    cd $root
  }

  let next = dep_state $dirs

  if $next == $state {
    break
  }

  if $pass == $max_passes {
    error make { msg: $"deps did not converge after ($max_passes) passes" }
  }

  $state = $next
}
