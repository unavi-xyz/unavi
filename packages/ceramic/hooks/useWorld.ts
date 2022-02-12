import useSWR from "swr";
import { loader, World } from "..";

export function useWorld(id: string) {
  async function fetcher() {
    if (!id) return;
    const doc = await loader.load(id);
    const world = doc.content as World;
    const controller = doc.metadata.controllers[0];

    return { world, controller };
  }

  const { data } = useSWR(`world-${id}`, fetcher);

  return data ?? { world: undefined, controller: undefined };
}
