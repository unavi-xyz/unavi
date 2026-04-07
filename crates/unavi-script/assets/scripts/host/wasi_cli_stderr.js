import { OutputStream } from "wasi:io/streams";
export function getStderr() {
  const s = Object.create(OutputStream.prototype);
  s.write = (bytes) => {
    const text = new TextDecoder().decode(bytes);
    console.error("[wasm]", text);
  };
  return s;
}
