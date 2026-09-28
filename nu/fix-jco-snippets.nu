# Copies needed jco objs to the snippets dir, so the js snippets can fetch them.

let base = (
  if ($env.TRUNK_STAGING_DIR? != null) {
    $env.TRUNK_STAGING_DIR
  } else {
    "dist"
  }
)

const vendor = "crates/unavi-script/node_modules/@bytecodealliance/jco-transpile/vendor"

let wasm_files = [
  $"($vendor)/js-component-bindgen-component.core.wasm",
  $"($vendor)/js-component-bindgen-component.core2.wasm"
]

for wasm in $wasm_files {
  if not ($wasm | path exists) {
    error make { msg: $"jco obj not found: ($wasm)" }
  }
}

let snippet_dirs = (ls $"($base)/snippets" | where type == dir | get name)

for dir in $snippet_dirs {
  let target = $"($dir)/dist"

  if not ($target | path exists) {
    print $"($target) does not exist"
    continue
  }

  for wasm in $wasm_files {
    let dest = $"($target)/($wasm | path basename)"
    cp $wasm $dest
  }
}
