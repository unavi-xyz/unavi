import { useEffect, useState } from "react";
import { IPublicRoomsChunkRoom, MatrixClient } from "matrix-js-sdk";
import { getPublicRoom, parseRoomTopic } from "..";

export function useWorldFromRoom(client: MatrixClient, roomTopic: string) {
  const [world, setWorld] = useState<null | IPublicRoomsChunkRoom>(null);

  useEffect(() => {
    if (!client || !roomTopic) return;
    const worldId = parseRoomTopic(roomTopic);

    getPublicRoom(client, worldId, true).then((res) => {
      setWorld(res);
    });
  }, [client, roomTopic]);

  return world;
}
