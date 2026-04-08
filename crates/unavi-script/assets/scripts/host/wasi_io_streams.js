import { Pollable } from "wasi:io/poll";

const cabiDispose = Symbol.for("cabiDispose");
const cabiRep = Symbol.for("cabiRep");

function newPollable() {
  const p = Object.create(Pollable.prototype);
  p[cabiRep] = 0;
  return p;
}

export class InputStream {
  static [cabiDispose](_rep) {}
  read(_len) {
    return new Uint8Array(0);
  }
  blockingRead(_len) {
    return new Uint8Array(0);
  }
  skip(_len) {
    return 0;
  }
  blockingSkip(_len) {
    return 0;
  }
  subscribe() {
    return newPollable();
  }
  drop() {}
}

export class OutputStream {
  static [cabiDispose](_rep) {}
  checkWrite() {
    return BigInt(4096);
  }
  write(_bytes) {}
  blockingWriteAndFlush(_bytes) {}
  flush() {}
  blockingFlush() {}
  writeZeroes(_len) {}
  blockingWriteZeroesAndFlush(_len) {}
  splice(_src, _len) {
    return BigInt(0);
  }
  blockingSplice(_src, _len) {
    return BigInt(0);
  }
  subscribe() {
    return newPollable();
  }
  drop() {}
}
