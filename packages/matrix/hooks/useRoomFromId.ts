import { useEffect, useState } from "react";
import { IPublicRoomsChunkRoom, MatrixClient } from "matrix-js-sdk";
import { getRoom } from "..";

export function useRoomFromId(client: MatrixClient, id: string) {
  const [room, setRoom] = useState<null | IPublicRoomsChunkRoom>(null);

  useEffect(() => {
    if (!client || !id) return;

    getRoom(client, id).then((res) => {
      setRoom(res);
    });
  }, [client, id]);

  return room;
}
