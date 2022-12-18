export function numberToHexDisplay(number: number): string {
  const str = number.toString(16).padStart(2, "0");
  return `0x${str}`;
}

export function hexDisplayToNumber(hex: string): number {
  return parseInt(hex.replace("0x", ""), 16);
}
