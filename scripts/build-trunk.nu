# WASM toolchain from Nix devshell:
# - clang-unwrapped: avoids Nix hardening flags breaking ring
# - libclang headers: provides stddef.h etc. for rusqlite/SQLite
# - Unset C_INCLUDE_PATH to prevent glibc headers polluting WASM build
hide-env C_INCLUDE_PATH

with-env {
  CC_wasm32_unknown_unknown: $env.WASM_CC
  AR_wasm32_unknown_unknown: $env.WASM_AR
  CFLAGS_wasm32_unknown_unknown: $env.WASM_CFLAGS
} {
  trunk build
}
