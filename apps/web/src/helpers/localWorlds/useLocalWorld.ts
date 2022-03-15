import { useQuery } from "react-query";
import { getLocalWorld } from "./db";

export function useLocalWorld(id: string) {
  async function fetcher() {
    if (!id) return;
    const world = await getLocalWorld(id);
    return world;
  }

  const { data } = useQuery(`local-world-${id}`, fetcher);

  return data;
}
