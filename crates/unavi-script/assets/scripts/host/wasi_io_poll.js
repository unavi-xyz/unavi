const cabiDispose = Symbol.for("cabiDispose");

export class Pollable {
  static [cabiDispose](_rep) {}
  ready() {
    return true;
  }
  block() {}
  drop() {}
}
