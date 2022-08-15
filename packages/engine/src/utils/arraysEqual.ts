export function arraysEqual(a: number[], b: number[]): boolean {
  if (a.length !== b.length) return false;

  const isEqual = a.every((value, index) => value === b[index]);
  return isEqual;
}
