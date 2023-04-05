export function deletePublication(id: string) {
  return fetch(`/api/publications/${id}`, { method: "DELETE" });
}
