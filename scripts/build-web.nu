# Build WebGL and WebGPU web clients.
def main [
  --release
] {
  npm-setup
  build-js

  let trunk_args = if $release { ["--release"] } else { [] }

  print "Building WebGL variant..."
  let time_webgl = timeit {
    (run-external "trunk" "build"
      "--dist" "dist-webgl"
      "--public-url" "/webgl/"
      "crates/unavi-client/index.html"
      ...$trunk_args
    )
  }
  print $"  | WebGL build time: ($time_webgl)"

  print "Building WebGPU variant..."
  let time_webgpu = timeit {
    (run-external "trunk" "build"
      "--dist" "dist-webgpu"
      "--public-url" "/webgpu/"
      "--features" "webgpu"
      "crates/unavi-client/index.html"
      ...$trunk_args
    )
  }
  print $"  | WebGPU build time: ($time_webgpu)"

  print "Combining outputs..."
  rm -rf dist
  mkdir dist

  # Copy loader as entry point.
  cp crates/unavi-client/loader.html dist/index.html

  # Move variant builds to subdirs.
  mv dist-webgl dist/webgl
  mv dist-webgpu dist/webgpu

  print "Build complete: dist/"
}

# Install npm dependencies
def npm-setup [] {
  print "Installing npm dependencies..."
  run-external "npm" "install" "--prefix" "crates/unavi-script" "--silent"
}

def build-js [] {
  let file = "runtime"
  let out = $"crates/unavi-script/dist/($file).js"
  print $"Bundling jco runtime → ($out)"

  # Use jco's browser.js (the ./component export) and exclude the node:fs/promises
  # dynamic import used only for the optional Binaryen optimisation pass.
  (run-external "esbuild"
    $"crates/unavi-script/($file).ts"
    "--bundle"
    "--format=esm"
    "--platform=browser"
    "--external:node:fs/promises"
    $"--outfile=($out)"
  )
}
