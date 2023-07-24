export function getDisplayName(value: string | undefined, id: bigint) {
  return value || `(${id})`;
}
