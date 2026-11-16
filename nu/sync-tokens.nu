const source = "../unum/crates/website/public/tokens.css"
const copy = "assets/tokens.css"

# Copies the website's token file into assets/. Run from the repo root.
def main [
  --check # Exit non-zero when the copy differs from the source, and change nothing
] {
  if not ($source | path exists) {
    error make { msg: $"missing ($source)" }
  }

  let same = ($copy | path exists) and ((open --raw $source) == (open --raw $copy))

  if $check {
    if not $same {
      error make { msg: $"($copy) differs from ($source); run nu/sync-tokens.nu" }
    }
    print "tokens.css is in sync"
    return
  }

  if $same {
    print "tokens.css already in sync"
  } else {
    cp $source $copy
    print $"copied ($source) to ($copy)"
  }
}
