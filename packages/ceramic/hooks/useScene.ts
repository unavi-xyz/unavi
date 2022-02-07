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

  async function merge(data: any) {
    const doc = await loader.load(id);
    const newContent = Object.assign(doc.content, data);
    await doc.update(newContent, undefined, { pin: true });
  }

  async function abandon() {
    const doc = await loader.load(id);
    await doc.update(doc.content, undefined, { pin: false });
  }

  return { scene: data?.scene, author: data?.author, merge, abandon };
}
