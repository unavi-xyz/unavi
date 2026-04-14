const hsd_out = "crates/unavi-client/assets/hsd"
const wasm_src = "wasm"

print "Building HSD assets"

rm -rf $hsd_out
mkdir $hsd_out

ls $wasm_src | where type == "dir" | where {|d|
    ($"($d.name)/asset.hsdx" | path exists)
} | each {|crate_dir|
    let crate = $crate_dir.name | path basename
    print $"→ ($crate)"
    let status = (cargo run --quiet -p hsd-cli --release -- build
        --input $"($crate_dir.name)/asset.hsdx"
        --out-dir $hsd_out
        | complete)
    if $status.exit_code != 0 {
        error make {
            msg: $"HSD build failed for ($crate): ($status.stderr | str trim)"
        }
    }
}
