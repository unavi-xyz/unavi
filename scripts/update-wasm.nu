const wasm_src = "wasm"

print "Updating WASM deps"

for dir in (ls $wasm_src | where type == "dir") {
  print $"→ Updating (basename $dir.name)"
  cd $dir.name;
  wit-deps update;
  cd ../..;
}
