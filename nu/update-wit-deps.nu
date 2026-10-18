const protocol_src = "protocol/wit"
const component_srcs = ["crates", "hsd"]

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

# Path dependencies that this script resolves itself. A manifest may also point
# at a dir without a `deps.toml`, which carries no ordering constraint.
def path_deps [dir: string, candidates: list<string>] {
  open $"($dir)/deps.toml"
  | values
  | each {|spec| if ($spec | describe) == "string" { $spec } else { $spec.path? } }
  | compact
  | each {|p| [$dir $p] | path join | path expand }
  | where {|p| $p in $candidates }
}

# wit-deps hoists a path dependency's materialized `deps` dir into its
# consumer's, reading whatever is already on disk. Resolving in dependency
# order — rather than iterating until the tree stops changing — makes the
# output a function of the manifests in a single pass.
def resolve_order [dirs: list<string>] {
  mut remaining = ($dirs | each {|d| $d | path expand })
  mut order = []

  while ($remaining | is-not-empty) {
    let ready = $remaining | where {|d| (path_deps $d $remaining | is-empty) }
    if ($ready | is-empty) {
      error make { msg: $"path dependency cycle among: ($remaining | str join ', ')" }
    }
    $order = $order | append $ready
    $remaining = $remaining | where {|d| not ($d in $ready) }
  }

  $order
}

# `--locked` materializes `deps` from the committed `deps.lock` files instead
# of re-resolving them, for CI and flake builds that should never silently
# drift onto different dependency content.
def main [--locked] {
  let root = $env.PWD | path expand
  let dirs = resolve_order (find_dirs)

  for dir in $dirs {
    rm -rf $"($dir)/deps"
    cd $dir
    if $locked {
      wit-deps -d deps -m deps.toml -l deps.lock lock
    } else {
      rm -f deps.lock
      wit-deps -d deps -m deps.toml -l deps.lock update
    }
    cd $root
  }

  # The nix sandbox builds from a fileset snapshot with no `.git`, so drift
  # can only be checked where a repo is actually present, like CI.
  if $locked and (".git" | path exists) {
    let locks = $dirs | each {|d| $d | path relative-to $root | path join "deps.lock" }
    let dirty = (git diff --exit-code -- ...$locks | complete)
    if $dirty.exit_code != 0 {
      error make {
        msg: "deps.lock is out of sync with wit/deps.toml manifests — rerun `update-wit-deps.nu` without --locked and commit the result"
      }
    }
  }
}
