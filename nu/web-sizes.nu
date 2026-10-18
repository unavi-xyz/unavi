# Writes dist/sizes.json with the uncompressed wasm size of each variant. The
# loader uses it as the denominator for download progress, since the browser's
# decoded stream counts uncompressed bytes. Depends on `just web` having
# populated dist/webgl and dist/webgpu.

const variants = ["webgl" "webgpu"]

def wasm_size [variant: string] {
  let wasm = (glob $"dist/($variant)/*_bg.wasm")
  if ($wasm | length) != 1 {
    error make { msg: $"expected one dist/($variant)/*_bg.wasm, found ($wasm | length)" }
  }
  ls ($wasm | first) | get size | first | into int
}

def main [] {
  $variants
    | reduce --fold {} {|variant, sizes| $sizes | insert $variant (wasm_size $variant) }
    | save -f dist/sizes.json
}
