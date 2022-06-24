export function deepClone(obj: any): any {
  return JSON.parse(JSON.stringify(obj));
}
