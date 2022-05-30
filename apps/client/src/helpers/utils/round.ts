export function round(value: number, digits = 3) {
  return Math.round(value * 10 ** digits) / 10 ** digits;
}
