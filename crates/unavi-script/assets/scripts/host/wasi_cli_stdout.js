import { OutputStream } from "wasi:io/streams";
export function getStdout() {
  const stream = Object.create(OutputStream.prototype);
  stream.write = (bytes) => {
    const text = new TextDecoder().decode(bytes);
    console.log("[wasm]", text);
  };
  return stream;
}
