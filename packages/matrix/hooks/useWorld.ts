import { MatrixClient } from "matrix-js-sdk";
import useSWR from "swr";

import { getWorld } from "..";

export function useWorld(client: MatrixClient, id: string) {
  async function fetcher() {
    const res = await getWorld(client, id);
    return res;
  }

  const { data } = useSWR(`world-${id}`, fetcher);

  return data;
}
