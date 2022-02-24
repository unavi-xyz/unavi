import useSWR from "swr";

import { Space } from "..";
import { loader } from "../constants";

export function useSpace(streamId: string) {
  async function fetcher() {
    if (!streamId) return;
    const doc = await loader.load(streamId);
    const space = doc.content as Space;
    return space;
  }

  const { data } = useSWR(`space-${streamId}`, fetcher);

  return data;
}
