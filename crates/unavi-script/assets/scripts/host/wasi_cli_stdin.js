import { InputStream } from "wasi:io/streams";
export function getStdin() {
  const stream = Object.create(InputStream.prototype);
  return stream;
}
