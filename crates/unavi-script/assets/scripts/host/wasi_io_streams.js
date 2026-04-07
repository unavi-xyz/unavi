const cabiDispose = Symbol.for("cabiDispose");

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
    return null;
  }
  drop() {}
}

export class OutputStream {
  static [cabiDispose](_rep) {}
  checkWrite() {
    return 0;
  }
  write(_bytes) {}
  blockingWriteAndFlush(_bytes) {}
  flush() {}
  blockingFlush() {}
  writeZeroes(_len) {}
  blockingWriteZeroesAndFlush(_len) {}
  splice(_src, _len) {
    return 0;
  }
  blockingSplice(_src, _len) {
    return 0;
  }
  subscribe() {
    return null;
  }
  drop() {}
}
