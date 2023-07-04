export function deleteWorld(id: string) {
  return fetch(`/api/worlds/${id}`, { method: "DELETE" });
}
