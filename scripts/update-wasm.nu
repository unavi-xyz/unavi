const protocol_src = "protocol/wit"
const component_srcs = ["crates", "wasm"]

print "Updating protocol deps"

for dir in (ls $protocol_src | where type == "dir") {
  cd $dir.name;

  if ("deps.toml" | path exists) {
    print $"→ Updating (basename $dir.name)"
    try { rm -r "deps" };
    try { rm "deps.lock" };
    wit-deps -d deps -m deps.toml -l deps.lock update;
  }

  cd ../../..;
}

print "Updating component deps"

for src in $component_srcs {
  for dir in (ls $src | where type == "dir") {
    cd $dir.name;

    if ("wit/deps.toml" | path exists) {
      print $"→ Updating (basename $dir.name)"
      try { rm -r "wit/deps" };
      try { rm "wit/deps.lock" };
      wit-deps update;
    }

    cd ../..;
  }
}
