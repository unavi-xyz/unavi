import { MatrixClient } from "matrix-js-sdk";
import useSWR from "swr";

import { getPublicRoom } from "..";

export function useRoom(client: MatrixClient, id: string) {
  async function fetcher() {
    const res = await getPublicRoom(client, id);
    return res;
  }

  const { data } = useSWR(`room-${id}`, fetcher);

  return data;
}
