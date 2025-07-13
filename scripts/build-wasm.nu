const wasm_out = "crates/unavi-app/assets/wasm"
const wasm_profile = "release-wasm"
const wasm_src = "wasm"
const wasm_target = "wasm32-wasip2"

print "🧹 Cleaning old wasm outputs..."

rm -rf $wasm_out
mkdir $wasm_out

print "🛠  Building WASM crates..."

for crate_dir in (ls $wasm_src | where type == "dir") {
   let crate = $crate_dir.name | path basename
   let crate_path = $crate_dir.name
   let wasm_file = ($crate | str replace '-' '_') + ".wasm"
   print $"→ Building ($crate)"

   let time = timeit {
     let status = (cargo build 
         --quiet 
         --target $wasm_target 
         --profile $wasm_profile 
         --manifest-path $"($crate_path)/Cargo.toml"
         | complete)

     if $status.exit_code != 0 {
         error make {
             msg: $"❌ Build failed for ($crate): ($status.stderr | str trim)"
         }
     }

     let src_path = $"target/($wasm_target)/($wasm_profile)/($wasm_file)"
     let dst_path = $"($wasm_out)/($wasm_file)"
     cp $src_path $dst_path
  }

  let time_ms = ($time | into int) / 1_000_000 | into int

  print $"  | took ($time_ms)ms"
}

print "📦 Organizing WASM outputs..."

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

print "✅ Done."
