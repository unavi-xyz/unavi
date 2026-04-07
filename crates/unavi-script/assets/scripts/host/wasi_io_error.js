const cabiDispose = Symbol.for("cabiDispose");

export class Error {
  static [cabiDispose](_rep) {}
  toDebugString() {
    return "wasi:io/error";
  }
  drop() {}
}
