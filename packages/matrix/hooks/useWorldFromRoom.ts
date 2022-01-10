import { useEffect, useState } from "react";
import { IPublicRoomsChunkRoom, MatrixClient } from "matrix-js-sdk";
import { getRoom, parseRoomTopic } from "..";

export function useWorldFromRoom(client: MatrixClient, roomTopic: string) {
  const [world, setWorld] = useState<null | IPublicRoomsChunkRoom>(null);

  useEffect(() => {
    if (!client || !roomTopic) return;
    const worldId = parseRoomTopic(roomTopic);

    getRoom(client, worldId, true).then((res) => {
      setWorld(res);
    });
  }, [client, roomTopic]);

  return world;
}
