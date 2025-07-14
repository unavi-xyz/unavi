print "Optimizing WASM assets"

for bin in (ls crates/unavi-app/assets/wasm/**/*.wasm) {
  print $"→ Optimizing ($bin.name)"

  wasm-opt -Os --output $bin.name $bin.name
  let new_bin = ls $bin.name | first

  print $"  | Size ($bin.size) -> ($new_bin.size)"
}
