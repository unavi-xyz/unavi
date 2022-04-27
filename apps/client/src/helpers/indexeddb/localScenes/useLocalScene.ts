import { useQuery } from "react-query";
import { getLocalScene } from "./db";

export function useLocalScene(id: string) {
  async function fetcher() {
    if (!id) return;
    const localScene = await getLocalScene(id);
    return localScene;
  }

  const { data } = useQuery(`local-scene-${id}`, fetcher);

  return data;
}
