import { useEffect, useState } from "react";
import { MatrixClient, Room } from "matrix-js-sdk";
import { getRoom } from "..";

export function useRoomFromId(client: MatrixClient, id: string) {
  const [room, setRoom] = useState<null | Room>(null);

  useEffect(() => {
    if (!client || !id) return;

    getRoom(client, id).then((res) => {
      setRoom(res);
    });
  }, [client, id]);

  return room;
}
