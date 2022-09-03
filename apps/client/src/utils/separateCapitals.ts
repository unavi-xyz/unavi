export function separateCapitals(str: string): string {
  return str.replace(/([A-Z])/g, " $1");
}
