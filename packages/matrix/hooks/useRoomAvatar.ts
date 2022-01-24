import { IPublicRoomsChunkRoom, MatrixClient } from "matrix-js-sdk";
import useSWR from "swr";

export function useRoomAvatar(
  client: MatrixClient,
  room: IPublicRoomsChunkRoom
) {
  async function fetcher() {
    if (!room.avatar_url) return;
    const res = client.mxcUrlToHttp(room.avatar_url);
    return res;
  }

  const { data } = useSWR(room, fetcher);
  return data;
}
