# Bundle-size table for the web builds in dist/. Depends on `just web`, which
# populates dist/webgl and dist/webgpu.

const variants = ["webgl" "webgpu"]

# Bytes in one file, raw or through a compressor. An external command's stdout
# is a byte stream, so its length is the compressed size with no temp file.
def compressed_size [path: string, mode: string] {
  match $mode {
    "raw" => (ls $path | get size | math sum | into int)
    "gzip" => (^gzip -6 -c $path | bytes length)
    "brotli" => (^brotli -q 11 -c $path | bytes length)
  }
}

# Every file under a path, or the path itself when it is a file.
def expand [path: string] {
  if ($path | path type) == "dir" {
    glob $"($path)/**/*" --no-dir
  } else {
    [$path]
  }
}

# Summed raw/gzip/brotli sizes for a set of files and directories. Each file
# is compressed on its own, so the totals match what a per-file server sends.
def measure [paths: list<string>, brotli: bool] {
  let files = ($paths | each {|path| expand $path } | flatten)
  let gzip = (size_of $files "gzip")
  {
    raw:    (size_of $files "raw")
    gzip:   $gzip
    brotli: (if $brotli { size_of $files "brotli" } else { $gzip })
  }
}

# The summed size of a set of files under one mode; zero for none.
def size_of [files: list<string>, mode: string] {
  if ($files | is-empty) {
    0
  } else {
    $files | each {|file| compressed_size $file $mode } | math sum
  }
}

def total [measures: list] {
  {
    raw:    ($measures | get raw | math sum)
    gzip:   ($measures | get gzip | math sum)
    brotli: ($measures | get brotli | math sum)
  }
}

def render [rows: list] {
  [{ part: "part", raw: "raw", gzip: "gzip -6", brotli: "brotli -q 11" }]
    | append $rows
    | each {|row|
        $"($row.part | fill -w 16)($row.raw | into string | fill -w 14 -a right)($row.gzip | into string | fill -w 14 -a right)($row.brotli | into string | fill -w 16 -a right)"
      }
    | str join (char nl)
}

def main [] {
  let has_brotli = (which brotli | is-not-empty)

  mut rows = []
  for variant in $variants {
    let dir = $"dist/($variant)"
    let parts = [
      { part: "wasm", paths: (glob $"($dir)/*_bg.wasm") }
      { part: "js", paths: ((glob $"($dir)/*.js") | append (glob $"($dir)/snippets/**/*.js")) }
      { part: "assets", paths: [$"($dir)/assets"] }
    ]
    let measures = ($parts | each {|part| measure $part.paths $has_brotli })
    for pair in ($parts | zip $measures) {
      $rows = ($rows | append { part: $"($variant) ($pair.0.part)" ...$pair.1 })
    }
    $rows = ($rows | append { part: $"($variant) total" ...(total $measures) })
  }

  let root = (measure (glob "dist/*" --no-dir | where {|f| ($f | path basename) != "metrics.txt" }) $has_brotli)
  let totals = ($rows | where {|row| $row.part | str ends-with "total" } | select raw gzip brotli)
  $rows = ($rows | append { part: "dist root" ...$root })
  $rows = ($rows | append { part: "dist total" ...(total ($totals | append $root)) })

  let note = (if $has_brotli {
    "bytes, each file compressed on its own and summed"
  } else {
    "bytes; brotli CLI not found, so the brotli column repeats gzip -6"
  })
  let out = (["UNAVI web bundle sizes" $note "" (render $rows)] | str join (char nl))
  print $out
  $out | save -f "dist/metrics.txt"
}
