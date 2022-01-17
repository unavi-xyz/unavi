import { MatrixClient } from "matrix-js-sdk";
import useSWR from "swr";

import { getRoom } from "..";

export function useRoom(client: MatrixClient, id: string) {
  async function fetcher() {
    const res = await getRoom(client, id);
    return res;
  }

  const { data } = useSWR(id, fetcher);

  return data;
}
