export function toHex(number: number): string {
  const str = number.toString(16).padStart(2, "0");
  return `0x${str}`;
}
