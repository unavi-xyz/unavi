const protocol_src = "protocol/wit"
const component_srcs = ["crates", "wasm"]
const max_passes = 10

# wit-deps hoists a path dependency's transitive deps by reading its
# materialized `deps` dir, so results depend on what already sits on disk.
# Clearing everything, then re-resolving until the tree stops changing, makes
# the output a function of the manifests (and, in `--locked` mode, the
# checked-in locks) alone.
def dep_state [dirs: list<string>] {
  $dirs
    | each {|d| glob $"($d)/**/*" --no-dir }
    | flatten
    | sort
    | each {|f| $"($f) (open --raw $f | hash sha256)" }
    | str join "\n"
}

def find_dirs [] {
  let protocol_dirs = ls $protocol_src
    | where type == "dir"
    | get name
    | where {|d| $"($d)/deps.toml" | path exists }

  let component_dirs = $component_srcs
    | each {|src| ls $src | where type == "dir" | get name }
    | flatten
    | where {|d| $"($d)/wit/deps.toml" | path exists }
    | each {|d| $"($d)/wit" }

  $protocol_dirs | append $component_dirs
}

# `--locked` materializes `deps` from the committed `deps.lock` files instead
# of re-resolving them, for CI and flake builds that should never silently
# drift onto different dependency content.
def main [--locked] {
  let root = $env.PWD
  let dirs = find_dirs

  for dir in $dirs {
    rm -rf $"($dir)/deps"
    if not $locked {
      rm -f $"($dir)/deps.lock"
    }
  }

  mut state = ""

  for pass in 1..$max_passes {
    print $"Resolving deps, pass ($pass)"

    for dir in $dirs {
      print $"→ ($dir)"

      cd $"($root)/($dir)"
      rm -rf deps
      if $locked {
        wit-deps -d deps -m deps.toml -l deps.lock lock
      } else {
        rm -f deps.lock
        wit-deps -d deps -m deps.toml -l deps.lock update
      }
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

  # The nix sandbox builds from a fileset snapshot with no `.git`, so drift
  # can only be checked where a repo is actually present, like CI.
  if $locked and (".git" | path exists) {
    let locks = $dirs | each {|d| $"($d)/deps.lock" }
    let dirty = (git diff --exit-code -- ...$locks | complete)
    if $dirty.exit_code != 0 {
      error make {
        msg: "deps.lock is out of sync with wit/deps.toml manifests — rerun `update-wasm.nu` without --locked and commit the result"
      }
    }
  }
}
