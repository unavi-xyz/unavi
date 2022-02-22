import useSWR from "swr";
import { loader, Post } from "..";

export function usePost(id: string) {
  async function fetcher() {
    if (!id) return;
    const doc = await loader.load(id);
    const post = doc.content as Post;
    const controller = doc.metadata.controllers[0];
    return { post, controller };
  }

  const { data } = useSWR(`post-${id}`, fetcher);

  return data ?? { post: undefined, controller: undefined };
}
