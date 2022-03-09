import { useQuery } from "react-query";

import { Room } from "../models/Room/types";
import { loader } from "../client";

export function useRoom(streamId: string) {
  async function fetcher() {
    if (!streamId) return;
    const doc = await loader.load(streamId);
    const room = doc.content as Room;

    return { room, controller: doc.controllers[0] };
  }

  const { data } = useQuery(`room-${streamId}`, fetcher);

  return { room: data?.room, controller: data?.controller };
}
