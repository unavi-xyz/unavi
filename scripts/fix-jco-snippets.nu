# Copies needed jco objs to the snippets dir, so the js snippets can fetch them.

let base = (
  if ($env.TRUNK_STAGING_DIR? != null) {
    $env.TRUNK_STAGING_DIR
  } else {
    "dist"
  }
)

let wasm_files = [
  "crates/unavi-script/node_modules/@bytecodealliance/jco/obj/js-component-bindgen-component.core.wasm",
  "crates/unavi-script/node_modules/@bytecodealliance/jco/obj/js-component-bindgen-component.core2.wasm"
]

let snippet_dirs = (ls $"($base)/snippets" | where type == dir | get name)

for dir in $snippet_dirs {
  let target = $"($dir)/dist"

  if not ($target | path exists) {
    print $"($target) does not exist"
    continue
  }

  for wasm in $wasm_files {
    if not ($wasm | path exists) {
      continue
    }

    let dest = $"($target)/($wasm | path basename)"
    cp $wasm $dest
  }
}
