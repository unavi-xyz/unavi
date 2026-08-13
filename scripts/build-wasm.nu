const hsd_out = "crates/unavi-client/assets/hsd"
const wasm_src = "wasm"

def main [
  --crate: string  # Build only this wasm crate
] {
  let crates = ls $wasm_src | where type == "dir" | where {|d|
    ($"($d.name)/asset.hsda" | path exists)
  }

  let crates = if $crate != null {
    $crates | where {|d| ($d.name | path basename) == $crate }
  } else {
    $crates
  }

  if ($crates | is-empty) {
    error make { msg: $"no wasm crate named '($crate)'" }
  }

  if $crate != null {
    let out = $"($hsd_out)/($crate | str replace -a '-' '_').hsdz"
    rm -f $out
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

    let fmt = (cargo run --quiet -p hsd-cli --release -- format $hsda | complete)
    if $fmt.exit_code != 0 {
      error make {
        msg: $"HSD format failed for ($crate): ($fmt.stderr | str trim)"
      }
    }
  }
}
