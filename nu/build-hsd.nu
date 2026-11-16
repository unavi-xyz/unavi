const hsd_out = "crates/unavi-app/assets/hsd"
const hsd_src = "hsd"

def main [
  --crate: string      # Build only this HSD crate
  --no-examples        # Skip the example crates, which only the cargo examples load
  --force              # Rebuild even when every package is newer than its inputs
] {
  let crates = ls $hsd_src | where type == "dir" | where {|d|
    ($"($d.name)/asset.hsda" | path exists)
  }

  let crates = if $no_examples {
    $crates | where {|d| not ($d.name | path basename | str starts-with "example-") }
  } else {
    $crates
  }

  let crates = if $crate != null {
    $crates | where {|d| ($d.name | path basename) == $crate }
  } else {
    $crates
  }

  if ($crates | is-empty) {
    error make { msg: $"no HSD crate named '($crate)'" }
  }

  if not $force and (up_to_date $crates) {
    print "HSD assets up to date"
    return
  }

  if $crate != null {
    rm -f (output_path $crate)
  } else {
    print "Building HSD assets"
    rm -rf $hsd_out
    mkdir $hsd_out
  }

  for crate_dir in $crates {
    let crate = $crate_dir.name | path basename
    print $"→ ($crate)"

    let hsda = $"($crate_dir.name)/asset.hsda"

    let build = (cargo run --quiet -p hsd-cli --release -- build
        --input $hsda
        --out-dir $hsd_out
      | complete)
    if $build.exit_code != 0 {
      error make {
        msg: $"HSD build failed for ($crate): ($build.stderr | str trim)"
      }
    }
  }
}

def output_path [crate: string] {
  $"($hsd_out)/($crate | str replace -a '-' '_').hsdz"
}

# Every file a package build reads: the guest sources and their WIT, the
# format and packer the packages are written with, the app assets a package
# may embed, and the workspace manifests. Build outputs are skipped.
def newest_input [] {
  let tree = glob "{hsd,protocol/wit,crates/hsd,crates/hsd-cli,crates/unavi-app/assets}/**/*" --no-dir --exclude ["**/target/**" "crates/unavi-app/assets/hsd/**"]
  $tree | append ["Cargo.toml" "Cargo.lock"]
  | par-each {|file| ls -l $file | get 0.modified }
  | math max
}

# Whether every selected package exists and is newer than every input. The
# check is coarse, since guests share library crates, so any input change
# rebuilds every package.
def up_to_date [crates: list<any>] {
  let outputs = $crates | each {|dir| output_path ($dir.name | path basename) }
  if ($outputs | any {|out| not ($out | path exists) }) {
    return false
  }
  let oldest = $outputs | each {|out| ls -l $out | get 0.modified } | math min
  (newest_input) < $oldest
}
