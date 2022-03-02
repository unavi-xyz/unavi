import useSWR from "swr";

import { Space } from "../models/Space/types";
import { loader } from "../client";
import { getImageUrl } from "../ipfs";

export function useSpace(streamId: string) {
  async function fetcher() {
    if (!streamId) return;
    const doc = await loader.load(streamId);
    const space = doc.content as Space;
    const controller = doc.controllers[0];

    if (space?.image) {
      space.image = getImageUrl(space.image);
    }

    return { space, controller };
  }

  const { data } = useSWR(`space-${streamId}`, fetcher);

  return { space: data?.space, controller: data?.controller };
}
