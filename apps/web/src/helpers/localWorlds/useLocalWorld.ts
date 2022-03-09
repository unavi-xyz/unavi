import { useQuery } from "react-query";
import { getLocalWorld } from "./db";

export default function useLocalWorld(id: string) {
  async function fetcher() {
    const world = await getLocalWorld(id);
    return world;
  }

  const { data } = useQuery(`local-world-${id}`, fetcher);

  return data;
}
