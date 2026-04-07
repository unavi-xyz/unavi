import { OutputStream } from "wasi:io/streams";
export function getStdout() {
  const stream = Object.create(OutputStream.prototype);
  return stream;
}
