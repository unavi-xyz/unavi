const protocol_src = "protocol/wit"
const wasm_src = "wasm"

print "Updating protocol deps"

for dir in (ls $protocol_src | where type == "dir") {
  cd $dir.name;

  if ("deps.toml" | path exists) {
    print $"→ Updating (basename $dir.name)"
    wit-deps -d deps -m deps.toml -l deps.lock update;
  }

  cd ../../..;
}

print "Updating WASM deps"

for dir in (ls $wasm_src | where type == "dir") {
  cd $dir.name;

  if ("wit/deps.toml" | path exists) {
    print $"→ Updating (basename $dir.name)"
    rm -r "wit/deps";
    wit-deps update;
  }

  cd ../..;
}
