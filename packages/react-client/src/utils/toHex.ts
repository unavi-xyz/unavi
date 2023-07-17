export function toHex(num: number) {
  return num.toString(16).padStart(4, "0x");
}
