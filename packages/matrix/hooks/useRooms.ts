import { MatrixClient } from "matrix-js-sdk";
import useSWR from "swr";

import { getPublicRooms } from "..";

export function useRooms(client: MatrixClient) {
  async function fetcher() {
    const res = await getPublicRooms(client);
    return res;
  }

  const { data } = useSWR("a", fetcher);

  return data;
}
