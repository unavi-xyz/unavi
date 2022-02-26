import useSWR from "swr";

import { Space } from "../models/Space/types";
import { loader } from "../client";
import { getImageUrl } from "../ipfs";

export function useSpace(streamId: string) {
  async function fetcher() {
    if (!streamId) return;
    const doc = await loader.load(streamId);
    const space = doc.content as Space;

    if (space?.image) {
      space.image = getImageUrl(space.image);
    }

    return space;
  }

  const { data } = useSWR(`space-${streamId}`, fetcher);

  return data;
}
