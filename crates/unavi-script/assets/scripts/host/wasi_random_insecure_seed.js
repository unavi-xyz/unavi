export function insecureSeed() {
  const buf = new BigUint64Array(2);
  crypto.getRandomValues(buf);
  return [buf[0], buf[1]];
}
