# Wipe the local dev document stores so the home, spaces and portals re-import
# from the current HSD assets. Identity keys stay, so the DID is unchanged.
#
# A script is embedded in the document that holds it, so a rebuilt guest does
# not reach a space that was authored before the change: only a fresh import
# does. Stop the node and apps first, or a running store re-creates its files
# as they are removed.

const targets = [
  { project: "unavi-app",  sub: "store" }
  { project: "unavi-app",  sub: "docs" }
  { project: "unavi-node", sub: "store" }
]

def data_root [] {
  if "XDG_DATA_HOME" in $env {
    $env.XDG_DATA_HOME | path expand
  } else {
    "~/.local/share" | path expand
  }
}

def main [--dry-run] {
  let root = data_root
  for t in $targets {
    let path = $root | path join $t.project $t.sub
    if ($path | path exists) {
      if $dry_run {
        print $"would remove ($path)"
      } else {
        rm -rf $path
        print $"removed      ($path)"
      }
    } else {
      print $"absent       ($path)"
    }
  }
}
