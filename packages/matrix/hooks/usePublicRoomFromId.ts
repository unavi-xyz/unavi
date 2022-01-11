import { useEffect, useState } from "react";
import { IPublicRoomsChunkRoom, MatrixClient } from "matrix-js-sdk";
import { getPublicRoom } from "..";

export function usePublicRoomFromId(client: MatrixClient, id: string) {
  const [room, setRoom] = useState<null | IPublicRoomsChunkRoom>(null);

  useEffect(() => {
    if (!client || !id) return;

    getPublicRoom(client, id).then((res) => {
      setRoom(res);
    });
  }, [client, id]);

  return room;
}
