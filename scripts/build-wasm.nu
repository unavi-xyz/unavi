const wasm_out = "crates/unavi-client/assets/wasm"
const wasm_profile = "release-wasm"
const wasm_src = "wasm"
const wasm_target = "wasm32-wasip2"

print "Building WASM crates"

rm -rf $wasm_out
mkdir $wasm_out

let time = timeit {
  ls $wasm_src | where type == "dir" | par-each {|crate_dir|
    let crate = $crate_dir.name | path basename
    let crate_path = $crate_dir.name
    let wasm_file = ($crate | str replace '-' '_') + ".wasm"
    let target_dir = $"target/($crate)"

    mut logs = $"→ Building ($crate)\n"

    let status = (cargo build 
        --quiet 
        --target $wasm_target 
        --profile $wasm_profile 
        --manifest-path $"($crate_path)/Cargo.toml"
        --target-dir $target_dir
        | complete)

    if $status.exit_code != 0 {
        error make {
            msg: $"❌ Build failed for ($crate): ($status.stderr | str trim)"
        }
    }

    let src_path = $"($target_dir)/($wasm_target)/($wasm_profile)/($wasm_file)"
    let dst_path = $"($wasm_out)/($wasm_file)"

    $logs += $"  | optimizing\n"
    let b_info = ls $src_path | first
    wasm-opt -O4 --enable-bulk-memory-opt -ffm $src_path -o $dst_path

    $logs += $"  | converting to component\n"
    # https://github.com/bytecodealliance/wasmtime/tree/main/crates/wasi-preview1-component-adapter
    wasm-tools component new $dst_path -o $dst_path --adapt scripts/wasi_snapshot_preview1.reactor.wasm

    let a_info = ls $dst_path | first
    $logs += $"  | size ($b_info.size) -> ($a_info.size)"

    print $logs
  }
}

print $"Total build time: ($time)"

for file in (ls $wasm_out) {
  let base = ($file.name | path basename | str replace '.wasm' '')

  let i = ($base | split chars | enumerate | find "_" | get index | first)
  let ns = ($base | str substring ..($i - 1))
  let rest = ($base | str substring ($i + 1)..)

  let ns_dir = $"($wasm_out)/($ns)"
  mkdir $ns_dir

  let src = $file.name
  let dst = $"($ns_dir)/($rest).wasm"

  print $"→ Moving ($src) → ($dst)"
  mv $src $dst
}
