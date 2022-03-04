import useSWR from "swr";

import { Room } from "../models/Room/types";
import { loader } from "../client";
import { getImageUrl } from "../ipfs";

export function useRoom(streamId: string) {
  async function fetcher() {
    if (!streamId) return;
    const doc = await loader.load(streamId);
    const room = doc.content as Room;

    if (room?.image) {
      room.image = getImageUrl(room.image);
    }

    return { room, controller: doc.controllers[0] };
  }

  const { data } = useSWR(`room-${streamId}`, fetcher);

  return { room: data?.room, controller: data?.controller };
}
