export function toHex(num: number) {
  return `0x${num.toString(16).padStart(2, "0")}`;
}
