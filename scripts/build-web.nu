# Build WebGL and WebGPU web clients.
def main [
  --release         # Build with release optimizations.
] {
  let trunk_args = if $release { ["--release"] } else { [] }

  print "Building WebGL variant..."
  let time_webgl = timeit {
    (run-external "trunk" "build"
      "--dist" "dist-webgl"
      "--public-url" "/webgl/"
      ...$trunk_args
      "crates/unavi-client/index.html")
  }
  print $"  | WebGL build time: ($time_webgl)"

  print "Building WebGPU variant..."
  let time_webgpu = timeit {
    (run-external "trunk" "build"
      "--dist" "dist-webgpu"
      "--public-url" "/webgpu/"
      "--features" "webgpu"
      ...$trunk_args
      "crates/unavi-client/index.html")
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
