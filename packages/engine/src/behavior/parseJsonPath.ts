export function parseJSONPath(jsonPath: string) {
  const parts = jsonPath.split("/") as string[];

  const resource = parts[1];
  const indexString = parts[2];
  const property = parts[3];

  if (!resource || !indexString || !property) return;

  const index = parseInt(indexString);

  return { resource, index, property };
}
