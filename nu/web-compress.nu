# Writes a .br (brotli -q 11) and a .gz (gzip -9) next to every compressible
# file in dist/, for nginx's brotli_static and gzip_static. Formats that are
# already compressed (png, ktx2, hsdz, glb, vrm) are left alone. Depends on
# `just web`.

const extensions = ["wasm" "js" "mjs" "css" "html" "json" "wgsl" "svg" "txt"]

def main [] {
  let files = (
    glob "dist/**/*" --no-dir
      | where {|file| ($file | path parse | get extension) in $extensions }
  )

  for file in $files {
    let brotli = (^brotli -q 11 -f -k -o $"($file).br" $file | complete)
    if $brotli.exit_code != 0 {
      error make { msg: $"brotli failed for ($file): ($brotli.stderr | str trim)" }
    }
    let gzip = (^gzip -9 -f -k -n $file | complete)
    if $gzip.exit_code != 0 {
      error make { msg: $"gzip failed for ($file): ($gzip.stderr | str trim)" }
    }
  }

  print $"compressed ($files | length) files"
}
