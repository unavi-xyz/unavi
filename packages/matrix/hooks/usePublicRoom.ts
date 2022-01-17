import { MatrixClient } from "matrix-js-sdk";
import useSWR from "swr";

import { getPublicRoom } from "..";

export function usePublicRoom(client: MatrixClient, id: string, world = false) {
  async function fetcher() {
    const res = await getPublicRoom(client, id, world);
    return res;
  }

  const { data } = useSWR(`publicRoom-${id}`, fetcher);

  return data;
}
