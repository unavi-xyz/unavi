import { useEffect, useState } from "react";
import { IPublicRoomsChunkRoom, MatrixClient } from "matrix-js-sdk";
import { getWorldInstances } from "..";

export function useRoomsFromWorld(client: MatrixClient, id: string) {
  const [rooms, setRooms] = useState<IPublicRoomsChunkRoom[]>([]);

  useEffect(() => {
    if (!client || !id) return;

    getWorldInstances(client, id).then((res) => {
      setRooms(res);
    });
  }, [client, id]);

  return rooms;
}
