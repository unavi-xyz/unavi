export function deletePublication(id: number) {
  return fetch(`/api/publications/${id}`, { method: "DELETE" });
}
