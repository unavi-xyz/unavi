export async function getPlayerCount(id: number) {
  const response = await fetch(`/api/spaces/${id}/player-count`, { method: "GET" });
  const playerCount = (await response.json()) as number;
  return playerCount;
}
