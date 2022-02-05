import useSWR from "swr";
import { loader, Scene } from "..";

export function useScene(id: string) {
  async function fetcher() {
    if (!id) return;
    const doc = await loader.load(id);
    const scene = doc.content as Scene;
    const author = doc.metadata.controllers[0];

    return { scene, author };
  }

  const { data } = useSWR(`scene-${id}`, fetcher);

  return data ?? { scene: undefined, author: undefined };
}
