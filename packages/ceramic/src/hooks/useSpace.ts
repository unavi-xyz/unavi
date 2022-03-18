import { useQuery } from "react-query";

import { Space } from "../models/Space/types";
import { loader } from "../client";

export function useSpace(streamId: string) {
  async function fetcher() {
    if (!streamId) return;
    const doc = await loader.load(streamId);
    const space = doc.content as Space;

    return { space, controller: doc.controllers[0] };
  }

  const { data } = useQuery(`space-${streamId}`, fetcher);

  return { space: data?.space, controller: data?.controller };
}
