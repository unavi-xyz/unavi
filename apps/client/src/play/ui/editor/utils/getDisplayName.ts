export function getDisplayName(value: string | undefined, id: string) {
  return value || `(${id})`;
}
