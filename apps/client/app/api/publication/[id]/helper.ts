export function deletePublication(id: number) {
  return fetch(`/api/publication/${id}`, { method: "DELETE" });
}
