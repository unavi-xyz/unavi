# Copies needed jco objs to the snippets dir, so the js snippets can fetch them,
# and the transpile worker beside runtime.js, so the main thread's
# `new Worker(new URL("./transpile-worker.js", import.meta.url))` finds it and
# the worker finds the same objs.

let base = (
  if ($env.TRUNK_STAGING_DIR? != null) {
    $env.TRUNK_STAGING_DIR
  } else {
    "dist"
  }
)

const vendor = "crates/unavi-script/node_modules/@bytecodealliance/jco-transpile/vendor"
const worker = "crates/unavi-script/dist/transpile-worker.js"

let wasm_files = [
  $"($vendor)/js-component-bindgen-component.core.wasm",
  $"($vendor)/js-component-bindgen-component.core2.wasm"
]

for wasm in $wasm_files {
  if not ($wasm | path exists) {
    error make { msg: $"jco obj not found: ($wasm)" }
  }
}

if not ($worker | path exists) {
  error make { msg: $"transpile worker not found: ($worker)" }
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

  cp $worker $"($target)/($worker | path basename)"
}
